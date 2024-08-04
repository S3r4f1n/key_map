use logical_expr::{Context, ContextValue};

use crate::{types::{FunctionString, Mode}, Function, Key};

pub trait Environment<M: Key, F: Function>: EnvFunctions<F> + EnvMode<M> + EnvVariables { }
pub trait EnvFunctions<F: Function> {
  // this should return a list of all functions. (some what the API)
  fn is_function(&self, name: &str) -> bool;
  fn set_functions(&mut self, functions: Vec<F>);
  fn get_functions(&self) -> Vec<&F>;
}
pub trait EnvMode<M: Key> {
  // this should return the current mode. If no modes are used just return "Normal"
  fn get_mode(&self) -> M;
  fn set_mode(&mut self, mode: M);
}
pub trait EnvVariables {
  // this should return all environment variables. 
  fn environment_variables(&self) -> &Context;
  fn set_environment_variables(&mut self, context: Context);
  fn set_environment_var(&mut self, name: String, value: ContextValue);
}

pub struct DefaultEnvironment {
  context: Context,
  functions: Vec<FunctionString>,
  mode: Mode,
}
impl DefaultEnvironment {
  pub fn new() -> Self {
    Self {context: Context::new(), functions: Vec::new(), mode: Mode::from("Normal")}
  }
}

impl Environment<Mode, FunctionString> for DefaultEnvironment {}
impl EnvFunctions<FunctionString> for DefaultEnvironment {
  fn is_function(&self, name: &str) -> bool {
    self.functions.iter().any(|x| *x == FunctionString::from(name))
  }
  fn set_functions(&mut self, functions: Vec<FunctionString>) {
    self.functions = functions;
  }
  fn get_functions(&self) -> Vec<&FunctionString> {
    self.functions.iter().collect()
  }
}
impl EnvMode<Mode> for DefaultEnvironment {
  fn get_mode(&self) -> Mode {
    self.mode.clone()
  }
  fn set_mode(&mut self, mode: Mode) {
    self.mode = mode
  }
}
impl EnvVariables for DefaultEnvironment {
  fn environment_variables(&self) -> &Context {
    &self.context
  }
  fn set_environment_var(&mut self, name: String, value: ContextValue) {
    self.context.insert(name, value);
  }
  fn set_environment_variables(&mut self, context: Context) {
    self.context = context
  }
}
