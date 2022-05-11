extern crate proc_macro; 
use proc_macro::TokenStream;
mod stack;
mod computations;
mod recurrence;
mod activations;
mod implementations;
mod macro_core;
#[macro_use] mod utils;

#[cfg(test)] mod tests;

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
  let cge_path = get_cge_path!(attr, "#[network(\"path/to/cge/file.cge\")]");
  let item = syn::parse_macro_input!(item as syn::Item);
  macro_core::core(cge_path, item, None)
}

/// Identical to `#[network("path/to/file.cge")]`, but prevents compilation if network is non-recurrent.
#[proc_macro_attribute]
pub fn recurrent(attr: TokenStream, item: TokenStream) -> proc_macro::TokenStream {
  let cge_path = get_cge_path!(attr, "#[recurrent(\"path/to/cge/file.cge\")]");
  let item = syn::parse_macro_input!(item as syn::Item);
  macro_core::core(cge_path, item, Some(true))
}

/// Identical to `#[network("path/to/file.cge")]`, but prevents compilation if network is recurrent.
#[proc_macro_attribute]
pub fn nonrecurrent(attr: TokenStream, item: TokenStream) -> proc_macro::TokenStream {
  let cge_path = get_cge_path!(attr, "#[nonrecurrent(\"path/to/cge/file.cge\")]");
  let item = syn::parse_macro_input!(item as syn::Item);
  macro_core::core(cge_path, item, Some(false))
}