use cge::{Network, gene::GeneExtras};
use proc_macro2::TokenStream;
use quote::quote;
use crate::{recurrence, evaluator, activations, macro_core::Invocation};

/// - Number of recurrent neural states we must retain (0 implies nonrecurrent architecture)
/// - A bundle of rust code to be interpolated in the final step
pub struct Synthesis {
  pub recurrency_count:    usize,
  pub documentation:       TokenStream,
  pub persistence_field:   TokenStream,
  pub persistence_methods: TokenStream,
  pub activation_function: TokenStream,
  pub evaluate_function:   TokenStream,
}

/// Load network
fn load_network(cge_path: &str) -> Network {
  let network = Network::load_from_file(&cge_path);
  match network {
    Ok(n) => n,
    Err(e) => panic!("Failed to open CGE file ({})", e)
  }
}

/// Load, evaluate, and synthesize an implementation.
pub fn synthesize(invocation: &Invocation) -> Synthesis {
  let mut network = load_network(&invocation.config.cge_path);

  // literally a list of floating point operations as rust code
  let mut computations_list = vec![];
  let recurrence_table = recurrence::identify_recurrence(&network);

  let size = network.size;
  let activation = network.function.clone();
  let output_count = evaluator::evaluate(
    &mut network, 
    0..size,
    true, 
    false, 
    true, 
    &mut computations_list, 
    &mut 0, 
    &recurrence_table,
    invocation.config.numeric_type
  ).expect("Corrupt CGE: network appears to have no outputs");

  let recurrency_count = recurrence_table.len();
  let input_count = network.genome.iter().filter(|g| matches!(g.variant, GeneExtras::Input(_))).count();
  let numeric_token = invocation.config.numeric_type.token();
  let numeric_bytes = invocation.config.numeric_type.size_of();

  // gimme an optimized expression for the activation function
  let activation_expression = activations::expression(activation, invocation.config.numeric_type);
  // now, can that expression be const?
  let activation_constness = activations::constness(activation, invocation.config.numeric_type);

  // generate a 'persistence' field and access methods (only if neccessary)
  let (persistence_field, persistence_methods) = {
    if recurrency_count == 0 {
      (quote!(), quote!())
    } else {
      (
        quote!(persistence: [#numeric_token; #recurrency_count],),
        quote!(
          /// Create network instance with internal recurrent state.
          /// - Useful for "restoring a snapshot" of the network's recurrent state.
          pub fn with_recurrent_state(persistence: &[#numeric_token; #recurrency_count]) -> Self {
            Self { persistence: *persistence }
          }
          /// Overwrite the networks recurrent state with the given one.
          /// - Useful for "restoring a snapshot" of the network's recurrent state (even if you don't know what any part of it really means).
          pub fn set_recurrent_state(&mut self, persistence: &[#numeric_token; #recurrency_count]) {
            self.persistence = persistence.clone();
          }

          /// Get a reference to the internal recurrent state.
          pub fn recurrent_state(&self) -> &[#numeric_token; #recurrency_count] {
            &self.persistence
          }

          /// Get a mutable reference to the internal recurrent state (for modifications).
          /// - This is advanced usage. Recurrent state will likely be _opaque_ (unclear to you what parts of the state do what - welcome to the party),
          /// but this method is made available for flexibility.
          pub fn recurrent_state_mut(&mut self) -> &mut [#numeric_token; #recurrency_count] {
            &mut self.persistence
          }
        )
      )
    }
  };

  // dynamically generate doc comments (with usage examples!) that match _this particlar network_.
  let documentation = {
    let build_info = format!(
"- Compiled from CGE file: `{}`
- {recurrency_statement}",
    invocation.config.cge_path, 
    recurrency_statement = if recurrency_count == 0 {
"No recurrency detected
  - network is stateless (a ZST)
  - `Self::evaluate` is static.".into()
    } else {
      format!(
"Network is recurrent (stateful)
  - {state_count} persistent state{state_plural}: `{byte_count} byte{byte_plural}`
  - `Self::evaluate` must take `&mut self`",
        state_count = recurrency_count,
        state_plural = if recurrency_count == 1 { "" } else { "s" },
        byte_count = recurrency_count * numeric_bytes,
        byte_plural = if recurrency_count * numeric_bytes == 1 { "" } else { "s" },
      )
    }
  );

    let input_declr = format!("let input = [{}];", {
      if input_count <= 4 {
        (0..input_count).map(|_| "0.").collect::<Vec<&str>>().join(", ")
      } else {
        format!("0.; {}", input_count)
      }
    });
    let output_declr = format!("let mut output = [{}];", {
      if output_count <= 4 {
        (0..output_count).map(|_| "0.").collect::<Vec<&str>>().join(", ")
      } else {
        format!("0.; {}", output_count)
      }
    });
    let network_declr = format!(
      "let{mutability} network = Network::default(); // {comment}", 
      mutability = if recurrency_count == 0 { "" } else { " mut" },
      comment = if recurrency_count == 0 { "no recurrency, zero-size type" } else { "recurrent state all zeros" },
    );

    quote! {
      #[doc = #build_info]
      /// 
      /// ### Example usage
      /// ```rust
      #[doc = #input_declr]
      #[doc = #output_declr]
      /// 
      #[doc = #network_declr]
      /// network.evaluate(&input, &mut output);
      /// ```
    }
  };

  let activation_function = quote! {
    // LLVM will inline this function if it thinks its a good idea.
    // - No activation may allocate, no activation may mutate anything else.
    // - These are all pure functions, but formally you're not able to call them `const` if they do any floating point math inside,
    //   because (very annoyingly) it could produce f64::NaN at compile time, the bit pattern of which is different on different machines,
    //   making it a runtime matter.
    #activation_constness fn activation(x: #numeric_token) -> #numeric_token { #activation_expression }
  };

  let evaluate_function = {
    // should the `evaluate` function get a `&mut self`, or can it be a static function?
    let self_argument = if recurrency_count == 0 { quote!() } else { quote!(&mut self,) };
    let numeric_comment = format!("-  - how fast your target hardware can perform numeric (`{}`) operations", numeric_token);

    quote! {
      /// Evaluate the network for a single input vector.
      /// 
      /// Properties:
      /// - allocationless, heapless, no_std compatible
      /// - should be near the fundamental speed/size limit given:
      /// -  - what LLVM can _safely_ emit (optimization through elision, reordering, vectorization, register reuse, etc)
      #[doc = #numeric_comment]
      pub fn evaluate(#self_argument inputs: &[#numeric_token; #input_count], outputs: &mut [#numeric_token; #output_count]) {
        #(#computations_list)*
      }
    }
  };

  Synthesis {
    recurrency_count,
    documentation,
    persistence_field,
    persistence_methods,
    activation_function,
    evaluate_function,
  }
}