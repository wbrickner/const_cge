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

/// The type of path we were supplied (e.g. my_netcrate::good_net or "nets/good_net.cge")
pub enum CgeType {
  /// Given a module to a netcrate.
  Module(syn::Path),

  /// Given a path to a CGE file.
  File(String),

  /// Given direct data as a string.
  Direct(String)
}

/// Details about the invocation config of the macro.
pub struct Config {
  /// the network
  pub cge:          CgeType,

  /// The **target** numeric type.
  pub numeric_type: NumericType
}

pub fn core(invocation: Invocation) -> TokenStream {
  if let CgeType::Module(p) = invocation.config.cge {
    let invocation_ident = match invocation.recurrency_constraint {
      RecurrencyConstraint::DontCare  => quote!(network),
      RecurrencyConstraint::Required  => quote!(recurrent),
      RecurrencyConstraint::Forbidden => quote!(nonrecurrent),
    };

    let numeric_token = invocation.config.numeric_type.token();

    let item = invocation.item;
    return quote! {
      // we have been given another macro (the one prepared by `netcrate!`),
      // which then expands to the `#[network("literal_cge_data")]` etc.,
      // which then expands to the actual implementation. convoluted.
      #p!(
        #invocation_ident,
        #item,
        
        // ADD MORE ARGUMENTS HERE IF YOU ADD SUPPORT FOR THEM IN THE MAIN MACRO (network, etc)
        numeric_type = #numeric_token
      );
    }.into()
  }

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
      if recurrency_count != 0 {
        match &s.fields {
          Fields::Named(f) if f.named.len() != 0 => panic!("Your network is recurrent. Only unit structs (no fields) support recurrency at this time."),
          Fields::Unnamed(f) if f.unnamed.len() != 0 => panic!("Your network is recurrent. Only unit structs (no fields) support recurrency at this time."),
          _ => {}
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
    s.fields = Fields::Named(parse_quote!({ #persistence_field }));

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