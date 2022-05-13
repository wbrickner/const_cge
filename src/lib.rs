#![doc = include_str!("../README.md")] // mirror the README verbatim
pub use const_cge_activations::*;      // make `activations` available to generated functions
pub use const_cge_macro::*;            // make the macros available to users

#[cfg(test)] mod tests;                // property test static evals against dynamic evals
