pub struct Stack<T: Clone> {
  pub data: Vec<T>,
}

impl<T: Clone> Stack<T> {
  pub fn new() -> Self {
    Self { data: Vec::new() }
  }

  pub fn pop(&mut self, count: usize) -> Option<Vec<T>> {
    if count > self.data.len() {
      return None;
    }

    let mut result = Vec::new();

    for _ in 0..count {
      result.push(self.data.pop().unwrap());
    }

    Some(result)
  }

  pub fn push(&mut self, value: T) {
    self.data.push(value);
  }
}
