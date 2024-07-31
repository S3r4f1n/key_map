use std::collections::HashMap;

use super::{Function};
use super::when_expression::Condition;

#[derive(Debug, Default)]
pub(crate) struct Command<F: Function> {
  condition: Condition,
  values: Vec<FunctionOrCommandName<F>>,
}

impl<F: Function> Command<F> {
  pub(crate) fn execute<'a>(&'a self, conglomerates: &'a HashMap<CommandName, Command<F>>) -> Vec<&F> {

    if self.condition.is_satisfied() {
      self.values.iter().map(|value| {

        match value {
          FunctionOrCommandName::CommandName(command_name) => {
            let name = command_name.to_string();
            let conglomerate = conglomerates.get(&name)
            .ok_or("command not found. this should never appear, since during parsing such errors are caught").unwrap();
            conglomerate.execute(conglomerates)
          }
          FunctionOrCommandName::Function(function) => vec![function]
        }

      }).flatten().collect()

    } else {
      Vec::new()
    }
  }

  pub(crate) fn new(values: Vec<FunctionOrCommandName<F>>, when: Condition) -> Self {
        Self {
            condition: when,
            values,
        }
    }
}

#[derive(Debug)]
pub(crate) enum FunctionOrCommandName<F: Function> {
  CommandName(CommandName),
  Function(F),
}

pub(crate) type CommandName = String;