use std::fmt::Display;

use crate::{Function, Key};

// this is the type used for modes, since it also acts as a Key it has to implement Key
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

// these types are here to help get started, but can be replaced by any type implementing Key/Function
#[derive(PartialEq, Eq, Debug)]
pub struct FunctionString(String);
impl Function for FunctionString {}
impl Display for FunctionString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<& str> for FunctionString {
    fn from(s: & str) -> Self {
        FunctionString(s.to_string())
    }
}
impl From<String> for FunctionString {
    fn from(s: String) -> Self {
        FunctionString(s.to_string())
    }
}
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