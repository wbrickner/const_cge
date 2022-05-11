use cge::{Network, gene::GeneExtras};
use proc_macro2::TokenStream;
use quote::quote;
use crate::{recurrence, computations, activations};

/// - Bundle of rust code to be interpolated into various implementation scenarios (derive, make-struct, etc).
/// - Number of recurrent neural states we must retain (0 implies nonrecurrent architecture)
pub struct Synthesis {
  pub recurrency_count:    usize,
  pub documentation:       TokenStream,
  pub persistence_field:   TokenStream,
  pub persistence_methods: TokenStream,
  pub activation_function: TokenStream,
  pub evaluate_function:   TokenStream,
}

fn load_network(cge_path: &str) -> Network {
  let network = Network::load_from_file(&cge_path);
  match network {
    Ok(n) => n,
    Err(e) => panic!("Failed to open CGE file ({})", e)
  }
}

pub fn synthesize(cge_path: String) -> Synthesis {
  let mut network = load_network(&cge_path);
  
  // literally a list of floating point operations as rust code
  let mut computations_list = vec![];
  let recurrence_table = recurrence::identify_recurrence(&network);

  let size = network.size;
  let activation = network.function.clone();
  let output_count = computations::evaluate(
    &mut network, 
    0..size,
    true, 
    false, 
    true, 
    &mut computations_list, 
    &mut 0, 
    &recurrence_table
  ).expect("Network appears to have no outputs");

  let recurrency_count = recurrence_table.len();
  let input_count = network.genome.iter().filter(|g| matches!(g.variant, GeneExtras::Input(_))).count();

  // gimme an optimized expression for the activation function
  let activation_expression = activations::expression(activation);
  // now, can that expression be const?
  let activation_constness = activations::constness(activation);

  // generate a 'persistence' field and access methods (only if neccessary)
  let (persistence_field, persistence_methods) = {
    if recurrency_count == 0 {
      (quote!(), quote!())
    } else {
      (
        quote!(persistence: [f64; #recurrency_count],),
        quote!(
          /// Create network instance with internal recurrent state.
          /// - Useful for "restoring a snapshot" of the network's recurrent state.
          pub fn with_recurrent_state(persistence: &[f64; #recurrency_count]) -> Self {
            Self { persistence: *persistence }
          }
          /// Overwrite the networks recurrent state with the given one.
          /// - Useful for "restoring a snapshot" of the network's recurrent state (even if you don't know what any part of it really means).
          pub fn set_recurrent_state(&mut self, persistence: &[f64; #recurrency_count]) {
            self.persistence = persistence.clone();
          }

          /// Get a reference to the internal recurrent state.
          pub fn recurrent_state(&self) -> &[f64; #recurrency_count] {
            &self.persistence
          }

          /// Get a mutable reference to the internal recurrent state (for modifications).
          /// - This is advanced usage. Recurrent state will likely be _opaque_ (unclear to you what parts of the state do what - welcome to the party),
          /// but this method is made available for flexibility.
          pub fn recurrent_state_mut(&mut self) -> &mut [f64; #recurrency_count] {
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
    cge_path, 
    recurrency_statement = if recurrency_count == 0 {
"No recurrency detected
  - network is stateless (a ZST)
  - `Self::evaluate` is static.".into()
    } else {
      format!(
"Network is recurrent (stateful)
  - {count} persistent state{plural} (`{size} bytes`)
  - `Self::evaluate` must take `&mut self`",
        count = recurrency_count,
        plural = if recurrency_count == 1 { "" } else { "s" },
        size = recurrency_count * core::mem::size_of::<f64>(),
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
    #activation_constness fn activation(x: f64) -> f64 { #activation_expression }
  };

  let evaluate_function = {
    // should the `evaluate` function get a `&mut self`, or can it be a static function?
    let self_argument = if recurrency_count == 0 { quote!() } else { quote!(&mut self,) };

    quote! {
      /// Evaluate the network for a single input.
      /// 
      /// Properties:
      /// - allocationless, heapless, no-std compatible
      /// - should be near the fundamental speed/size limit given:
      /// -  - what LLVM can _safely_ emit (optimization through elision, reordering, automatic vectorization, etc)
      /// -  - how fast your target hardware can perform sequential floating point operations
      pub fn evaluate(#self_argument inputs: &[f64; #input_count], outputs: &mut [f64; #output_count]) {
        #(
          #computations_list
        )*
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