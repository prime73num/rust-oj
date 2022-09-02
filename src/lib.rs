
pub mod postapi;
pub mod config;
pub mod job;

use std::sync::{Mutex, Arc};
use job::JobInfo;
use lazy_static::lazy_static;
use serde::Serialize;


lazy_static!(
    pub static ref JOBDATA: Arc<Mutex<JobData>> = Arc::default();
);

pub struct JobData {
    job_list: Vec<job::Job>,
    total_jobs: u32,
    user_list: Vec<User>
}

impl Default for JobData{
    fn default() -> Self {
        let job_list: Vec<job::Job> = Vec::new();
        let total_jobs = 0;
        let user_list = vec![User{
            id: 0,
            name: "root".to_string()
        }];
        Self {
            job_list, 
            total_jobs,
            user_list
        }
    }
}

pub struct User {
    id: u32,
    name: String
}

#[derive(Debug, Serialize, Clone)]
pub struct CaseResult {
    id: u32,
    result: RunResult,
    time: u32,
    memory: u32,
    info: String
}

impl CaseResult {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            result: RunResult::Waiting,
            time: 0,
            memory: 0,
            info: String::new()
        }
    }

}
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    code: u32,
    reason: String
}

impl ErrorResponse {
    pub fn new(code: u32, reason: &str) -> Self {
        Self {
            code,
            reason: reason.to_string()
        }
    }

}

#[derive(Debug, Serialize)]
pub struct Response {
    id: u32,
    created_time: String,
    updated_time: String,
    submission: JobInfo,
    state: State,
    result: RunResult,
    score: f32,
    cases: Vec<CaseResult>
}


#[derive(Debug, Serialize, Clone, Copy)]
pub enum State {
    Queueing,
    Running,
    Finished,
    Canceled
}

#[derive(Debug, Serialize, PartialEq, Clone, Copy)]
pub enum RunResult {
    Waiting,
    Running,
    Accepted,
    #[serde(rename(serialize = "Compilation Error"))]
    CompilationError,
    #[serde(rename(serialize = "Compilation Success"))]
    CompilationSuccess,
    #[serde(rename(serialize = "Wrong Answer"))]
    WrongAnswer,
    #[serde(rename(serialize = "Runtime Error"))]
    RuntimeError,
    #[serde(rename(serialize = "Time Limit Exceeded"))]
    TimeLimitExceeded,
    #[serde(rename(serialize = "Memory Limit Exceeded"))]
    MemoryLimitExceeded,
    #[serde(rename(serialize = "System Error"))]
    SystemError,
    #[serde(rename(serialize = "SPJ Error"))]
    SpjError,
    Skipped
}
