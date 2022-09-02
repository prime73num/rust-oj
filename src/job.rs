use std::fs::{write, File, self, OpenOptions};
use std::path::Path;
use std::process::{Command, Stdio};
use std::io;

use serde::Deserialize;
use serde::Serialize;

use wait_timeout::ChildExt;
use std::time::Duration;

use chrono::prelude::*;

use crate::{config, State};
use crate::RunResult;
use crate::CaseResult;
use crate::Response;

const DIRPREFIX: &str = "./tmp";


#[derive(Debug, Serialize, Deserialize, Clone)]
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
    score: f32,
    created_time: String,
    updated_time: String,
    state: State,
    result: RunResult,
}

impl Job {
    pub fn new(job_id: u32, info: JobInfo) -> Self {
        Self {
            job_id,
            info,
            score: 0.0,
            created_time: "".to_string(),
            updated_time: "".to_string(),
            state: State::Queueing,
            result: RunResult::Waiting,
        }
    }
    pub fn run(&mut self, config: &config::Config) -> Response {

        if !self.is_valid(config) {
            log::info!(target: "Job::run", "Job invalid");
        }
        let mut cases: Vec<CaseResult> = Vec::new();
        let problem = config.problems.iter().find(
            |item| { item.id==self.info.problem_id }
            ).unwrap();
        cases.push(CaseResult::new(0));
        problem.cases.iter().enumerate().for_each( |(mut i,_)| {
            i += 1;
            cases.push(CaseResult::new(i as u32));
        });

        self.init();
        self.result = RunResult::Running;
        self.state = State::Running;

        let res = self.compile_source_code(config);
        cases[0].result = res;
        if res != RunResult::CompilationSuccess {
            self.result = RunResult::CompilationError;
            return self.response(cases);
        }
        self.result = RunResult::CompilationSuccess;

        for (i, case) in problem.cases.iter().enumerate() {
            let res = self.run_one_case(case);
            cases[i+1].result = res;
            self.result = res;
            if res != RunResult::Accepted {
                return self.response(cases);
            }
            self.score += case.score;
        }
        self.clear();
        return self.response(cases);
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
    fn response(&mut self, cases: Vec<CaseResult>) -> Response {
        let dt = Utc::now();
        self.updated_time = dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        Response {
            id: self.job_id,
            created_time: self.created_time.clone(),
            updated_time: self.updated_time.clone(),
            submission: self.info.clone(),
            state: State::Finished,
            result: self.result,
            score: self.score,
            cases,
        }
    }
    fn init(&mut self) -> io::Result<()>{
        self.clear();
        let path = format!("{}/job_{}", DIRPREFIX, self.job_id);
        if !Path::new(&path).is_dir() {
            fs::create_dir(&path)?;
        }
        let dt = Utc::now();
        self.created_time = dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        self.updated_time = dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        self.score = 0.0;
        self.state = State::Queueing;
        self.result = RunResult::Waiting;
        Ok(())
    }
    fn clear(&self) -> io::Result<()> {
        let path = format!("{}/job_{}", DIRPREFIX, self.job_id);
        if Path::new(&path).is_dir() {
            fs::remove_dir_all(&path)?;
        }
        Ok(())
    }
    fn run_one_case(&self, case: &config::Case) -> RunResult {
        let try_do = || -> io::Result<RunResult> {
            let input = File::open(&case.input_file)?;
            let output = OpenOptions::new().read(true).write(true).truncate(true).create(true)
                .open(self.path("output"))?;
            let mut process = Command::new(self.path("a.out"))
                .stdin(input)
                .stdout(Stdio::from(output)).spawn()?;
            let res = process.wait_timeout(Duration::from_micros(case.time_limit as u64))?;
            match res {
                Some(exit) => {
                    if exit.success() {
                        let ans = fs::read_to_string(&case.answer_file)?;
                        let out = fs::read_to_string(self.path("output"))?;
                        if ans==out { return Ok(RunResult::Accepted);}
                        else { return Ok(RunResult::WrongAnswer);}
                    }
                    return Ok(RunResult::RuntimeError);
                },
                None => {
                    process.kill()?;
                    return Ok(RunResult::TimeLimitExceeded);
                }
            }
        };
        let res = try_do().unwrap_or_else(|err| {
            log::info!(target: "Job::run_one_case", "System io error {}", err);
            RunResult::SystemError
        });
        return res;
    }
    fn path(&self, filename: &str) -> String {
        format!("{}/job_{}/{}", DIRPREFIX, &self.job_id, filename)
    }
    fn compile_source_code(&self, config: &config::Config) -> RunResult {

        let try_do = || -> io::Result<RunResult> {
            let mut language = config.languages.iter().find(
                |item| {item.name==self.info.language}
                ).unwrap().clone();

            language.replace("%OUTPUT%", &self.path("a.out"));
            language.replace("%INPUT%", &self.path(&language.file_name));
            write(self.path(&language.file_name), &self.info.source_code)?;

            let mut process = Command::new(&language.command[0])
                .args(&language.command[1..])
                .spawn()?;
            let exitstatus = process.wait()?;
            if !exitstatus.success() { return Ok(RunResult::CompilationError);}
            return Ok(RunResult::CompilationSuccess);
        };
        let res = try_do().unwrap_or_else(|err| {
            log::info!(target: "Job::compile_source_code", "System io error {}", err);
            RunResult::SystemError
        });
        return res;
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
        let mut job = Job::new(0, info);
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
        let mut job = Job::new(0, info);
        job.init();
        let res = job.compile_source_code(&config);
        assert_eq!(res, RunResult::CompilationSuccess);
        let problem = &config.problems[0];
        let case1 = &problem.cases[0];
        let case2 = &problem.cases[1];

        let res = job.run_one_case(case1);
        assert_eq!(res, RunResult::Accepted);
        let res = job.run_one_case(case2);
        assert_eq!(res, RunResult::Accepted);

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
        let mut job = Job::new(0, info);
        // job.init();
        assert_eq!(job.score, 0.0);
        job.run(&config);
        assert_eq!(job.score, 100.0);
        job.clear();
    }
}

