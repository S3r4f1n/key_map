
#[derive(Debug)]
pub struct Condition {
  when: String,
}

impl Default for Condition {
  fn default() -> Self {
    Self { when: "true".to_owned() }
  }
}

impl Condition {
  pub fn new(when: &str) -> Self {
    Self { when: when.to_owned() }
  }
  pub fn is_satisfied(&self) -> bool {
    self.when == "true"
  }
}