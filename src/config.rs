use std::f32;
use serde::{Serialize, Deserialize};
use serde_json::value::Value;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    server: Server,
    pub problems: Vec<Problem>,
    pub languages: Vec<Language>
}

#[derive(Debug,Serialize, Deserialize, Clone)]
struct Server {
    #[serde(default = "default_address")]
    bind_address: String,
    #[serde(default = "default_port")]
    bind_port: u16
}

fn default_address() -> String { "127.0.0.1".to_string() }

fn default_port() -> u16 { 12345 }

#[derive(Debug,Serialize, Deserialize, Clone)]
pub struct Problem {
    pub id: u32,
    pub name: String,
    #[serde(rename(deserialize = "type"))]
    pub problem_type: String,
    pub misc: Value,
    pub cases: Vec<Case>
}

#[derive(Debug,Serialize, Deserialize, Clone)]
pub struct Case {
    pub score: f32,
    pub input_file: String,
    pub answer_file: String,
    pub time_limit: u32,
    pub memory_limit: u32
}


#[derive(Debug,Serialize, Deserialize, Clone)]
pub struct Language {
    pub name: String,
    pub file_name: String,
    pub command: Vec<String>
}

impl Language {
    pub fn replace(&mut self, before: &str, after: &str) -> bool {
        let pos = self.command.iter_mut().find(|item| {*item==before});
        if let Some(pos) = pos {
            *pos = after.to_string();
            return true;
        }
        return false;
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use super::*;
    use serde_json;
    #[test]
    fn test_config() {
        let json = fs::read_to_string("./config.json").unwrap();
        let config: Config = serde_json::from_str(&json).expect("Parse failed");
        let output = serde_json::to_string_pretty(&config).unwrap();
        println!("{}", output);
    }
    #[test]
    fn test_replace() {
        let json = fs::read_to_string("./config.json").unwrap();
        let config: Config = serde_json::from_str(&json).expect("Parse failed");
        let mut lang = config.languages[0].clone();
        lang.replace("%OUTPUT%", "jobid");
        lang.replace("%INPUT%", "main.rs");
        println!("{:?}", &lang.command);
        assert_eq!(lang.command[4], "jobid");
        assert_eq!(lang.command[5], "main.rs");
    }
}
