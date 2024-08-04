use std::collections::HashMap;

use command_execution::CommandName;
use environment::EnvFunctions;

use crate::environment::Environment;
use crate::json_parser::{self, KeyMapData, CommandType};
use super::*;
use super::command_execution::{FunctionOrCommandName};
use super::when_expression::Condition;

pub(crate) fn try_into_evaluation_tree<M: Key, K: Key, F: Function, E: Environment<M, F>>(raw: KeyMapData, environment: &E) -> Result<EvaluationTree<M, K, F>, String> {
  let mut tree = EvaluationTree::new();
  tree.commands = try_into_commands::<F, E>(raw.commands, &environment)?;

  for raw_key_map in raw.key_maps {
    println!("mode: {raw_key_map:?}");
    for mode in raw_key_map.mode {
      tree.tree.entry(M::from(mode))
      .and_modify(|node| node.insert_raw_data(&raw_key_map.keys, &raw_key_map.command))
      .or_insert({let mut n = KeyMapNode::new();
        n.insert_raw_data(&raw_key_map.keys, &raw_key_map.command);
        n});
    }
  }

  Ok(tree)
}


fn try_into_commands<F: Function, E: EnvFunctions<F>>(raw_commands:Vec<json_parser::Command>, environment: &E) -> Result<HashMap<String, Command<F>>, String> {
    let raw_command_names: Vec<CommandName> = raw_commands.iter().map(|c| CommandName::from(&c.name)).collect();
    let mut commands: HashMap<String, Command<F>> = HashMap::new();

    for raw_command in &raw_commands{
      let (command, errors) = raw_command_to_command::<F, E>(raw_command, &raw_command_names, environment);
      if commands.insert(raw_command.name.to_owned(), command).is_some() {
        return Err(format!("duplicate command name: {}", raw_command.name));
      };
      if errors.len() > 0 {
        return Err(errors.join("\n"));
      }
    }
    Ok(commands)
  }

fn raw_command_to_command<F: Function, E: EnvFunctions<F>>(raw_command: &json_parser::Command, raw_command_names: &Vec<CommandName>, env_functions: &E) -> (Command<F>, Vec<String>) {
  let mut errors = Vec::new();
  match raw_command.command_type {
    CommandType::FunctionSequence => {
      (Command::new(
        raw_command.commands.iter().map(|f| {if !env_functions.is_function(f) {errors.push(format!("function not found: {}", f))};
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
          if !env_functions.is_function(c) {errors.push(format!("function not found: {}", c))};
          FunctionOrCommandName::Function(F::from(c.to_owned()))
        }
      ).collect(),
      Condition::new(&raw_command.when)),
      errors)
    },
  }
}