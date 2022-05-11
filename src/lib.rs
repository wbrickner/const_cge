pub use const_cge_activations::*;
pub use const_cge_macro::*;

#[cfg(test)]
mod test {
  use crate as const_cge;
  use const_cge::*;

  #[test]
  fn it_works() {
    #[network("./test_inputs/test_net.cge", numeric_type = f32)]
    struct MyNet;

    let net = MyNet::default();
  }
}