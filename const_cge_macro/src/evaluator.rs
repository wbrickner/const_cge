use cge::{gene::{Gene, NeuronId}, network::NeuronInfo};
use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident};
use std::{collections::HashMap, ops::Range};
use crate::{stack::Stack, numeric_type::NumericType};

/// Generates unique variable identifiers (names).
#[derive(Default)]
pub struct ResultNames { index: u64 }

impl ResultNames {
  /// Get the next name, modifying the generator.
  pub fn advance(&mut self) -> Ident {
    let ident = format_ident!("c{}", self.index);
    self.index += 1;
    ident
  }

  /// Get the most recently generated name.
  pub fn last(&self) -> Ident {
    format_ident!("c{}", self.index.saturating_sub(1))
  }
}

/// Generate a list of low-level floating-point operations from CGE.
/// This is the meat.
pub fn evaluate(
  genome: &[Gene<f64>],                        // The network to evaluate
  neuron_info: &HashMap<NeuronId, NeuronInfo>, // The neuron info for the network
  range: Range<usize>,                         // Range of genes to evaluate
  neuron_update: bool,                         // Should the execution of this subnetwork update the neuron values?
  j: bool,                                     // I do not understand this flag. You caught me.
  root: bool,                                  // Is this the 'root' invocation of this function? used for writing into the outputs array.
  computations: &mut Vec<TokenStream>,         // Computations tracks the actual expressions & assignments (e.g. `let c2 = (w0 * c0) + (w1 * c1);`)
  computations_end: &mut Vec<TokenStream>,     // Stuff to tack onto the end.

  result_names: &mut ResultNames,          // Counts upwards and is used for making variable names
  recurrence_table: &HashMap<NeuronId, usize>, // A complete table mapping all neuron IDs to the index in the "persistence array"
                                               // - the persistence array is a minimal set of floats that are needed to support the recurrent behavior of the network
                                               // - if there are 6 "backwards" connections, then the persistence array will need 6 floats, and this hashmap will contain
                                               //   6 entries.
  numeric_type: NumericType,                   // The _target_ numeric type to be used in the generated code.
  activation:   TokenStream,                   // Path to optimized activation function (e.g. `const_cge::activations::f32::relu`)
) -> Option<usize> {
  let mut stack = Stack::new();
  
  for gene_index in range.clone().rev() {
    match &genome[gene_index] {
      Gene::Input(input) => {
        // If the gene is an input, push its value multiplied by the inputs weight onto
        // the stack
        let input_id = input.id().as_usize();
        let input_weighting = numeric_type.naive_conversion(input.weight());
        let result_id = result_names.advance();

        // the input ID of `j` corresponds to the `j`th element of input buffer
        computations.push(quote! { let #result_id = #input_weighting * inputs[#input_id]; });
        stack.push(result_id);
      },
      Gene::Neuron(n) => {
        // If the gene is a neuron, pop a number (the neurons input count) of inputs
        // off the stack, and push the transfer function applied to the sum of these
        // inputs multiplied by the neurons weight onto the stack
        let weight = n.weight();
        let neuron_id = n.id();
        let input_count = n.num_inputs();
        let result_id = result_names.advance();

        // sum the most recently visited N inputs
        let mut inputs = stack
          .pop(input_count)
          .unwrap_or_else(|| panic!("Corrupt CGE: neuron (ID {:?}) did not receive enough inputs (expected {}, but only received {})", neuron_id, input_count, stack.data.len()));

        // reverse the order of sum to provide a perfect bitwise match with `cge`.
        inputs.reverse();

        computations.push(quote! {
          let #result_id = #(#inputs)+*;            // sum the inputs for neuron ##neuron_id
          let #result_id = #activation(#result_id); // apply activation function
        });

        if neuron_update {
          if let Some(index) = recurrence_table.get(&neuron_id) {
            // if we're told to update state, and if this neuron is recurrent (may not be!),
            // update the persistence array (but delay until the end)
            computations_end.push(quote! { self.persistence[#index] = #result_id; });
          }
        }

        // when j flag is set, do not include weight of last neuron link as jump forward has a different weight
        if !j || gene_index != range.start {
          // otherwise use regular weight of connection in stack
          let weight = numeric_type.naive_conversion(weight);
          let new_result_id = result_names.advance();
          computations.push(quote! { let #new_result_id = #result_id * #weight; });
          stack.push(new_result_id);
        } else {
          stack.push(result_id);
        }
      },
      Gene::ForwardJumper(f) => {
        // If the gene is a forward jumper, evaluate the subnetwork starting at the
        // neuron with id of the jumper, and push the result multiplied by the jumpers
        // weight onto the stack
        let weight = numeric_type.naive_conversion(f.weight());
        let id = f.source_id(); // THIS IS A GUESS, VERIFY WITH OWEN
        let subnetwork_range = neuron_info[&id].subgenome_range();

        // set j flag to true so the neuron does not include it's regular link weight
        // otherwise the values will be off by whatever factor the neuron weight is
        evaluate(
          genome,
          neuron_info,
          subnetwork_range,
          false,
          true,
          false,
          computations,
          computations_end,
          result_names,
          recurrence_table,
          numeric_type,
          activation.clone()
        );

        let subnetwork_result_id = result_names.last();
        let weighted_result_id = result_names.advance();
        computations.push(quote! { let #weighted_result_id = #subnetwork_result_id * #weight; });
        stack.push(weighted_result_id);
      },
      Gene::RecurrentJumper(r) => {
        // If the gene is a recurrent jumper, push the previous value of the neuron
        // with the id of the jumper multiplied by the jumpers weight onto the stack
        let persistence_index = recurrence_table
          .get(&r.source_id())
          .unwrap_or_else(|| panic!("Corrupt CGE: encountered a recurrent connection (gene {}) with an invalid neuron ID ({})", gene_index, r.source_id().as_usize()));

        // this is useless code (`let c137 = self.persistence[2];`)
        // however it lets us keep the same process consistent & LLVM will optimize it out I'm p sure
        let result_id = result_names.advance();
        let weight = numeric_type.naive_conversion(r.weight());

        // access persistence, apply weighting
        computations.push(quote! { let #result_id = #weight * self.persistence[#persistence_index]; });
        stack.push(result_id);
      },
      Gene::Bias(b) => {
        // If the gene is a bias input, push the bias constant multiplied by the genes
        // weight onto the stack
        let bias = numeric_type.naive_conversion(b.value());

        // this is more junk rustc will const-propagate / LLVM will optimize (`let c137 = -0.02302234`);
        // NOTE: maybe it could be helpful if we declare like `const c137: #NUMERIC_TYPE = -0.02302234;`?
        //       An immutible literal /has the same properties as a constant/, but idk if rustc treats them the identically or not.
        let result_id = result_names.advance();
        computations.push(quote! { let #result_id = #bias; });
        stack.push(result_id);
      }
    }
  }

  // now the stack contains the identifiers (variable names) of the result of the network.
  // - we must write them into output buffer rather than return them
  if root {
    let output_count = stack.data.len();

    for (index, identifier) in stack.data.iter().enumerate() {
      computations.push(quote! {
        outputs[#index] = #identifier; // store network output in output buffer
      });
    }

    Some(output_count)
  } else {
    None
  }
}
