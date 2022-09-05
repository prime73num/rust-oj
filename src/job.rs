use std::fs::{write, File, self, OpenOptions};
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{io, usize};

use serde::Deserialize;
use serde::Serialize;

use serde_json::json;
use wait_timeout::ChildExt;
use std::time::Duration;

use chrono::prelude::*;

use crate::{config, State};
use crate::RunResult;
use crate::CaseResult;
use crate::Response;

const DIRPREFIX: &str = "./tmp";


// the struct represent the json content from the post job http request 
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JobInfo {
    pub source_code: String,
    pub language: String,
    pub user_id: u32,
    pub contest_id: u32,
    pub problem_id: u32
}

// use this struct to run a job and get a response
pub struct Job {
    pub user_name: String,
    pub job_id: u32,
    pub info: JobInfo,
    pub score: f32,
    pub created_time: DateTime<Utc>,
    pub updated_time: DateTime<Utc>,
    pub state: State,
    pub result: RunResult,
    pub case_res: Vec<CaseResult>
}

impl Job {
    pub fn new(user_name: &str, job_id: u32, info: &JobInfo) -> Self {
        Self {
            user_name: user_name.to_string(),
            job_id,
            info: info.clone(),
            score: 0.0,
            created_time: DateTime::default(),
            updated_time: DateTime::default(),
            state: State::Queueing,
            result: RunResult::Waiting,
            case_res: Vec::new()
        }
    }

    // sparate the run commond to several parts
    // first init and the compile the souce code 
    // then run the a.out and compare to answer to get the result for each case
    pub fn run(&mut self, config: &config::Config) -> Response {

        // get the problem from the config
        let problem = config.problems.iter().find(
            |item| { item.id==self.info.problem_id }
            ).unwrap();

        // if init failed set the state and result and return
        if !self.init(config) {
            self.state = State::Finished;
            self.result = RunResult::SystemError;
            return self.response();
        }
        // init success set the state and result
        self.result = RunResult::Running;
        self.state = State::Running;

        // compile failed set state and result return
        if !self.compile_source_code(config, 0) {
            self.state = State::Finished;
            self.result = RunResult::CompilationError;
            return self.response();
        }
        // compile success
        self.result = RunResult::CompilationSuccess;

        let mut ans = true;
        // run and test each case of the problem
        for (i, case) in problem.cases.iter().enumerate() {
            if !self.run_one_case(problem, case, i+1) {
                self.state = State::Finished;
                self.result = self.case_res[i+1].result;
                return self.response();
            }
            if self.case_res[i+1].result==RunResult::Accepted {
                self.score += case.score;
            } else {
                ans = false;
            }
        }
        self.state = State::Finished;
        if ans {
            self.result = RunResult::Accepted;
        } else {
            self.result = RunResult::WrongAnswer;
        }
        return self.response();
    }
    // check valid of the job with the config
    pub fn is_valid(&self, config: &config::Config) -> bool {
        if config.languages.iter().find(
            |item| {item.name==self.info.language}).is_none() 
        { return false;}
        if config.problems.iter().find(
            |item| { item.id==self.info.problem_id }).is_none()
        { return false;}
        return true;
    }
    // use the informatin of the struct to construct a response
    pub fn response(&self) -> Response {
        Response {
            id: self.job_id,
            created_time: self.created_time.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            updated_time: self.updated_time.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            submission: self.info.clone(),
            state: self.state,
            result: self.result,
            score: self.score,
            cases: self.case_res.clone()
        }
    }
    // init for the job 
    // clean the directory
    // set the init field value
    // return true if init success 
    // false othewise
    fn init(&mut self, config: &config::Config) -> bool {
        // try clean the directory
        let try_do = || -> io::Result<()> {
            self.clear();
            let path = format!("{}/job_{}", DIRPREFIX, self.job_id);
            if !Path::new(&path).is_dir() {
                fs::create_dir(&path)?;
            }
            Ok(())
        };
        if let Err(e) = try_do() {
            // failed to clean
            log::info!(target: "Job::init", "System io error {}", e);
            self.result = RunResult::SystemError;
            return false;
        }

        assert!(self.is_valid(config));
        // set the init value of the field
        self.created_time = Utc::now();
        self.updated_time = Utc::now();
        self.score = 0.0;
        self.state = State::Queueing;
        self.result = RunResult::Waiting;
        self.case_res.clear();
        let problem = config.problems.iter().find(
            |item| { item.id==self.info.problem_id }
            ).unwrap();
        // init case res with id and waiting result
        for i in 0..=problem.cases.len() {
            self.case_res.push(CaseResult::new(i as u32));
        }
        return true;
    }
    // clear the directory
    fn clear(&self) {
        let path = format!("{}/job_{}", DIRPREFIX, self.job_id);
        if Path::new(&path).is_dir() {
            fs::remove_dir_all(&path).expect("Clear failed");
        }
    }
    // run one case of the problem
    // return true if success 
    // false otherwise
    fn run_one_case(&mut self, problem: &config::Problem, case: &config::Case, caseidx: usize) -> bool {
        let mut ret = false;
        let mut info = String::new();
        // try run one case
        let mut try_do = || -> io::Result<RunResult> {
            // input and output file
            let input = File::open(&case.input_file)?;
            let output = OpenOptions::new().read(true).write(true).truncate(true).create(true)
                .open(self.path("output"))?;
            // creat the process
            let mut process = Command::new(self.path("a.out"))
                .stdin(input)
                .stdout(Stdio::from(output)).spawn()?;
            // wait timeout of the process
            let res = process.wait_timeout(Duration::from_micros(case.time_limit as u64))?;
            match res {
                // exit 
                Some(exit) => {
                    // exit successs
                    if exit.success() {
                        ret = true;
                        // problem with special_judge argument
                        if let Some(spj) = problem.misc.get("special_judge") {
                            let mut args: Vec<String> = serde_json::from_value(spj.clone()).unwrap();
                            let pos = args.iter_mut().find(|item| {*item=="%OUTPUT%"}).unwrap();
                            *pos = self.path("output");
                            let pos = args.iter_mut().find(|item| {*item=="%ANSWER%"}).unwrap();
                            *pos = case.answer_file.clone();
                            let process = Command::new(&args[0])
                                .args(&args[1..])
                                .stdout(Stdio::piped()).spawn()?;
                            let mut res = String::new();
                            process.stdout.unwrap().read_to_string(&mut res).unwrap();
                            let res: Vec<&str> = res.split('\n').map(|x| {x}).collect();
                            let ret: RunResult = serde_json::from_value(
                                json!(res[0].to_string())
                                ).unwrap();
                            info = res[1].to_string();
                            return Ok(ret);
                        } else {  // problem with out special_judge argument
                            let ans = fs::read_to_string(&case.answer_file)?;
                            let out = fs::read_to_string(self.path("output"))?;
                            if ans==out { return Ok(RunResult::Accepted);}
                            else { return Ok(RunResult::WrongAnswer);}
                        }
                    }
                    // exit with error 
                    return Ok(RunResult::RuntimeError);
                },
                // timeout
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

        // set the case result of the caseidx
        self.case_res[caseidx].result = res;
        self.case_res[caseidx].info = info;
        return ret;
    }
    // the root temp diectory of the job
    fn path(&self, filename: &str) -> String {
        format!("{}/job_{}/{}", DIRPREFIX, &self.job_id, filename)
    }
    // compile source code
    fn compile_source_code(&mut self, config: &config::Config, caseidx: usize) -> bool {

        let mut ret = false;
        // try compile return io error if failed
        let mut try_do = || -> io::Result<RunResult> {
            let mut language = config.languages.iter().find(
                |item| {item.name==self.info.language}
                ).unwrap().clone();

            // replace compile commond with the output file and a.out
            language.replace("%OUTPUT%", &self.path("a.out"));
            language.replace("%INPUT%", &self.path(&language.file_name));
            write(self.path(&language.file_name), &self.info.source_code)?;

            let mut process = Command::new(&language.command[0])
                .args(&language.command[1..])
                .spawn()?;
            let exitstatus = process.wait()?;
            if !exitstatus.success() { return Ok(RunResult::CompilationError);}
            ret = true;
            return Ok(RunResult::CompilationSuccess);
        };
        let res = try_do().unwrap_or_else(|err| {
            log::info!(target: "Job::compile_source_code", "System io error {}", err);
            RunResult::SystemError
        });
        self.case_res[caseidx].result = res;
        return ret;
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
        let mut job = Job::new("root", 0, &info);
        job.init(&config);
        job.compile_source_code(&config, 0);
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
        let mut job = Job::new("root", 0, &info);
        job.init(&config);
        let res = job.compile_source_code(&config, 0);
        assert!(res);
        let problem = &config.problems[0];
        let case1 = &problem.cases[0];
        let case2 = &problem.cases[1];

        let res = job.run_one_case(problem, case1, 1);
        assert!(res);
        let res = job.run_one_case(problem, case2, 2);
        assert!(res);

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
        let mut job = Job::new("root", 0, &info);
        // job.init();
        assert_eq!(job.score, 0.0);
        job.run(&config);
        assert_eq!(job.score, 100.0);
        job.clear();
    }
    #[test]
    fn test_response() {
        let json = fs::read_to_string("./config.json").unwrap();
        let config: Config = serde_json::from_str(&json).expect("Parse failed");
        let info = JobInfo {
            source_code: "fn main() {let mut line1 = String::new();std::io::stdin().read_line(&mut line1).unwrap();let a: i32 = line1.trim().parse().unwrap();let mut line2 = String::new();std::io::stdin().read_line(&mut line2).unwrap();let b: i32 = line2.trim().parse().unwrap();println!(\"{}\", a + b);}".to_string(),
            language: "Rust".to_string(),
            user_id: 0,
            contest_id: 0,
            problem_id: 0
        };
        // job.init();
        let mut job = Job::new("root", 0, &info);
        assert_eq!(job.score, 0.0);
        let resp = job.run(&config);
        let out = serde_json::to_string_pretty(&resp).unwrap();
        println!("{}", out);
        assert_eq!(job.score, 100.0);

        job.clear();
    }
}

