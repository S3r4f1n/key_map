use crate::{evaluation_tree::{Mode}, Function};

pub fn function_names<F: Function>() -> Vec<F> {
  vec![
    F::from("function_one"),
    F::from("function_two"),
    F::from("funky"),]
}

pub fn mode() -> Mode {
  Mode::from("Normal")
}

