use proc_macro2::TokenStream;
use quote::quote;

/// Supported numeric types
/// TODO: support f16, u64, u32, u16, u8, i64, i32, i16, i8, etc. with automatic conversion
#[derive(Clone, Copy)]
pub enum NumericType {
  Float64,
  Float32
}

impl NumericType {
  pub const VARIANTS_LIST: &'static str = "f64, f32";

  /// Provides the token of the chosen numeric type, suitable for interpolation
  pub fn token(&self) -> TokenStream {
    match self {
      NumericType::Float64 => quote! { f64 },
      NumericType::Float32 => quote! { f32 },
    }
  }

  /// Provide the byte size of the chosen numeric type
  pub fn size_of(&self) -> usize {
    match self {
      NumericType::Float64 => core::mem::size_of::<f64>(),
      NumericType::Float32 => core::mem::size_of::<f32>(),
    }
  }

  /// Take standard high-precision f64 input and convert it to the desired numeric type
  pub fn naive_conversion(&self, base: f64) -> TokenStream {
    match self {
      NumericType::Float64 => quote! { #base },
      NumericType::Float32 => {
        let converted = base as f32;
        quote! { #converted }
      }
    }
  }
}