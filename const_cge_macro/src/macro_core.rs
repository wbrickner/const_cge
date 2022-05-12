extern crate proc_macro; 
use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, parse_quote, Fields};
use crate::{numeric_type::NumericType, recurrence::RecurrencyConstraint};
use super::synthesis::{synthesize, Synthesis};

/// All the invocation information.
pub struct Invocation {
  pub config: Config,

  /// The item on which we are implementing (unit struct, possibly an enum).
  pub item: Item,

  /// Constraint on the recurrency of the network
  pub recurrency_constraint: RecurrencyConstraint
}

/// Details about the invocation config of the macro.
pub struct Config {
  /// path to the network
  pub cge_path:     String,

  /// The **target** numeric type.
  pub numeric_type: NumericType
}

pub fn core(invocation: Invocation) -> TokenStream {
  let Synthesis {
    recurrency_count,
    documentation,
    persistence_field,
    associated_constants,
    persistence_methods,
    evaluate_function
  } = synthesize(&invocation);

  // if the recurrency of the network does not conform to our constraint, panic.
  match invocation.recurrency_constraint {
    // no constraint
    RecurrencyConstraint::DontCare  => {},

    // require recurrency
    RecurrencyConstraint::Required  => if recurrency_count == 0 { panic!("Network is not recurrent (it was demanded)."); },

    // forbid recurrency
    RecurrencyConstraint::Forbidden => if recurrency_count != 0 { panic!("Network is recurrent (it was forbidden)."); }
  }

  // fail for enum and non-unit structs (ONLY IF the network requires a persistence field).
  let name = match invocation.item {
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

  let item = if let Item::Struct(mut s) = invocation.item.clone() {
    // we now need to add the recurrent data field
    match &s.fields {
      // unit structs => MyStruct { #p } (if needed)
      Fields::Unit => s.fields = Fields::Named(parse_quote!({ #persistence_field })),
      // do nothing, if it was recurrent and non-unit, we would have panicked above.
      _ => {}
    };

    Item::Struct(s)
  } else {
    invocation.item
  };

  quote! {
    #documentation
    #[derive(Clone, Copy, Default)]
    #item

    impl #name {
      #associated_constants
      #persistence_methods
      #evaluate_function
    }
  }.into()
}