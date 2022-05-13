## If you want to use [`const_cge`](https://crates.io/crates/const_cge), you may be in the wrong place.

# Shared Activations Crate
This is a support crate for `const_cge`, allowing all generated code to reuse the exact same activation functions.

# Backing Implementation
Because `const_cge` codegen supports `no_std`, we must rely on
replacements for some floating point "*extensions*" (like `.sqrt()`, `.exp()`, etc).

**There are 3 implementations you can pick from, enabled by features.**

### `std`
The `std` feature enables the standard library extensions, and *immediately makes this crate `no_std` incompatible!*

### `libm` – **default**
The `libm` feature enables dependency on the [`libm` crate](https://crates.io/crate/libm).
As I understand, the implementations inside `libm` will either identically or closely match those in the standard library. This is the reason it is the default.

### `micromath`
The `micromath` feature will enable using the [`micromath` crate](https://crates.io/crate/micromath) *when possible*.
  - As I understand, `micromath` approximations are supposed to be less accurate, but faster.
  - However, `micromath` only provides support for `f32`, and not `f64`.
  - This means if you have enabled the `micromath` feature and are using the `f64::*` activation functions, you are actually using `libm` under the hood.
  - Similarly, `micromath` provides no `tanh` approximation, and so the `libm` implementation must be used instead.
