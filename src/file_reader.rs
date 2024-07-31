use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};

// this reads the folder structure at root and expects json files containing commands and key maps.
// command names are prefixed by the path_to_folder, the file names are ignored. starting with no prefix in the root folder.
// All json files in the root and sub folders are merged into one key map data struct.
// the json should look like this, where optional fields are set to defaults if missing
// {
//   "commands": [                                      //optional defaults to empty vec
//     {
//       "name": "command_one",
//       "command": ["function_one", "function_only"],
//       "when": "true",                                // optional defaults to "true" which means always
//       "mode": "FunctionSequence"                     // optional defaults to mixed
//     },
//     {
//       "name": "command_two",
//       "command": ["function_one", "command_one"],
//       "when": "true",                                // optional defaults to "true" which means always
//       "mode": "Mixed"                                // optional defaults to mixed
//     }
//   ],
//   "key_maps": [                                      //optional defaults to empty vec
//     {
//       "keys": ["a"],
//       "command": "command_one",
//       "mode": "Command"                              // optional defaults to mixed
//     }
//   ]
//}
pub fn key_map_data_from_path(root: &Path) -> KeyMapData {
  let contents = read_all_json_files(root);
  let mut data = KeyMapData::default();
  for (path, json) in contents {
    let relative_path = get_relative_path(root, &path);
    let name_extend = convert_relative_path_to_string(relative_path);
    let data_to_add = parse_key_map_json(json, &name_extend);
    data.commands.extend(data_to_add.commands);
    data.key_maps.extend(data_to_add.key_maps);
  }
  data
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub(crate) struct KeyMapData {
  #[serde(default)]
  pub(crate) commands: Vec<Command>,
  #[serde(default)]
  pub(crate) key_maps: Vec<KeyMap>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Command {
  pub(crate) name: String,
  pub(crate) commands: Vec<String>,
  #[serde(default)]
  pub(crate) command_type: CommandType,
  #[serde(default = "default_when")]
  pub(crate) when: String,
}

fn default_when() -> String { "true".to_owned() }

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct KeyMap {
  pub(crate) keys: Vec<String>,
  pub(crate) command: String,
  #[serde(default)]
  pub(crate) command_type: KeyMapCommandType,
  #[serde(default = "default_mode")]
  pub(crate) mode: Vec<String>,
}

fn default_mode() -> Vec<String> { vec!["Normal".to_owned()] }

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) enum CommandType {
  CommandGroup, // will execute all commands with satisfied when expression (usually only one)
  FunctionSequence, // executes the function in sequence
  #[default]
  Mixed, // executes functions and commands (if when expression is satisfied) in sequence
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) enum KeyMapCommandType {
  Command,
  Function,
  #[default]
  Mixed
}

//------------------------------------------

fn read_all_json_files(dir: &Path) -> Vec<(Box<Path>, String)> {
    let entries = fs::read_dir(dir).unwrap();
    let mut result: Vec<(Box<Path>, String)> = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            result.extend(read_all_json_files(&path));
        } else {
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    let relative_path = dir;
                    let content = fs::read_to_string(path).unwrap();
                    result.push((relative_path.into(), content));
                }
            }
        }
    }
    result
}

fn get_relative_path<'a>(root: &'a Path, path: &'a Path) -> &'a Path {
    path.strip_prefix(root).unwrap()
}

fn convert_relative_path_to_string(path: &Path) -> String {
    path.to_str().unwrap().to_owned().replace("\\", "_").replace("/", "_")
}

fn parse_key_map_json(json_string: String, name_extend: &str) -> KeyMapData {
  let mut data: KeyMapData = serde_json::from_str(&json_string).unwrap();
  if name_extend != "" {
    data.commands.iter_mut().for_each(|c| c.name = format!("{}_{}", name_extend, c.name));
  }
  data
}



//--------------------------------------

#[test]
fn read_dir() {
  let result = read_all_json_files(Path::new("./src/key_maps"));
  let (path, json) = &result[1];
  println!("{:?}, {}", path, json);
}

#[test]
fn convert() {
    let path = Path::new("sub/map.json");
    let result = convert_relative_path_to_string(path);
    assert_eq!(result, "sub_map.json")
}

#[test]
fn get_relative_path_test() {
    let root = Path::new("./src/key_maps");
    let path = Path::new("./src/key_maps/sub/some");
    let result = get_relative_path(root, path);
    assert_eq!(result, Path::new("sub/some"))
}

#[test]
fn parse_key_map_json_test() {
  let json_string = fs::read_to_string("./src/key_maps/simple.json").unwrap();
  let c: KeyMapData = parse_key_map_json(json_string, "extension");
  println!("{:?}", c);
}

#[test]
fn keymapdata_from_path_test() {
  let data = key_map_data_from_path(Path::new("./src/key_maps"));
  println!("{:?}", data);
}