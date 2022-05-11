/// Assert that compiles do in fact fail when the provided network does not match recurrency constraints
#[test]
fn recurrency_violations() {
  let t = trybuild::TestCases::new();
  t.compile_fail("tests/recurrency_violations/*.rs");
}

/// Assert that compiles do in fact succeed when used correctly
#[test]
fn correct_usage() {
  let t = trybuild::TestCases::new();
  t.compile_fail("tests/correct_usage/*.rs");
}