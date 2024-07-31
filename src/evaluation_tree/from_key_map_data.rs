use std::collections::HashMap;

use command_execution::CommandName;

use crate::file_reader::{self, KeyMapData, CommandType};
use super::*;
use super::command_execution::{FunctionOrCommandName};
use super::when_expression::Condition;
use crate::environment::function_names;

pub(crate) fn try_into_evaluation_tree<K: Key, F: Function>(raw: KeyMapData) -> Result<EvaluationTree<K, F>, String> {
  let mut tree = EvaluationTree::new();
  tree.commands = try_into_commands::<F>(raw.commands)?;

  for raw_key_map in raw.key_maps {

    let node = raw_key_map.keys.iter().fold(
      KeyMapNode::new_command(CommandName::from(raw_key_map.command)),
      |key_map_node, raw_key| {
        let mut outer_key_map_node = KeyMapNode::new();
        outer_key_map_node.add(K::from(raw_key.to_owned()), key_map_node);
        outer_key_map_node
      });

    for mode in raw_key_map.mode {
      {
        let mode = Mode::from(mode);
        let key_map_node = node.clone();
        tree.add(mode, key_map_node);
      };
    }

  }

  Ok(tree)
}


fn try_into_commands<F: Function>(raw_commands:Vec<file_reader::Command>) -> Result<HashMap<String, Command<F>>, String> {
    let raw_command_names: Vec<CommandName> = raw_commands.iter().map(|c| CommandName::from(&c.name)).collect();
    let function_names = function_names::<F>();
    let mut commands: HashMap<String, Command<F>> = HashMap::new();

    for raw_command in &raw_commands{
      let (command, errors) = raw_command_to_command::<F>(raw_command, &raw_command_names, &function_names);
      if commands.insert(raw_command.name.to_owned(), command).is_some() {
        return Err(format!("duplicate command name: {}", raw_command.name));
      };
      if errors.len() > 0 {
        return Err(errors.join("\n"));
      }
    }
    Ok(commands)
  }

fn raw_command_to_command<F: Function>(raw_command: &file_reader::Command, raw_command_names: &Vec<CommandName>, function_names: &Vec<F>) -> (Command<F>, Vec<String>) {
  let mut errors = Vec::new();
  match raw_command.command_type {
    CommandType::FunctionSequence => {
      (Command::new(
        raw_command.commands.iter().map(|f| {if !function_names.contains(&F::from(f.to_owned())) {errors.push(format!("function not found: {}", f))};
          FunctionOrCommandName::Function(F::from(f.to_owned()))}).collect(),
        Condition::new(&raw_command.when)),
      errors)
    }
    CommandType::CommandGroup => {
      (Command::new(
        raw_command.commands.iter().map(|c| {if !raw_command_names.contains(c) {errors.push(format!("command not found: {}", c))};
        FunctionOrCommandName::CommandName(CommandName::from(c))}).collect(),
        Condition::new(&raw_command.when)),
      errors)
    },
    CommandType::Mixed => {
      (Command::new(raw_command.commands.iter().map(|c|
        if raw_command_names.contains(c) {
          FunctionOrCommandName::CommandName(CommandName::from(c))
        } else {
          if !function_names.contains(&F::from(c.to_owned())) {errors.push(format!("function not found: {}", c))};
          FunctionOrCommandName::Function(F::from(c.to_owned()))
        }
      ).collect(),
      Condition::new(&raw_command.when)),
      errors)
    },
  }
}