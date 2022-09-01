use std::fs::{write, File, self, OpenOptions};
use std::path::Path;
use std::process::{Command, Stdio};

use serde::Deserialize;
use crate::config;

const DIRPREFIX: &str = "./tmp";


#[derive(Debug, Deserialize)]
pub struct JobInfo {
    source_code: String,
    language: String,
    user_id: u32,
    contest_id: u32,
    problem_id: u32
}

pub struct Job {
    jonid: u32,
    info: JobInfo
}

impl Job {
    pub fn run(&self, config: &config::Config) {
        self.init();
        self.compile_source_code(config);
        let problem = config.problems.iter().find(
            |item| { item.id==self.info.problem_id }
            ).unwrap();
        for case in problem.cases.iter() {
            let input = File::open(&case.input_file).unwrap();
            let output = OpenOptions::new().read(true).write(true).create(true)
                .open(self.path("output")).expect("Creat output Failed");
            let process = Command::new(self.path("a.out"))
                .stdin(input)
                .stdout(Stdio::from(output)).spawn().expect("Run cases Failed");
        }
        self.clear();
    }
    pub fn init(&self) {
        let path = format!("{}/job_{}", DIRPREFIX, self.jonid);
        if !Path::new(&path).is_dir() {
            fs::create_dir(&path).expect(
                format!("Creat dir {} failed", &path).as_str()
                );
        }
    }
    pub fn clear(&self) {
        let path = format!("{}/job_{}", DIRPREFIX, self.jonid);
        if Path::new(&path).is_dir() {
            fs::remove_dir_all(&path).unwrap();
        }
    }
    fn path(&self, filename: &str) -> String {
        format!("{}/job_{}/{}", DIRPREFIX, &self.jonid, filename)
    }
    fn compile_source_code(&self, config: &config::Config) {

        let mut language = config.languages.iter().find(
            |item| {item.name==self.info.language}
            ).unwrap().clone();

        language.replace("%OUTPUT%", &self.path("a.out"));
        language.replace("%INPUT%", &self.path(&language.file_name));
        println!("{}", self.path(&language.file_name));
        write(self.path(&language.file_name), &self.info.source_code).unwrap();

        let mut process = Command::new(&language.command[0])
            .args(&language.command[1..])
            .spawn()
            .expect(
                format!( "Job {} spawn failed", self.jonid) .as_str(),
                );
        process.wait().expect(
            format!( "Job {} compile failed", self.jonid) .as_str(),
            );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{fs, io::{self, Write}};
    use serde_json;
    use crate::config::Config;
    #[test]
    fn test_compile() {
        let json = fs::read_to_string("./config.json").unwrap();
        let config: Config = serde_json::from_str(&json).expect("Parse failed");
        let info = JobInfo {
            source_code: "fn main() { println!(\"Hello World!\"); }".to_string(),
            language: "Rust".to_string(),
            user_id: 0,
            contest_id: 0,
            problem_id: 0
        };
        let job = Job {
            jonid: 0,
            info
        };
        job.init();
        job.compile_source_code(&config);
        let output = Command::new("./tmp/job_0/a.out")
            .output().unwrap();

        io::stdout().write_all(&output.stdout).unwrap();
        assert!(output.status.success());
        assert_eq!("Hello World!\n".as_bytes(), output.stdout);
        job.clear();
    }
}

