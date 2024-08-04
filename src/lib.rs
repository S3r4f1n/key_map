// key maps is a simple lib which enables key mappings.
// at the core it maps (chains of) key codes into "command" strings which can be matched onto functions
// these functions are provided
// parse_wait(string) this pareses an input key code 
// enter() this executes the associated command
// modes: manual, enter has to be called to execute the parsing
//        auto(ms), the command is executed directly if single command or after a delay if key code chain is possible.

// the key maps are configured in two stages. There are command groups and commands. And key maps.
// a command consists of a when expression and a command string, command or command group or a list of commands which will be executed in sequence.
// a command group consists of several commands, usually only one command should have a when expression evaluating to true

// each key combination can only be mapped to one command or command group.
// multi mapping is therefore created in two steps create a mapping two a group and add several commands to the group. Enabling easier remapping.

// key maps and commands are stored in json files.


// todo 
// has_next
// has_command
// threading and delay

mod evaluation_tree;
mod json_parser;
mod environment;
mod types;

use std::path::Path;

use json_parser::key_map_data_from_path;
use evaluation_tree::{EvaluationTree};
use evaluation_tree::from_key_map_data::try_into_evaluation_tree;
use types::{FunctionString, KeyCode, Mode};
use environment::{DefaultEnvironment, Environment};

pub trait Key: From<&'static str> + From<String> + Clone + std::fmt::Debug + std::hash::Hash + Eq { }
pub trait Function: From<&'static str> + From<String> + Eq + std::fmt::Display{ }
pub struct KeyParser<M: Key, K: Key, F: Function, E: Environment<M, F>> {
    json_path: String,
    evaluation_tree: Option<EvaluationTree<M, K, F>>,
    pub env: E
}

impl<K: Key> Default for KeyParser<Mode, K, FunctionString, DefaultEnvironment> {
    fn default() -> Self {
        Self::new("src/key_maps".to_string(), DefaultEnvironment::new())
    }
}

impl<M: Key, K: Key, F: Function, E: Environment<M, F>> KeyParser<M, K, F, E> {
    pub fn new(json_path: String, environment: E) -> Self {
        Self { json_path, evaluation_tree: None , env: environment }
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
