/// Custom test to check that basic recurrency is functioning
mod custom_recurrency_tests {
  use crate as const_cge;
  use const_cge::*;

  /// Test that a case of true recurrence is handled correctly
  #[test]
  fn recurrent_previous_value() {
    // statically create network (enforce recurrency, that is central to what we're testing!)
    #[recurrent("./test_inputs/recurrent_previous_value.cge", numeric_type = f64)]
    struct TestNet;
    let mut net = TestNet::default();  // start them both in the same state

    for (input, correct) in [([], [1.0]), ([], [4.0])] {
      let mut static_outputs = [0.0; 1];
      net.evaluate(&input, &mut static_outputs);
      assert_eq!(static_outputs, correct);
    }
  }
}

/// Test `./test_inputs/test_net.cge`
mod test_net {
  use crate as const_cge; 
  use const_cge::*;
  use cge::*;
  use proptest::{
    prelude::*,
    collection::vec,
    array::{uniform5, UniformArrayStrategy}
  };

  /// Static and dynamic constructions of the network should
  /// have identical output for all inputs.
  /// 
  /// - Network memory is wiped after every test.
  #[test]
  fn recurrent_memorywipe_50k_trials() {
    // statically create network
    #[network("./test_inputs/test_net.cge", numeric_type = f64)]
    struct TestNet;

    // dynamically load the exact same network
    let runtime = Network::load_from_file("./test_inputs/test_net.cge")
      .expect("Failed to dynamically load CGE file");

    // gimme 50,000 `[f64; N]`; where each f64 falls in [-1, +1]
    proptest!(ProptestConfig::with_cases(50_000), |(input_vector in UniformArrayStrategy::<_, [f64; 5]>::new(-1.0f64..1.0f64))| {
      let mut net = TestNet::default();
      let mut runtime = runtime.clone();

      let static_outputs = {
        let mut outputs = [0.0; 1];
        net.evaluate(&input_vector, &mut outputs);
        outputs.to_vec()
      };

      let runtime_outputs = runtime.evaluate(&input_vector[..]);
      assert_eq!(static_outputs, runtime_outputs);
    });
  }

  /// - Static and dynamic constructions of the network should
  ///   have identical output for all inputs, with memory, out to 5000 eval cycles.
  /// - Repeat this trial 50 times
  #[test]
  fn recurrent_5k_cycles_1k_trials() {
    // statically create network (enforce recurrency, that is central to what we're testing!)
    #[recurrent("./test_inputs/test_net.cge", numeric_type = f64)]
    struct TestNet;

    // dynamically load the exact same network
    let runtime = Network::load_from_file("./test_inputs/test_net.cge")
      .expect("Failed to dynamically load CGE file");

    // gimme 50 `vec<[f64; 5]>`, each 5k elements long, where each f64 falls in [-1, +1]
    proptest!(ProptestConfig::with_cases(50), |(input_vectors in vec(uniform5(-1.0f64..1.0f64), 5000..=5000))| {
      let mut net = TestNet::default();  // start them both in the same state
      let mut runtime = runtime.clone(); // start them both in the same state

      // run 1000 evals deep
      for input_vector in input_vectors {
        let static_outputs = {
          let mut outputs = [0.0; 1];
          net.evaluate(&input_vector, &mut outputs);
          outputs.to_vec()
        };

        let runtime_outputs = runtime.evaluate(&input_vector[..]);
        assert_eq!(static_outputs, runtime_outputs);
      }
    });
  }
}

/// Test `./test_inputs/fig_5_3_paper.cge` 
/// Network from [Figure 5.3](https://www.semanticscholar.org/paper/Towards-a-unified-approach-to-learning-and-Kassahun/f0a39d0e8e891cb9ff6a81172f0c5ebb37ea52e9/figure/30) of [the paper](https://www.semanticscholar.org/paper/Towards-a-unified-approach-to-learning-and-Kassahun/f0a39d0e8e891cb9ff6a81172f0c5ebb37ea52e9)
mod figure_5_3_paper {
  use crate as const_cge; 
  use const_cge::*;
  use cge::*;
  use proptest::{
    prelude::*,
    collection::vec,
    array::{uniform5, UniformArrayStrategy}
  };

  /// Static and dynamic constructions of the network should
  /// have identical output for all inputs.
  /// 
  /// - Network memory is wiped after every test.
  #[test]
  fn recurrent_memorywipe_50k_trials() {
    // statically create network
    #[network("./test_inputs/fig_5_3_paper.cge", numeric_type = f64)]
    struct TestNet;

    // dynamically load the exact same network
    let runtime = Network::load_from_file("./test_inputs/fig_5_3_paper.cge")
      .expect("Failed to dynamically load CGE file");

    // gimme 50,000 `[f64; N]`; where each f64 falls in [-1, +1]
    proptest!(ProptestConfig::with_cases(50_000), |(input_vector in UniformArrayStrategy::<_, [f64; 5]>::new(-1.0f64..1.0f64))| {
      let mut net = TestNet::default();
      let mut runtime = runtime.clone();

      let static_outputs = {
        let mut outputs = [0.0; 1];
        net.evaluate(&input_vector, &mut outputs);
        outputs.to_vec()
      };

      let runtime_outputs = runtime.evaluate(&input_vector[..]);
      assert_eq!(static_outputs, runtime_outputs);
    });
  }

  /// - Static and dynamic constructions of the network should
  ///   have identical output for all inputs, with memory, out to 5000 eval cycles.
  /// - Repeat this trial 50 times
  #[test]
  fn recurrent_5k_cycles_1k_trials() {
    // statically create network (enforce recurrency, that is central to what we're testing!)
    #[recurrent("./test_inputs/fig_5_3_paper.cge", numeric_type = f64)]
    struct TestNet;

    // dynamically load the exact same network
    let runtime = Network::load_from_file("./test_inputs/fig_5_3_paper.cge")
      .expect("Failed to dynamically load CGE file");

    // gimme 50 `vec<[f64; 5]>`, each 5k elements long, where each f64 falls in [-1, +1]
    proptest!(ProptestConfig::with_cases(50), |(input_vectors in vec(uniform5(-1.0f64..1.0f64), 5000..=5000))| {
      let mut net = TestNet::default();  // start them both in the same state
      let mut runtime = runtime.clone(); // start them both in the same state

      // run 1000 evals deep
      for input_vector in input_vectors {
        let static_outputs = {
          let mut outputs = [0.0; 1];
          net.evaluate(&input_vector, &mut outputs);
          outputs.to_vec()
        };

        let runtime_outputs = runtime.evaluate(&input_vector[..]);
        assert_eq!(static_outputs, runtime_outputs);
      }
    });
  }
}

/// Test `./test_inputs/fig_5_3_paper_plus_one.cge` 
/// Augmentation of `figure_5_3_paper` with an single additional neuron.
mod figure_5_3_paper_plus_one {
  use crate as const_cge; 
  use const_cge::*;
  use cge::*;
  use proptest::{
    prelude::*,
    collection::vec,
    array::{uniform5, UniformArrayStrategy}
  };

  /// Static and dynamic constructions of the network should
  /// have identical output for all inputs.
  /// 
  /// - Network memory is wiped after every test.
  #[test]
  fn recurrent_memorywipe_50k_trials() {
    // statically create network
    #[recurrent("./test_inputs/fig_5_3_paper_plus_one.cge", numeric_type = f64)]
    struct TestNet;

    // dynamically load the exact same network
    let runtime = Network::load_from_file("./test_inputs/fig_5_3_paper_plus_one.cge")
      .expect("Failed to dynamically load CGE file");

    // gimme 50,000 `[f64; N]`; where each f64 falls in [-1, +1]
    proptest!(ProptestConfig::with_cases(50_000), |(input_vector in UniformArrayStrategy::<_, [f64; 5]>::new(-1.0f64..1.0f64))| {
      let mut net = TestNet::default();
      let mut runtime = runtime.clone();

      let static_outputs = {
        let mut outputs = [0.0; 1];
        net.evaluate(&input_vector, &mut outputs);
        outputs.to_vec()
      };

      let runtime_outputs = runtime.evaluate(&input_vector[..]);
      assert_eq!(static_outputs, runtime_outputs);
    });
  }

  /// - Static and dynamic constructions of the network should
  ///   have identical output for all inputs, with memory, out to 5000 eval cycles.
  /// - Repeat this trial 50 times
  #[test]
  fn recurrent_5k_cycles_1k_trials() {
    // statically create network (enforce recurrency, that is central to what we're testing!)
    #[recurrent("./test_inputs/fig_5_3_paper_plus_one.cge", numeric_type = f64)]
    struct TestNet;

    // dynamically load the exact same network
    let runtime = Network::load_from_file("./test_inputs/fig_5_3_paper_plus_one.cge")
      .expect("Failed to dynamically load CGE file");

    // gimme 50 `vec<[f64; 5]>`, each 5k elements long, where each f64 falls in [-1, +1]
    proptest!(ProptestConfig::with_cases(50), |(input_vectors in vec(uniform5(-1.0f64..1.0f64), 5000..=5000))| {
      let mut net = TestNet::default();  // start them both in the same state
      let mut runtime = runtime.clone(); // start them both in the same state

      // run 1000 evals deep
      for input_vector in input_vectors {
        let static_outputs = {
          let mut outputs = [0.0; 1];
          net.evaluate(&input_vector, &mut outputs);
          outputs.to_vec()
        };

        let runtime_outputs = runtime.evaluate(&input_vector[..]);
        assert_eq!(static_outputs, runtime_outputs);
      }
    });
  }
}