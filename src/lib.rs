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
// when expression parsing
// environment variables

mod evaluation_tree;
mod file_reader;
mod environment;

use std::path::Path;

use file_reader::key_map_data_from_path;
use evaluation_tree::{EvaluationTree, Mode, KeyCode, FunctionString};
use evaluation_tree::from_key_map_data::try_into_evaluation_tree;
use environment::mode;

pub trait Key: From<&'static str> + From<String> + Clone + std::fmt::Debug + std::hash::Hash + Eq { }
pub trait Function: From<&'static str> + From<String> + Eq{ }
pub struct KeyParser<K: Key, F: Function> {
    json_path: String,
    evaluation_tree: Option<EvaluationTree<KeyCode, F>>,
    some: (K, F)
    // potential options
}

impl<F: Function> KeyParser<KeyCode, F> {
    pub fn new(json_path: String) -> Self {
        Self { json_path, evaluation_tree: None, some: (KeyCode::from ("some"), F::from ("some")) }
    }
    pub fn init(&mut self) -> Result<(), String>{
        //code got hella ugly but interface should ok.
        self.evaluation_tree = Some(try_into_evaluation_tree::<KeyCode, F>(key_map_data_from_path(Path::new(&self.json_path))).unwrap());
        Ok(())
    }
    pub fn pares_input(&self, keys: &[KeyCode]) -> Result<Vec<&F>, String> {
        if let Some(et) = &self.evaluation_tree {
            et.evaluate(&mode(), keys)
        } else {
            Err("no evaluation tree. KeyParser is not initialized. Potentially due to some flawed key map json".to_string())
        }
    }
}

#[test]
fn integration_test() {
    let mut kp: KeyParser<KeyCode, FunctionString> = KeyParser::new("src/key_maps".to_string());
    kp.init().unwrap();
    let functions: Vec<&FunctionString> = kp.pares_input(&[KeyCode::from("a")]).unwrap();
    // println!("{:?}", kp.evaluation_tree);
    println!("{functions:?}");
}
