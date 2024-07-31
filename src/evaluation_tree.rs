pub(crate) mod command_execution;
pub(crate) mod when_expression;
pub(crate) mod from_key_map_data;

use std::{collections::HashMap};
use command_execution::{Command, CommandName};
use crate::{Function, Key};

pub type FunctionString = String;
impl Function for FunctionString {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCode(String);
impl Key for KeyCode {}
impl From<& str> for KeyCode {
    fn from(s: & str) -> Self {
        KeyCode(s.to_string())
    }
}
impl From<String> for KeyCode {
    fn from(s: String) -> Self {
        KeyCode(s.to_string())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mode(String);
impl Key for Mode {}
impl From<& str> for Mode {
    fn from(s: & str) -> Self {
        Mode(s.to_string())
    }
}
impl From<String> for Mode {
    fn from(s: String) -> Self {
        Mode(s.to_string())
    }
}

#[derive(Debug)]
pub struct EvaluationTree<K: Key, F: Function> {
  tree: HashMap<Mode, KeyMapNode<K>>,
  commands: HashMap<String, Command<F>>,
}

impl<K: Key, F: Function> EvaluationTree<K, F> {
  fn new() -> Self {
    Self { tree: HashMap::new(), commands: HashMap::new()}
  }

  fn add(&mut self, mode: Mode, key_map_node: KeyMapNode<K>) {
    self.tree.insert(mode, key_map_node);
  }

  pub fn evaluate(&self, mode: &Mode, keys: &[K]) -> Result<Vec<&F>, String>{
    let name = self.tree.get(mode).ok_or(format!("mode should have some keybindings: {mode:?} has none"))?.evaluate(keys)?;
    let command = self.commands.get(name).ok_or(format!("command not found: {name}"))?;
    let functions = command.execute(&self.commands);
    if functions.len() == 0 {
      Err(format!("No functions found for command: {name}"))
    } else {
      Ok(functions)
    }
  }
}



#[derive(Clone, Debug)]
struct KeyMapNode<K: Key> {
  next: Option<HashMap<K, Box<KeyMapNode<K>>>>,
  command: Option<String>,
}

impl<K: Key> KeyMapNode<K> {
  fn new_command(command: String) -> Self {
    Self { next: None, command: Some(command) }
  }
  fn new() -> Self {
    Self { next: Some(HashMap::new()), command: None }
  }
  fn add(&mut self, key: K, node: KeyMapNode<K>) {
    self.next.as_mut().unwrap().insert(key, Box::new(node));
  }
  fn evaluate(&self, keys: &[K]) -> Result<&String, String> {
    if let Some(k) = &self.next {
      return k.get(keys.first().ok_or(format!("keys should not be empty: {keys:?}"))?)
      .ok_or(format!("key {keys:?} does not exist at this position in eval tree"))?
      .evaluate(&keys[1..]);
    }
    match self.command {
      Some(ref c) => Ok(c),
      None => Err(format!("no command {keys:?}")),
    }
  }
}

impl<K: Key> From<CommandName> for KeyMapNode<K> {
  fn from(command: String) -> Self {
    Self::new_command(command)
  }
}


#[test]
fn evaluate_evaluation_tree_test() {
  let mut tree: EvaluationTree<KeyCode, FunctionString> = EvaluationTree::new();
  let mut node = KeyMapNode::new(); 
  let command = "command".to_owned();
  node.add(KeyCode::from("a"), command.into());
  tree.add(Mode::from("Normal"), node);
  let c = tree.evaluate(&Mode::from("Normal"), &[KeyCode::from("a"), KeyCode::from("b")]);
  println!("{:?}", c);
}

