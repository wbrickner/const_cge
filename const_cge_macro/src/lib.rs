extern crate proc_macro; 
use std::io::Read;
use syn::parse_macro_input;
use proc_macro::TokenStream;

mod stack;
mod numeric_type;
mod evaluator;
mod recurrence; 
use recurrence::RecurrencyConstraint;
mod synthesis;
mod macro_core;
mod netcrate_invocation; 
use netcrate_invocation::NetcrateInvocation;
#[macro_use] mod invocation_parser;

/// Adds the required fields and functions for executing a network loaded from a CGE file.
/// - If your network has recurrent     architecture, it only works on unit structs (no fields).
/// - If your network has non-recurrent architecture, it works on any struct or enum.
/// - To control target numeric type (`f32`/`f64`), use the `numeric_type` attribute: `#[network("net.cge", numeric_type = f32)`.
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
  macro_core::core(parse_invocation!(attr, item, RecurrencyConstraint::DontCare))
}

/// Identical to `#[network]`, but prevents compilation if network is non-recurrent.
#[proc_macro_attribute]
pub fn recurrent(attr: TokenStream, item: TokenStream) -> proc_macro::TokenStream {
  macro_core::core(parse_invocation!(attr, item, RecurrencyConstraint::Required))
}

/// Identical to `#[network]`, but prevents compilation if network is recurrent.
#[proc_macro_attribute]
pub fn nonrecurrent(attr: TokenStream, item: TokenStream) -> proc_macro::TokenStream {
  macro_core::core(parse_invocation!(attr, item, RecurrencyConstraint::Forbidden))
}

/// API for `netcrate` authors.
/// 
/// ## Usage
/// ```rust
/// #![cfg_attr(not(feature = "std"), no_std)]
/// const_cge::netcrate!(ocr = "optical_char_recog.cge");
/// ```
/// End users can use your network like:
/// ```rust
/// use const_cge::ocr;
/// #[network(ocr)]
/// struct SomeEndUserStruct;
/// ```
/// 
/// - and choose their own numeric_type, 
/// - and their own floating point backend (`std`, `libm`, `micromath`, etc)
/// 
/// Your job may be finished, go rest.
/// 
/// ## Specifying Path
/// The path you provide is relative to the _crate root_ (e.g. a crate file inside `src` would look like
/// `src/my_net.cge`).
/// 
/// ## Multiple Networks
/// Unfortunately, you cannot use the module system to organize networks for users,
/// they will all be forcibly hoisted to the top level.
/// 
/// ```rust
/// const_cge::netcrate!(ocr          = "ocr.cge");
/// const_cge::netcrate!(denoise      = "denoise.cge");
/// const_cge::netcrate!(cart_pole    = "cart.cge");   
/// const_cge::netcrate!(octopus_legs = "octopus.cge");
/// ```
/// 
/// Now, end users can use your network like:
/// ```rust
/// #[network(network_zoo::ocr, numeric_type = f32)]
/// struct HandwritingOCR;
/// ```
#[proc_macro]
pub fn netcrate(input: TokenStream) -> proc_macro::TokenStream {
  let NetcrateInvocation { name, path } = parse_macro_input!(input as NetcrateInvocation);

  // convert to absolute path based on the currently-building crate
  let manifest_path = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to discover crate manifest directory!");
  let cge_path = std::path::Path::new(&manifest_path).join(path);

  // extract the CGE data
  let cge_data = {
    let mut file = std::fs::File::open(cge_path).expect("Failed to open CGE file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read CGE file");
    contents
  };

  quote::quote! {
    #[allow(unused_macros)] // silence warning author will see about some invisible macro
    #[macro_export]         // force this macro to be hoisted and made public
    macro_rules! #name {    // name the macro according to user
      (
        $invocation: ident,
        $item: item,
        $($rest:stmt),*
      ) => {
        #[const_cge::$invocation(#cge_data, $($rest),*)]
        $item
      }
    }
  }.into()
}