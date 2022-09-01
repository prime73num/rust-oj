use std::fs::{write, File, self, OpenOptions};
use std::path::Path;
use std::process::{Command, Stdio};

use serde::Deserialize;
use serde::Serialize;
use crate::config;

use wait_timeout::ChildExt;
use std::time::Duration;

const DIRPREFIX: &str = "./tmp";


#[derive(Debug, Serialize, Deserialize)]
pub struct JobInfo {
    source_code: String,
    language: String,
    user_id: u32,
    contest_id: u32,
    problem_id: u32
}

pub struct Job {
    job_id: u32,
    info: JobInfo,
    score: f32
}

impl Job {
    pub fn new(job_id: u32, info: JobInfo) -> Self {
        Self {
            job_id,
            info,
            score: 0.0
        }
    }
    pub fn run(&mut self, config: &config::Config) {
        self.init();
        self.compile_source_code(config);
        let problem = config.problems.iter().find(
            |item| { item.id==self.info.problem_id }
            ).unwrap();
        let mut score: f32 = 0.0;
        problem.cases.iter()
            . filter(
                |case| { self.run_one_case(case) }
                )
            .for_each(
                |case| { score += case.score }
                );
        self.score = score;
        self.clear();
    }
    pub fn is_valid(&self, config: &config::Config) -> bool {
        if config.languages.iter().find(
            |item| {item.name==self.info.language}).is_none() 
        { return false;}
        if config.problems.iter().find(
            |item| { item.id==self.info.problem_id }).is_none()
        { return false;}
        return true;
    }
    fn init(&self) {
        self.clear();
        let path = format!("{}/job_{}", DIRPREFIX, self.job_id);
        if !Path::new(&path).is_dir() {
            fs::create_dir(&path).expect(
                format!("Creat dir {} failed", &path).as_str()
                );
        }
    }
    fn clear(&self) {
        let path = format!("{}/job_{}", DIRPREFIX, self.job_id);
        if Path::new(&path).is_dir() {
            fs::remove_dir_all(&path).unwrap();
        }
    }
    fn run_one_case(&self, case: &config::Case) -> bool {
        let input = File::open(&case.input_file).unwrap();
        let output = OpenOptions::new().read(true).write(true).truncate(true).create(true)
            .open(self.path("output")).expect("Creat output Failed");
        let mut process = Command::new(self.path("a.out"))
            .stdin(input)
            .stdout(Stdio::from(output)).spawn().expect("Spawn cases Failed");
        let res = process.wait_timeout(Duration::from_micros(case.time_limit as u64)).expect("Wait failed");
        if res.is_some() {
            let ans = fs::read_to_string(&case.answer_file).unwrap();
            let output = fs::read_to_string(self.path("output")).unwrap();
            if ans == output {
                return true;
            }
        }
        return false;
    }
    fn path(&self, filename: &str) -> String {
        format!("{}/job_{}/{}", DIRPREFIX, &self.job_id, filename)
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
                format!( "Job {} spawn failed", self.job_id) .as_str(),
                );
        process.wait().expect(
            format!( "Job {} compile failed", self.job_id) .as_str(),
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
            job_id: 0,
            info,
            score: 0.0
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
    #[test]
    fn test_run_one_case() {
        let json = fs::read_to_string("./config.json").unwrap();
        let config: Config = serde_json::from_str(&json).expect("Parse failed");
        let info = JobInfo {
            source_code: "fn main() {let mut line1 = String::new();std::io::stdin().read_line(&mut line1).unwrap();let a: i32 = line1.trim().parse().unwrap();let mut line2 = String::new();std::io::stdin().read_line(&mut line2).unwrap();let b: i32 = line2.trim().parse().unwrap();println!(\"{}\", a + b);}".to_string(),
            language: "Rust".to_string(),
            user_id: 0,
            contest_id: 0,
            problem_id: 0
        };
        let job = Job {
            job_id: 0,
            info,
            score: 0.0
        };
        job.init();
        job.compile_source_code(&config);
        let problem = &config.problems[0];
        let case1 = &problem.cases[0];
        let case2 = &problem.cases[1];

        assert!(job.run_one_case(case1));
        assert!(job.run_one_case(case2));

        job.clear();
    }
    #[test]
    fn test_run_aplusb() {
        let json = fs::read_to_string("./config.json").unwrap();
        let config: Config = serde_json::from_str(&json).expect("Parse failed");
        let info = JobInfo {
            source_code: "fn main() {let mut line1 = String::new();std::io::stdin().read_line(&mut line1).unwrap();let a: i32 = line1.trim().parse().unwrap();let mut line2 = String::new();std::io::stdin().read_line(&mut line2).unwrap();let b: i32 = line2.trim().parse().unwrap();println!(\"{}\", a + b);}".to_string(),
            language: "Rust".to_string(),
            user_id: 0,
            contest_id: 0,
            problem_id: 0
        };
        let mut job = Job {
            job_id: 0,
            info,
            score: 0.0
        };
        // job.init();
        assert_eq!(job.score, 0.0);
        job.run(&config);
        assert_eq!(job.score, 100.0);
        job.clear();
    }
}

