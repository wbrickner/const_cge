extern crate proc_macro; 
use proc_macro::TokenStream;
mod stack;
mod numeric_type;
mod evaluator;
mod recurrence;
mod activations;
mod synthesis;
mod macro_core;
#[macro_use] mod invocation_parser;

/// Adds the required fields and functions for executing a network loaded from a CGE file.
/// - If your network has recurrent     architecture, it only works on unit structs (no fields).
/// - If your network has non-recurrent architecture, it works on any struct or enum.
/// ```rust
/// use const_cge::network;
/// 
/// /// Controls the robot's limbs, predicting motor actuations
/// /// that will move it in the desired direction.
/// #[network("./walker.cge")]
/// struct Walker;
/// 
/// fn main() {
///   let mut walk = Walker::default();
///   walk.evaluate(&inputs, &mut outputs);
/// }
/// ```
#[proc_macro_attribute]
pub fn network(attr: TokenStream, item: TokenStream) -> proc_macro::TokenStream {
  let invocation = parse_invocation!(attr, item, None);
  macro_core::core(invocation)
}

/// Identical to `#[network("path/to/file.cge")]`, but prevents compilation if network is non-recurrent.
#[proc_macro_attribute]
pub fn recurrent(attr: TokenStream, item: TokenStream) -> proc_macro::TokenStream {
  let invocation = parse_invocation!(attr, item, Some(true));
  // let item = syn::parse_macro_input!(item as syn::Item);
  macro_core::core(invocation)
}

/// Identical to `#[network("path/to/file.cge")]`, but prevents compilation if network is recurrent.
#[proc_macro_attribute]
pub fn nonrecurrent(attr: TokenStream, item: TokenStream) -> proc_macro::TokenStream {
  let invocation = parse_invocation!(attr, item, Some(false));
  macro_core::core(invocation)
}