extern crate proc_macro; 
use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, parse_quote, Fields};
use super::implementations::{synthesize, Synthesis};

pub fn core(cge_path: String, item: Item, recurrency_constraint: Option<bool>) -> TokenStream {
  let Synthesis {
    recurrency_count,
    documentation,
    persistence_field,
    persistence_methods,
    activation_function,
    evaluate_function
  } = synthesize(cge_path);

  // if the recurrency of the network does not conform to our constraint, panic.
  match recurrency_constraint {
    // no constraint
    None => {},

    // demand recurrency
    Some(true)  => if recurrency_count == 0 { panic!("Network is not recurrent (it was demanded)."); },

    // demand non-recurrency
    Some(false) => if recurrency_count != 0 { panic!("Network is recurrent (it was forbidden)."); }
  }

  // fail for enum and non-unit structs (ONLY IF the network requires a persistence field).
  let name = match item {
    syn::Item::Struct(ref s) => {
      if !matches!(s.fields, Fields::Unit) {
        if recurrency_count != 0 {
          panic!("Your network is recurrent. Only unit structs support recurrency at this time.");
        }
      }

      s.ident.clone()
    },
    syn::Item::Enum(ref e) => {
      if recurrency_count != 0 {
        // these could actually be supported
        //  - if the enum data was always [f64; R],
        //  - or perhaps network eval could be available only on variants with correct data,
        //  - or perhaps each variant could be a distinct network
        //  - or perhaps each variant could be a distinct state of the network
        // I'm bad at macros... not today, rustc.
        panic!("Your network is recurrent. Enums cannot always store recurrent state, so are not supported as targets of recurrent networks (for now).");
      }

      e.ident.clone()
    },
    _ => panic!("Unsupported language construct (`struct` and `enum` only).")
  };

  let item = if let Item::Struct(mut s) = item {
    // we now need to add the recurrent data field
    match &s.fields {
      // unit structs => MyStruct { #p } (if needed)
      Fields::Unit => s.fields = Fields::Named(parse_quote!({ #persistence_field })),
      // do nothing, if it was recurrent and non-unit, we would have panicked above.
      _ => {}
    };

    Item::Struct(s)
  } else {
    item
  };

  quote! {
    #documentation
    #[derive(Clone, Copy, Default)]
    #item

    impl #name {
      #persistence_methods
      #activation_function
      #evaluate_function
    }
  }.into()
}