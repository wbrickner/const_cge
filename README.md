# `const_cge`: Neural Network Compiler

![Cover graphic for the ecosystem of crates including EANT2 and const_cge](https://github.com/wbrickner/const_cge/raw/main/images/ecosystem.png)

# What do?

![Illustration depicts transformation of a neural network into a rust function](https://github.com/wbrickner/const_cge/raw/main/images/transform.png)

`const_cge` performs a symbolic evaluation of your neural network at compile time, producing efficient rust code with identical behavior.

With the information about data dependencies inside the network available, 
LLVM is able to perform more advanced optimizations, like instruction ellision, 
pipeline-aware reordering, SIMD vectorization, register + stack size minimization, and more.

The generated rust code: 
- will never allocate, panic, or rely on `std` (unless using `std` feature!)
- has perfect determinism
- has input and output dimensions which are statically declared
- has internal data dependencies that are statically analyzable
- utilizes an exactly minimal recurrent state array, or none at all (only pay for what you use)
- statically captures properties of your neural network in the type system
- incurs zero overhead cost at runtime

Check out [`eant2`](https://github.com/pengowen123/eant2) to see how to train a neural network compatible with `const_cge`.

```toml
const_cge = "0.2"
```

### Floating Point in `#![no_std]`-land

You can pick a floating point implementation through features: `libm` (default), `std`, or `micromath`, like:

```toml
const_cge = "0.2" # use libm
const_cge = { version = "0.2", default-features = false, features = ["std"] } # `no_std` incompatible
const_cge = { version = "0.2", default-features = false, features = ["micromath"] } # use micromath
```

# Simple Example

## Network
The `network` macro generates all of the fields and functions required to evaluate our network.

```rust
/// Use sensor data to control the limbs of a robot (using f32 only).
#[network("nets/walk.cge", numeric_type = f32)]
struct Walk;

let mut walk = Walk::default();

// I/O is statically sized to match our network
walk.evaluate(&input, &mut output);
```

# Compile Time Guarantees
## Nonrecurrent
It is sometimes a problem if a network can squirel away information about its past states (recurrency).

You can use `nonrecurrent`, which will halt compilation if the imported network contains any recurrency:

```rust
/// Predict which lighting color would best 
/// complement the current sunlight color
#[nonrecurrent("nets/color.cge")]
struct Color;

// evaluate is now a static function.
// it has no state, and this is captured in the type system.
Color::evaluate(&input, &mut output);
```

## Recurrent
Some tasks are best solved using recurrent architectures, and the inclusion of a non-recurrent network would be a mistake.

You can use `recurrent`, which will halt compilation if the imported network contains no recurrency:

```rust
/// Detect if our device has just been dropped 
/// and is currently falling through the air
#[recurrent("nets/drop.cge")]
struct Dropped;

let mut d = Dropped::default();
d.evaluate(&input, &mut output);
```

# Recurrent State?

Recurrent state stores the previous value of a neuron for use in the next evaluation (sent backwards in the network).

The state inside a recurrent network is represented as either `[f64; N]` (or `[f32; N]`), and is updated on every evaluation. As mentioned before, it is made only as large as it needs to be.

If you like, you can read this state, modify it, restore it later, etc.

```rust
/// Attempt to clarify audio stream
#[recurrent("nets/denoise.cge")]
struct Denoise;

// I want a specific recurrent state, 
// not the `::default()` initially-zero recurrent state.
let mut d = Denoise::with_recurrent_state(&saved_state);

// Some evaluations later, read internal state
let state = d.recurrent_state();

// Or modify internal state
do_something_to_state(d.recurrent_state_mut());

// Or set custom state after construction
d.set_recurrent_state(&saved_state);
```

# `numeric_type`

- You often don't need the precision of `f64`, and `f64` is in general larger and slower than `f32`. Using `f64` will behave __identically__ to your CGE file, and so it is the default behavior.
- You can perform (lossy) parameter 'downcasting' on your network, causing all parameters and operations to use your requested type: 

```rust
#[network("net.cge", numeric_type = f32)]
struct SmallerFaster;
```

- Only `f64` and `f32` are supported for now. Maybe I will add support for `f16` / integer / fixed-precision in the future.

# Netcrates!

## What is a netcrate?

- `const_cge` netcrates are pre-trained neural networks _as crates_!

- `const_cge` functions as a common format, allowing the community to share
neural networks for common tasks.

Let's see how you'd use one!

```rust
use netcrate_ocr::ocr;
#[network(ocr)]
struct HandwritingOCR;
```

## Publishing a netcrate

In your `Cargo.toml` file, 
- make sure to **disable `default-features`** for `const_cge`, 
- and **add an `std`** feature:
```toml
[dependencies]
const_cge = { version = "0.2", default-features = false } # <== important!

[features]
std = [] # <== important!
```

In your `stc/lib.rs` file,
- make sure to **conditionally enable `no_std`**
```rust
#![cfg_attr(not(feature = "std"), no_std)]  // <== important!
const_cge::netcrate!(ocr_english  = "nets/ocr/en.cge");
const_cge::netcrate!(ocr_japanese = "nets/ocr/jp.cge");
```

Done!

### Extensions

If you'd like to provide a nicer interface that wraps your network,
please write a macro which provides the implementation, like so:

```rust
#[macro_export]
macro_rules! ocr_ext {
  ($name: ident, $numeric_type: ty) => {
    impl $name {
      /// Returns the unicode char
      pub fn predict_char(&mut self, image: &ImageBuffer) -> char {
        // access everything a `const_cge` struct normally has:
        let output_dim = $name::OUTPUT_SIZE;
        self.recurrent_state_mut()[0] *= -1.0;

        // even access the particluar activation function implementation the end
        // user has chosen:
        const_cge::activations::$numeric_type::relu(x);
      }
    }

    // or produce a new struct, whatever you think is best.
    struct SmolOCR {
      network: $name,
      extra_memory_bank: [$numeric_type; 6 * $name::OUTPUT_SIZE]
    }

    impl SmolOCR {
      //...
  }
}
```

And an end user can simply:

```rust
use netcrate_ocr::*;
#[network(ocr_japanese, numeric_type = f32)]
struct JapaneseOCR;
ocr_ext!(JapaneseOCR, f32);
```

<details>
  <summary>So, how do "netcrates" <i>really</i> work?</summary>

- This approach is a necessary evil because we must allow users to choose their own numerical backend for `no_std` environments, and the options will evolve.
- There are enormous headaches that come with any approach in which you publish a generated implementation, as it is fixed in time, and generated using the author's `const_cge` version, not the end user's version.
- The solution of course is to not expose a particular generated implementation, but to instead expose the data encoding the network, and let the end user's `const_cge` make all the implementation decisions.
- Macro expansion is a little complicated, not to mention you cannot access the values of constants from inside a proc macro.  So what's the trick?

```text
   ╔═════════════════════════════════════════════════════════════════╗
   ║             Smuggling constant values across crate              ║
   ║               boundaries at macro-expansion time                ║
   ╚═════════════════════════════════════════════════════════════════╝
   ┏━━━━━━━━━┓                                                        
   ┃ ocr_net ┃                                                        
   ┣━━━━━━━━━┻━━━━━━━━━━━━━━━━━━━━━┓                                  
   ┃ ┌───────────────────────────┐ ┃                                  
   ┃ │ netcrate!(z = "path.cge") │ ┃ ──┐  ┌──────────────────────────┐
   ┃ └┬─────────────────────────┬┘ ┃   │  │  proc-macro expands to   │
   ┃ ┌▼─────────────────────────▼┐ ┃   ├──┤  exported macro_rules,   │
   ┃ │ #[macro_export]           │ ┃   │  │ which in turn expands to │
   ┃ │ macro_rules! z {          │ ┃   │  │      smuggled data.      │
┌─ ┃ │  () => "<RAW_DATA>"       │ ┃ ──┘  └──────────────────────────┘
│  ┃ │ }                         │ ┃                                  
│  ┃ └───────────────────────────┘ ┃                                  
│  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛                                  
│  ┏━━━━━━━━━━┓                                                       
│  ┃ end_user ┃                                                       
│  ┣━━━━━━━━━━┻━━━━━━━━━━━━━━━━━━━━┓                                  
│  ┃ ┌───────────────────────────┐ ┃                                  
└▶ ┃ │ #[network(ocr_net::z)]    │ ┃ ──┐  ┌──────────────────────────┐
   ┃ │ struct UserNet;           │ ┃   │  │  proc-macro recognizes   │
   ┃ └┬─────────────────────────┬┘ ┃   ├──┤ macro input, invokes it, │
   ┃ ┌▼─────────────────────────▼┐ ┃   │  │   producing a daughter   │
   ┃ │ #[network(ocr_net::z!())] │ ┃ ──┘  │   `network` invocation   │
   ┃ │ struct UserNet;           │ ┃      └──────────────────────────┘
   ┃ └┬─────────────────────────┬┘ ┃      ┌──────────────────────────┐
   ┃ ┌▼─────────────────────────▼┐ ┃      │    rustc expands the     │
   ┃ │ #[network("<RAW_DATA>")]  │ ┃ ──┬──┤    macro_rules macro,    │
   ┃ │ struct UserNet;           │ ┃   │  │    producing a final     │
   ┃ └┬─────────────────────────┬┘ ┃   │  │ invocation of `network`  │
   ┃ ┌▼─────────────────────────▼┐ ┃   │  └──────────────────────────┘
   ┃ │   [NET IMPLEMENTATION]    │ ┃   │  ┌──────────────────────────┐
   ┃ │                           │ ┃   └──┤ `network` recognizes raw │
   ┃ │  Numeric type, backend,   │ ┃      │ data input, and finally  │
   ┃ │     and optimizations     │ ┃      │  generates the desired   │
   ┃ │    all chosen by user.    │ ┃      │      implementation      │
   ┃ └───────────────────────────┘ ┃      └──────────────────────────┘
   ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛                                  
```
</details>

# Design Goals & Drawbacks

- You can accomplish quite a lot with "small" networks, especially for control tasks. `const_cge` is not intended for use in "deep learning" tasks (language modeling, etc).
- Tradeoffs that enable embedded use cases (robotics, 5¢ microcontrollers)
- Lots of individual `const_cge` networks in the same binary may end up being _larger_ or _slower_ than a runtime evaluation approach.
This will depend on the target machine and the networks you're evaluating. If you really care, measure. This library should cover the common use case perfectly.

# [`MIT License`](https://opensource.org/licenses/MIT)

```text
Copyright © 2022 Will Brickner

Permission is hereby granted, free of charge, to any person obtaining a 
copy of this software and associated documentation files (the "Software"), 
to deal in the Software without restriction, including without limitation 
the rights to use, copy, modify, merge, publish, distribute, sublicense, 
and/or sell copies of the Software, and to permit persons to whom the 
Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in 
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS 
OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, 
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE 
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER 
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING 
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER 
DEALINGS IN THE SOFTWARE.
```