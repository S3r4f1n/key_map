pub(crate) mod command_execution;
pub(crate) mod when_expression;
pub(crate) mod from_key_map_data;

use std::{collections::HashMap, fmt::format};
use command_execution::{Command, CommandName};
use crate::{environment::{self, EnvMode, EnvVariables, Environment}, types::{FunctionString, KeyCode, Mode}, Function, Key};


#[derive(Debug)]
pub struct EvaluationTree<M: Key, K: Key, F: Function> {
  tree: HashMap<M, KeyMapNode<K>>,
  commands: HashMap<String, Command<F>>,
  pressed: Vec<K>,
  current_node: Option<KeyMapNode<K>>,
}

impl<M: Key, K: Key, F: Function> EvaluationTree<M, K, F> {
  fn new() -> Self {
    Self { tree: HashMap::new(), commands: HashMap::new(), pressed: Vec::new(), current_node: None }
  }

  fn add(&mut self, mode: M, key_map_node: KeyMapNode<K>) {
    self.tree.insert(mode, key_map_node);
  }

  pub fn evaluate<E: Environment<M, F>>(&self, keys: &[K], environment: &E) -> Result<Vec<&F>, String>{
    let mode = &environment.get_mode();
    let name = self.tree.get(mode).ok_or(format!("mode should have some keybindings: {mode:?} has none"))?.evaluate(keys)?;
    self.get_functions(name, environment)
  }

  fn get_functions<E: EnvVariables>(&self, name: &str, environment: &E) -> Result<Vec<&F>, String>{
    let command = self.commands.get(name).ok_or(format!("command not found: {name}"))?;
    let functions = command.execute(&self.commands, environment);
    if functions.len() == 0 {
      Err(format!("No functions found for command: {name}"))
    } else {
      Ok(functions)
    }
  }
  
  pub fn has_next<E: EnvMode<M>>(&self, key: &K, environment: &E) -> bool {
    if let Some(ref node) = self.current_node{
      node.get_next(key).is_some()
    } else if let Some(node) = self.tree.get(&environment.get_mode()) {
      node.get_next(key).is_some()
    } else {
      false
    }
  }
  
  pub fn enter_key<E: Environment<M, F>>(&mut self, key: &K, environment: &E) -> Result<Option<Vec<&F>>, String> {
    self.pressed.push(key.to_owned());
    if self.current_node.is_none() {
      self.current_node = self.tree.get(&environment.get_mode()).map(|node| node.to_owned());
    }
    let node = self.current_node.take().ok_or(&format!("No keybindings for mode: {:?}", environment.get_mode()))?;
    if let Some(next) = node.get_next(key) { // next is none unless its set again

      if next.next.is_none() {
        self.pressed = Vec::new();
        return self.get_functions(next.evaluate(&[])?, environment).map(|x| Some(x));
      }
      
      self.current_node = Some(next); // set the next node

    } else {
      let msg = format!("Invalid key combination: {:?}, {:?}", self.pressed, key);
      self.pressed = Vec::new();
      return Err(msg);
    }
    
    Ok(None)
  }
  
  pub(crate) fn enter_key_terminate<E: EnvVariables>(&mut self, environment: &E) -> Result<Option<Vec<&F>>, String> {
        let node = self.current_node.take().ok_or(format!("a node should be selected, probably no key entered"))?;
        self.pressed = Vec::new();
        return self.get_functions(node.evaluate(&[])?, environment).map(|x| Some(x));
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
    Self { next: None, command: None }
  }
  fn add(&mut self, key: K, node: KeyMapNode<K>) {
    self.next.as_mut().unwrap().insert(key, Box::new(node));
  }
  fn evaluate(&self, keys: &[K]) -> Result<&String, String> {
    if keys.len() <= 0 {
      return match self.command {
        Some(ref c) => Ok(c),
        None => Err(format!("no command {keys:?}")),
      }
    }
    if let Some(k) = &self.next {
      println!("keys: {keys:?}");
      return k.get(keys.first().expect(&format!("should never happen: {keys:?} len bigger than 0 but first is none")))
      .ok_or(format!("key {keys:?} does not exist at this position in eval tree"))?
      .evaluate(&keys[1..]);
    }
    Err(format!("no key at position in eval tree: {keys:?}"))
  }
  
  fn get_next(&self, key: &K) -> Option<KeyMapNode<K>> {
        if let Some(ref node) = self.next { 
          node.get(key).map(|n| *n.to_owned()) 
        } else { None } 
    }
    
    fn insert_raw_data(&mut self, raw_keys: &[String], raw_command: &str) {
        if raw_keys.len() <= 0 {
          self.command = Some(raw_command.to_owned());
        } else {
          let next = self.next.take();
          let mut next = match next {
            Some(next) => next,
            None => HashMap::new(),
          };
          next.entry(K::from(raw_keys[0].clone()))
          .and_modify(|node| node.insert_raw_data(&raw_keys[1..], raw_command))
          .or_insert_with(|| {
            let mut n = Box::new(KeyMapNode::new());
            n.insert_raw_data(&raw_keys[1..], raw_command);
            n});
          self.next = Some(next);
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
  let mut tree: EvaluationTree<Mode, KeyCode, FunctionString> = EvaluationTree::new();
  let mut node = KeyMapNode::new(); 
  let command = "command".to_owned();
  node.add(KeyCode::from("a"), command.into());
  tree.add(Mode::from("Normal"), node);
  let c = tree.evaluate(&[KeyCode::from("a"), KeyCode::from("b")], &environment::DefaultEnvironment::new()).unwrap();
  println!("{:?}", c);
}

