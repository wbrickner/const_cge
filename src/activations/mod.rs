// catch invalid configurations that would only lead to more opaque errors later on.
// I know you aren't supposed to do this type of thing. Unfortunately, my features are an enum.
#[cfg(all(feature = "std",       feature = "micromath"))] compile_error!("`std` feature is enabled with `micromath`. These are mutually exclusive, pick one.");
#[cfg(all(feature = "std",       feature = "libm"))]      compile_error!("`std` feature is enabled with `libm`. These are mutually exclusive, pick one.");
#[cfg(all(feature = "micromath", feature = "libm"))]      compile_error!("`micromath` feature is enabled with `libm`. These are mutually exclusive, pick one.");

// if we have the `std` feature (default absent)
#[cfg(feature = "std")]       mod std_impl;
#[cfg(feature = "std")]       pub use std_impl::*;

// if we have the `libm` feature (default present)
#[cfg(feature = "libm")]      mod libm_impl;
#[cfg(feature = "libm")]      pub use libm_impl::*;
#[cfg(all(
  feature = "libm", 
  feature = "expose"
))] pub use libm;

// if we have the `micromath` feature (default absent)
#[cfg(feature = "micromath")] mod mm_impl;
#[cfg(feature = "micromath")] pub use mm_impl::*;
#[cfg(all(
  feature = "micromath", 
  feature = "expose"
))] pub use micromath;