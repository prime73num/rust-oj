use std::f32;
use serde::{Serialize, Deserialize};
use serde_json::value::Value;


#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    server: Server,
    problems: Vec<Problem>,
    languages: Vec<Language>
}

#[derive(Debug,Serialize, Deserialize)]
struct Server {
    #[serde(default = "default_address")]
    bind_address: String,
    #[serde(default = "default_port")]
    bind_port: u16
}

fn default_address() -> String { "127.0.0.1".to_string() }

fn default_port() -> u16 { 12345 }

#[derive(Debug,Serialize, Deserialize)]
struct Problem {
    id: u32,
    name: String,
    #[serde(rename(deserialize = "type"))]
    problem_type: String,
    misc: Value,
    cases: Vec<Case>
}

#[derive(Debug,Serialize, Deserialize)]
struct Case {
    score: f32,
    input_file: String,
    answer_file: String,
    time_limit: u32,
    memory_limit: u32
}


#[derive(Debug,Serialize, Deserialize)]
struct Language {
    name: String,
    file_name: String,
    command: Vec<String>
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
}
