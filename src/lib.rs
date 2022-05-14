#![no_std]
#![doc = core::include_str!("../README.md")] // mirror the README verbatim
pub use const_cge_macro::*;            // make the macros available to users & netcrate authors
pub mod activations;                   // expose feature-determined `activations` implementations available to generated functions.

#[cfg(test)] mod tests;                // property test static evals against dynamic evals
