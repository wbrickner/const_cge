mod netcrate_test {
  use crate as const_cge;
  use const_cge::*;

  mod netcrate_example {
    use crate as const_cge;
    const_cge::netcrate!(testnet = "./test_inputs/test_network_v1.cge");
  }

  mod netcrate_example_2 {
    use crate as const_cge;
    const_cge::netcrate!(testnet2 = "./test_inputs/test_network_v1.cge");
  }

  #[test]
  fn recurrency() {
    #[recurrent(testnet)]
    struct MyNet {}
  }

  #[test] // does indeed fail
  fn nonrecurrency() {
    // #[nonrecurrent(testnet)]
    // struct MyNet {}
  }
}

/// Custom test to check that basic recurrency is functioning
mod custom_recurrency_tests {
  use crate as const_cge;
  use const_cge::*;

  /// Test that a case of true recurrence is handled correctly
  #[test]
  fn recurrent_previous_value() {
    // statically create network (enforce recurrency, that is central to what we're testing!)
    #[recurrent("./test_inputs/test_network_recurrent.cge", numeric_type = f64)]
    struct TestNet;
    let mut net = TestNet::default();

    for (input, correct) in [([], [1.0]), ([], [4.0])] {
      let mut static_outputs = [0.0; 1];
      net.evaluate(&input, &mut static_outputs);
      assert_eq!(static_outputs, correct);
    }
  }
}

/// Test `./test_inputs/test_network_v1.cge`
mod test_network_v1 {
  use crate as const_cge; 
  use const_cge::*;
  use cge::*;
  use proptest::{
    prelude::*,
    collection::vec,
    array::uniform2
  };

  /// Static and dynamic constructions of the network should
  /// have identical output for all inputs.
  /// 
  /// - Network memory is wiped after every test.
  #[test]
  fn recurrent_memorywipe_50k_trials() {
    // statically create network
    #[network("./test_inputs/test_network_v1.cge", numeric_type = f64)]
    struct TestNet;

    // dynamically load the exact same network
    let (mut runtime, _, _) = Network::load_file::<(), _>("./test_inputs/test_network_v1.cge", WithRecurrentState(false))
      .expect("Failed to dynamically load CGE file");
    runtime.clear_state();

    // gimme 50,000 `[f64; 2]`; where each f64 falls in [-1, +1]
    proptest!(ProptestConfig::with_cases(50_000), |(input_vector in uniform2(-1.0f64..1.0f64))| {
      let mut net = TestNet::default();
      let mut runtime = runtime.clone();

      let static_outputs = {
        let mut outputs = [0.0; 1];
        net.evaluate(&input_vector, &mut outputs);
        outputs.to_vec()
      };

      let runtime_outputs = runtime.evaluate(&input_vector[..]).unwrap();

      assert_eq!(static_outputs, runtime_outputs);
    });
  }

  /// - Static and dynamic constructions of the network should
  ///   have identical output for all inputs, with memory, out to 5000 eval cycles.
  /// - Repeat this trial 50 times
  #[test]
  fn recurrent_5k_cycles_1k_trials() {
    // statically create network (enforce recurrency, that is central to what we're testing!)
    #[recurrent("./test_inputs/test_network_v1.cge", numeric_type = f64)]
    struct TestNet;

    // dynamically load the exact same network
    let (mut runtime, _, _) = Network::load_file::<(), _>("./test_inputs/test_network_v1.cge", WithRecurrentState(false))
      .expect("Failed to dynamically load CGE file");
    runtime.clear_state();

    // gimme 50 `vec<[f64; 2]>`, each 5k elements long, where each f64 falls in [-1, +1]
    proptest!(ProptestConfig::with_cases(50), |(input_vectors in vec(uniform2(-1.0f64..1.0f64), 5000..=5000))| {
      let mut net = TestNet::default();  // start them both in the same state
      let mut runtime = runtime.clone(); // start them both in the same state

      // run 1000 evals deep
      for input_vector in input_vectors {
        let static_outputs = {
          let mut outputs = [0.0; 1];
          net.evaluate(&input_vector, &mut outputs);
          outputs.to_vec()
        };

        let runtime_outputs = runtime.evaluate(&input_vector[..]).unwrap();
        assert_eq!(static_outputs, runtime_outputs);
      }
    });
  }
}

/// Test `./test_inputs/fig_5_3_paper.cge` 
/// Network from [Figure 5.3](https://www.semanticscholar.org/paper/Towards-a-unified-approach-to-learning-and-Kassahun/f0a39d0e8e891cb9ff6a81172f0c5ebb37ea52e9/figure/30) of [the paper](https://www.semanticscholar.org/paper/Towards-a-unified-approach-to-learning-and-Kassahun/f0a39d0e8e891cb9ff6a81172f0c5ebb37ea52e9)
mod with_extra_data_v1 {
  use crate as const_cge; 
  use const_cge::*;
  use cge::*;
  use proptest::{
    prelude::*,
    collection::vec,
    array::uniform2
  };

  /// Static and dynamic constructions of the network should
  /// have identical output for all inputs.
  /// 
  /// - Network memory is wiped after every test.
  #[test]
  fn recurrent_memorywipe_50k_trials() {
    // statically create network
    #[network("./test_inputs/with_extra_data_v1.cge", numeric_type = f64)]
    struct FigureNet;

    // dynamically load the exact same network
    let (mut runtime, _, _) = Network::load_file::<(), _>("./test_inputs/with_extra_data_v1.cge", WithRecurrentState(false))
      .expect("Failed to dynamically load CGE file");
    runtime.clear_state();

    // gimme 50,000 `[f64; 2]`; where each f64 falls in [-1, +1]
    proptest!(ProptestConfig::with_cases(50_000), |(input_vector in uniform2(-1.0f64..1.0f64))| {
      let mut net = FigureNet::default();
      let mut runtime = runtime.clone();

      let static_outputs = {
        let mut outputs = [0.0; FigureNet::OUTPUT_COUNT];
        net.evaluate(&input_vector, &mut outputs);
        outputs.to_vec()
      };

      let runtime_outputs = runtime.evaluate(&input_vector[..]).unwrap();
      assert_eq!(static_outputs, runtime_outputs);
    });
  }

  /// - Static and dynamic constructions of the network should
  ///   have identical output for all inputs, with memory, out to 5000 eval cycles.
  /// - Repeat this trial 50 times
  #[test]
  fn recurrent_5k_cycles_1k_trials() {
    // statically create network (enforce recurrency, that is central to what we're testing!)
    #[recurrent("./test_inputs/with_extra_data_v1.cge", numeric_type = f64)]
    struct TestNet;

    // dynamically load the exact same network
    let (mut runtime, _, _) = Network::load_file::<(), _>("./test_inputs/with_extra_data_v1.cge", WithRecurrentState(false))
      .expect("Failed to dynamically load CGE file");
    runtime.clear_state();

    // gimme 50 `vec<[f64; 2]>`, each 5k elements long, where each f64 falls in [-1, +1]
    proptest!(ProptestConfig::with_cases(50), |(input_vectors in vec(uniform2(-1.0f64..1.0f64), 5000..=5000))| {
      let mut net = TestNet::default();  // start them both in the same state
      let mut runtime = runtime.clone(); // start them both in the same state

      // run 1000 evals deep
      for input_vector in input_vectors {
        let static_outputs = {
          let mut outputs = [0.0; 1];
          net.evaluate(&input_vector, &mut outputs);
          outputs.to_vec()
        };

        let runtime_outputs = runtime.evaluate(&input_vector[..]).unwrap();
        assert_eq!(static_outputs, runtime_outputs);
      }
    });
  }
}

// /// Test `./test_inputs/fig_5_3_paper_plus_one.cge` 
// /// Augmentation of `figure_5_3_paper` with an single additional neuron.
// mod figure_5_3_paper_plus_one {
//   use crate as const_cge; 
//   use const_cge::*;
//   use cge::*;
//   use proptest::{
//     prelude::*,
//     collection::vec,
//     array::uniform2
//   };

//   /// Static and dynamic constructions of the network should
//   /// have identical output for all inputs.
//   /// 
//   /// - Network memory is wiped after every test.
//   #[test]
//   fn recurrent_memorywipe_50k_trials() {
//     // statically create network
//     #[recurrent("./test_inputs/fig_5_3_paper_plus_one.cge", numeric_type = f64)]
//     struct TestNet;

//     // dynamically load the exact same network
//     let (runtime, _, _) = Network::load_file::<(), _>("./test_inputs/fig_5_3_paper_plus_one.cge", WithRecurrentState(false))
//       .expect("Failed to dynamically load CGE file");

//     // gimme 50,000 `[f64; 2]`; where each f64 falls in [-1, +1]
//     proptest!(ProptestConfig::with_cases(50_000), |(input_vector in uniform2(-1.0f64..1.0f64))| {
//       let mut net = TestNet::default();
//       let mut runtime = runtime.clone();

//       let static_outputs = {
//         let mut outputs = [0.0; 1];
//         net.evaluate(&input_vector, &mut outputs);
//         outputs.to_vec()
//       };

//       let runtime_outputs = runtime.evaluate(&input_vector[..]).unwrap();
//       assert_eq!(static_outputs, runtime_outputs);
//     });
//   }

//   /// - Static and dynamic constructions of the network should
//   ///   have identical output for all inputs, with memory, out to 5000 eval cycles.
//   /// - Repeat this trial 50 times
//   #[test]
//   fn recurrent_5k_cycles_1k_trials() {
//     // statically create network (enforce recurrency, that is central to what we're testing!)
//     #[recurrent("./test_inputs/fig_5_3_paper_plus_one.cge", numeric_type = f64)]
//     struct TestNet;

//     // dynamically load the exact same network
//     let (runtime, _, _) = Network::load_file::<(), _>("./test_inputs/fig_5_3_paper_plus_one.cge", WithRecurrentState(false))
//       .expect("Failed to dynamically load CGE file");

//     // gimme 50 `vec<[f64; 2]>`, each 5k elements long, where each f64 falls in [-1, +1]
//     proptest!(ProptestConfig::with_cases(50), |(input_vectors in vec(uniform2(-1.0f64..1.0f64), 5000..=5000))| {
//       let mut net = TestNet::default();  // start them both in the same state
//       let mut runtime = runtime.clone(); // start them both in the same state

//       // run 1000 evals deep
//       for input_vector in input_vectors {
//         let static_outputs = {
//           let mut outputs = [0.0; 1];
//           net.evaluate(&input_vector, &mut outputs);
//           outputs.to_vec()
//         };

//         let runtime_outputs = runtime.evaluate(&input_vector[..]).unwrap();
//         assert_eq!(static_outputs, runtime_outputs);
//       }
//     });
//   }
// }
