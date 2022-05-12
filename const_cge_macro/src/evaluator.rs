use cge::{gene::GeneExtras, Network};
use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use std::{collections::HashMap, ops::Range};
use crate::{stack::Stack, numeric_type::NumericType};

/// Generate a list of low-level floating-point operations from CGE.
/// This is the meat.
pub fn evaluate(
  network: &mut Network,                    // The network to evaluate
  range: Range<usize>,                      // Range of genes to evaluate
  neuron_update: bool,                      // Should the execution of this subnetwork update the neuron values?
  j: bool,                                  // I do not understand this flag. You caught me.
  root: bool,                               // Is this the 'root' invocation of this function? used for writing into the outputs array.
  computations: &mut Vec<TokenStream>,      // Computations tracks the actual expressions & assignments (e.g. `let c2 = (w0 * c0) + (w1 * c1);`)
  computations_end: &mut Vec<TokenStream>,  // Stuff to tack onto the end.

  latest_output: &mut u64,                  // Counts upwards and is used for making variable names
  recurrence_table: &HashMap<usize, usize>, // A complete table mapping all neuron IDs to the index in the "persistence array"
                                            // - the persistence array is a minimal set of floats that are needed to support the recurrent behavior of the network
                                            // - if there are 6 "backwards" connections, then the persistence array will need 6 floats, and this hashmap will contain
                                            //   6 entries.
  numeric_type: NumericType,                // The _target_ numeric type to be used in the generated code.
  activation:   TokenStream                 // Path to optimized activation function (e.g. `const_cge::activations::f32::relu`)
) -> Option<usize> {
  let mut stack = Stack::new(); // stack tracks the /names/ of past results (e.g. `let c2`)
  
  // Iterate backwards over the specified slice
  let mut gene_index = range.end;
  while gene_index >= range.start {
    match network.genome[gene_index].variant {
      GeneExtras::Input(_) => {
        // If the gene is an input, push its value multiplied by the inputs weight onto
        // the stack
        let (input_weighting, input_id, _) = network.genome[gene_index].ref_input().unwrap();
        let input_weighting = numeric_type.naive_conversion(input_weighting);

        let result_id = format_ident!("c{}", latest_output);
        *latest_output += 1;

        // the input ID of `j` corresponds to the `j`th element of input buffer
        computations.push(quote! {
          let #result_id = #input_weighting * inputs[#input_id]; // weight input ##input_id by #input_weighting
        });
        stack.push(result_id);
      },
      GeneExtras::Neuron(_, _, _) => {
        // If the gene is a neuron, pop a number (the neurons input count) of inputs
        // off the stack, and push the transfer function applied to the sum of these
        // inputs multiplied by the neurons weight onto the stack
        let (weight, neuron_id, _current_value, _, inputs) =
          network.genome[gene_index].ref_mut_neuron().unwrap();

        // commit to a result ID
        let result_id = format_ident!("c{}", latest_output);
        *latest_output += 1;

        // sum the most recently visited N inputs
        let inputs = stack
          .pop(*inputs)
          .unwrap_or_else(|| panic!("Corrupt CGE: neuron (ID {}) did not receive enough inputs (expected {}, but only received {})", neuron_id, inputs, stack.data.len()));

        computations.push(quote! {
          let #result_id = #(#inputs)+*;            // sum the inputs for neuron ##neuron_id
          let #result_id = #activation(#result_id); // apply activation function
        });

        if neuron_update {
          if let Some(index) = recurrence_table.get(&neuron_id) {
            // if we're told to update state, and if the neuron is recurrent, update the persistence array (but delay until the end)
            computations_end.push(quote! {
              self.persistence[#index] = #result_id;
            });
          }
        }

        // when j flag is set, do not include weight of last neuron link as jump forward has a different weight
        if !j || gene_index != range.start {
          // otherwise use regular weight of connection in stack
          let weight = numeric_type.naive_conversion(*weight);
          let new_result_id = format_ident!("c{}", latest_output); *latest_output += 1;
          computations.push(quote! {
            let #new_result_id = #result_id * #weight; // apply weighting
          });
          stack.push(new_result_id);
        } else {
          stack.push(result_id);
        }
      },
      GeneExtras::Forward => {
        // This is inefficient because it can run the neuron evaluation code multiple
        // times
        // TODO: Turn current value of neurons into a struct with a flag representing
        // whether the neuron has been evaluated this network evaluation. Reset this
        // flag after every network evaluation.

        // If the gene is a forward jumper, evaluate the subnetwork starting at the
        // neuron with id of the jumper, and push the result multiplied by the jumpers
        // weight onto the stack
        let weight = numeric_type.naive_conversion(network.genome[gene_index].weight);
        let id = network.genome[gene_index].id;
        let subnetwork_range = network
          .get_subnetwork_index(id)
          .unwrap_or_else(|| panic!("Corrupt CGE: forward connection (gene {}) had an invalid neuron ID ({})", gene_index, id));

        // set j flag to true so the neuron does not include it's regular link weight
        // otherwise the values will be off by whatever factor the neuron weight is
        evaluate(
          network,
          subnetwork_range,
          false,
          true,
          false,
          computations,
          computations_end,
          latest_output,
          recurrence_table,
          numeric_type,
          activation.clone()
        );

        // the recursive call to evaluate modified latest_output, the result id is latest_output - 1
        // Q: can a subnetwork be emtpy, thereby leaving latest_output unchanged? worrying.
        let subnetwork_result_id = format_ident!("c{}", *latest_output - 1);
        let weighted_result_id = format_ident!("c{}", latest_output);
        *latest_output += 1;
        computations.push(quote! {
          let #weighted_result_id = #subnetwork_result_id * #weight;
        });

        stack.push(weighted_result_id);
      },
      GeneExtras::Recurrent => {
        // If the gene is a recurrent jumper, push the previous value of the neuron
        // with the id of the jumper multiplied by the jumpers weight onto the stack
        let gene = &network.genome[gene_index];
        let persistence_index = recurrence_table
          .get(&gene.id)
          .unwrap_or_else(|| panic!("Corrupt CGE: encountered a recurrent connection (gene {}) with an invalid neuron ID ({})", gene_index, gene.id));

        // assert that `network` agrees that this neuron exists / this ID is valid
        assert!(
          network.get_neuron_index(gene.id).is_some(),
          "Corrupt CGE: encountered a recurrent connection with an invalid neuron ID"
        );

        // this is useless code (`let c137 = self.persistence[2];`)
        // however it lets us keep the same process consistent & LLVM will optimize it out I'm p sure
        let result_id = format_ident!("c{}", latest_output);
        *latest_output += 1;
        let weight = numeric_type.naive_conversion(gene.weight);

        computations.push(quote! {
          let #result_id = #weight * self.persistence[#persistence_index]; // access persistence, apply weighting
        });

        stack.push(result_id);
      },
      GeneExtras::Bias => {
        // If the gene is a bias input, push the bias constant multiplied by the genes
        // weight onto the stack

        let gene = &network.genome[gene_index];
        let bias = numeric_type.naive_conversion(gene.weight);

        // this is more junk rustc will const-propagate / LLVM will optimize (`let c137 = -0.02302234`);
        // NOTE: maybe it could be helpful if we declare like `const c137: #NUMERIC_TYPE = -0.02302234;`?
        //       An immutible literal /has the same properties as a constant/, but idk if rustc treats them the identically or not.
        let result_id = format_ident!("c{}", latest_output);
        *latest_output += 1;
        computations.push(quote! {
          let #result_id = #bias; // apply bias
        });
        stack.push(result_id);
      }
    }

    if gene_index == range.start {
      break;
    }

    gene_index -= 1;
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
