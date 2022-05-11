use proc_macro2::TokenStream;
use cge::Activation;
use quote::quote;
use crate::numeric_type::NumericType;

// TODO: revisit bit hacks if it seems worthwhile
pub fn expression(activation: Activation, _numeric_type: NumericType) -> TokenStream {
  match activation {
    Activation::Linear       => quote! { x },
    Activation::Threshold    => quote! { if x > 0.0 { 1.0 } else { 0.0 } },
    Activation::Relu         => quote! { if x > 0.0 { x } else { 0.0 } } ,
    Activation::Sign         => quote! { if x > 0.0 { 1.0 } else if x == 0.0 { 0.0 } else { -1.0 } },
    Activation::Sigmoid      => quote! { 1.0 / (1.0 + (-x).exp()) },
    Activation::Tanh         => quote! { x.tanh() },
    Activation::SoftSign     => quote! { x / (1.0 + x.abs()) },
    Activation::BentIdentity => quote! { (((x.powi(2) + 1.0).sqrt() - 1.0) / 2.0) + x },
  }
}

/// Can the activation function be `const fn`?
/// - `true` if the activation expression contains _no actual floating point math_
/// - `false` otherwise
pub fn constness(activation: Activation, _numeric_type: NumericType) -> TokenStream {
  match activation {
    //
    // (used to be) bit hacks, no floating point math:
    //
    Activation::Linear       => quote! { const },
    Activation::Threshold    => quote! {       },
    Activation::Relu         => quote! {       },
    Activation::Sign         => quote! {       },

    //
    // Contains floating point math:
    //
    Activation::Sigmoid      => quote! {       },
    Activation::Tanh         => quote! {       },
    Activation::SoftSign     => quote! {       },
    Activation::BentIdentity => quote! {       },
  }
}