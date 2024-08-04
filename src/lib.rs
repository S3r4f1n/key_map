//! key maps is a simple lib which enables key mappings. From keys onto functions.  
//!
//! **Features:**
//! - configure keybindings with json files
//! - split json into several files in a folder and sub folders
//! - supports when expressions which "lookup" values in the environment
//! - supports chained key inputs e.g. [\<c-k\>, \<c-c\>]  
//!
//! **Design:**  
//! - Types are kept as traits to allow for loose coupling. E.g. the keys have to implement the Key trait. Which mainly consists of a conversion from string to key and hashing.
//! - Only one struct is part of the API - KeyParser. which ships with a default. Ready to use.
//! - Traits have default implementations.
//! - KeyParser.env contains all environment related functions. Setting valid functions, mode and EnvVariables.
//! - evaluate key sequences directly or key_by_key
//! - the KeyParser has to be initialized with a json path.  
//! 
//! **Json Structure, Mapping Philosophy:**
//! The central idea is to map keys combinations onto functions. Key combinations are pressed by the use and functions are provided by the software. This lib maps one onto the other.  
//! Between the two, commands are used as an abstraction for the mappings.  
//! - **A Commands** contains a when expression and a list of commands and functions. When a command is called the when expression is evaluated. If true the list of commands and functions is "called". The result of such a call will be a list of functions which should be executed in sequence.
//! - **A KeyMap** contains a list of Keys, a Mode, and single command. Most software won't make use of the Mode, but this lib was designed keeping modal editors in mind. The list of keys translates to the sequence of keys which need to be pressed to execute the command. Further only **One** command can be called by a key map. If several commands should be called by a key map, use a command as abstraction to call them all. This is deliberately chosen to facilitate rebinding of keys. E.g. mapping tab onto to will be easy since tab only calls one command.
//!
//! **Remark** this is still some work in progress bugs are likely to appear.  
//! **Usage**
//! ```
//! use logical_expr::Context; // external crate for expression parsing.
//! use key_maps::{KeyParser, environment::{EnvFunctions, EnvMode, EnvVariables}};
//!
//! let mut key_parser = KeyParser::default();
//! key_parser.set_path("path/to/json/folder");
//! key_parser.env.set_functions(vec![Function::from("function_one"), Function::from("function_two")]); //list of all supported functions
//! key_parser.env.set_mode(Mode::from("Normal")); // default mode
//! key_parser.env.set_variables(Context::new());  // used for when expressions.
//! key_parser.init(); // parsing the json
//!
//! let function_list = key_parser.parse_key_sequences(vec!["<c-k>", "<c-c>"]).unwrap(); // parsing key sequences
//! ```
////////////////////////////////////////////////////////////////////////////////////////////////////

mod evaluation_tree;
mod json_parser;
pub mod environment;
pub mod types;

use std::path::Path;

use json_parser::key_map_data_from_path;
use evaluation_tree::{EvaluationTree};
use evaluation_tree::from_key_map_data::try_into_evaluation_tree;
use types::{FunctionString, KeyCode, Mode};
use environment::{DefaultEnvironment, EnvFunctions, EnvMode, EnvVariables, Environment};
pub trait Key: From<&'static str> + From<String> + Clone + std::fmt::Debug + std::hash::Hash + Eq { }
pub trait Function: From<&'static str> + From<String> + Eq + std::fmt::Display{ }
pub struct KeyParser<M: Key, K: Key, F: Function, E: Environment<M, F>> {
    json_path: String,
    evaluation_tree: Option<EvaluationTree<M, K, F>>,
    pub env: E
}

impl<K: Key> Default for KeyParser<Mode, K, FunctionString, DefaultEnvironment> {
    fn default() -> Self {
        Self::new("key_maps".to_string(), DefaultEnvironment::new())
    }
}

impl<M: Key, K: Key, F: Function, E: Environment<M, F>> KeyParser<M, K, F, E> {
    pub fn new(json_path: String, environment: E) -> Self {
        Self { json_path, evaluation_tree: None , env: environment }
    }
    pub fn set_path(&mut self, json_path: String) {
        self.json_path = json_path
    }
    pub fn get_path(&mut self) -> String {
        self.json_path.clone()
    }
    pub fn init(&mut self) -> Result<(), String>{
        self.evaluation_tree = Some(
            try_into_evaluation_tree::<M, K, F, E>(
                key_map_data_from_path(Path::new(&self.json_path)),
                &self.env)?);
        Ok(())
    }

    pub fn parse_key_sequence(&self, keys: &[K]) -> Result<Vec<&F>, String> {
        if let Some(et) = &self.evaluation_tree {
            et.evaluate(keys, &self.env)
        } else {
            Err("no evaluation tree. KeyParser is not initialized. Potentially due to some flawed key map json".to_string())
        }
    }
    pub fn key_by_key(&mut self, key: K) -> Result<Option<Vec<&F>>, String> {
        if let Some(et) = &mut self.evaluation_tree {
            et.enter_key(&key, &self.env)
        } else {
            Err("no evaluation tree. KeyParser is not initialized. Potentially due to some flawed key map json".to_string())
        }
    }
    pub fn key_by_key_has_next(&mut self, key: K) -> bool {
        if let Some(et) = &mut self.evaluation_tree {
            et.has_next(&key, &self.env)
        } else {
            false
        }
    }
    pub fn key_by_key_enter(&mut self) -> Result<Option<Vec<&F>>, String> {
        if let Some(et) = &mut self.evaluation_tree {
            et.enter_key_terminate(&self.env)
        } else {
            Err("no evaluation tree. KeyParser is not initialized. Potentially due to some flawed key map json".to_string())
        }
    }
}


#[cfg(test)]
mod tests {
    use environment::{EnvFunctions, EnvMode};

    use super::*;

    #[test]
    fn integration_test_pares_sequence() {
        let mut kp = KeyParser::default();
        kp.env.set_functions(vec![FunctionString::from("function_one"), FunctionString::from("function_two"), FunctionString::from("funky")]);
        kp.init().unwrap();
        let functions: Vec<&FunctionString> = kp.parse_key_sequence(&[KeyCode::from("a"), KeyCode::from("b")]).unwrap();
        println!("{functions:?}");
        let functions: Vec<&FunctionString> = kp.parse_key_sequence(&[KeyCode::from("c")]).unwrap();
        // println!("{:?}", kp.evaluation_tree);
        println!("{functions:?}");
    }

    #[test]
    fn integration_enter_key_test() {
        let mut kp = KeyParser::default();
        kp.env.set_functions(vec![FunctionString::from("function_one"), FunctionString::from("function_two"), FunctionString::from("funky")]);
        kp.init().unwrap();
        let functions: Option<Vec<&FunctionString>> = kp.key_by_key(KeyCode::from("a")).unwrap();
        println!("{functions:?}");
        let functions: Option<Vec<&FunctionString>> = kp.key_by_key(KeyCode::from("b")).unwrap();
        println!("{functions:?}");
        let functions: Option<Vec<&FunctionString>> = kp.key_by_key(KeyCode::from("c")).unwrap();
        println!("{functions:?}");
        let functions: Option<Vec<&FunctionString>> = kp.key_by_key(KeyCode::from("a")).unwrap();
        println!("{functions:?}");
        let functions: Option<Vec<&FunctionString>> = kp.key_by_key_enter().unwrap();
        println!("{functions:?}");
        kp.env.set_mode(Mode::from("Insert"));
        if let Err(msg) = kp.key_by_key(KeyCode::from("a")){
            assert_eq!(msg, "No keybindings for mode: Mode(\"Insert\")".to_string());
        } else{
            assert!(false)
        }
        
    }
}
