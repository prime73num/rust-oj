
pub mod jobapi;
pub mod config;
pub mod job;
pub mod userapi;

use std::sync::{Mutex, Arc};
use job::{JobInfo, Job};
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};


lazy_static!(
    pub static ref JOBDATA: Arc<Mutex<JobData>> = Arc::default();
);

pub struct JobData {
    job_list: Vec<job::Job>,
    total_jobs: u32,
    user_list: Vec<User>,
    total_users: u32,
}

impl JobData {
    pub fn add_job(&mut self, info: JobInfo, config: &config::Config) -> Option<Response> {
        let id = self.total_jobs;
        let res = self.user_list.iter().find(|x| x.id==info.user_id);
        if res.is_none() {
            return None;
        }
        let user_name = &res.unwrap().name;
        let mut job = Job::new(&user_name, id, info);
        if !job.is_valid(config) {
            return None;
        }
        job.run(config);
        let res = job.response();
        self.total_jobs += 1;
        self.job_list.push(job);
        Some(res)
    }
    pub fn add_user(&mut self, name: &str) -> UserRes {
        if self.user_list.iter().find(|x| {
            x.name == name
        }).is_some() { return UserRes::NameExit;}
        let temp = User::new(self.total_users, name);
        self.user_list.push(temp.clone());
        self.total_users += 1;
        UserRes::Succecc(temp)
    }
    pub fn update_user(&mut self, id: u32, name: &str) -> UserRes {
        let has_name = self.user_list.iter().find(|x| {
            x.name == name
        }).is_some();
        let res = self.user_list.iter_mut().find(|x| {
            x.id == id
        });
        if res.is_none() { return UserRes::IdNotFound; }
        if has_name { return UserRes::NameExit; }

        let user = res.unwrap();
        user.name = name.to_string();
        UserRes::Succecc(user.clone())
    }
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
            user_list,
            total_users: 1
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    id: u32,
    name: String
}
#[derive(Debug)]
pub enum UserRes {
    Succecc(User),
    IdNotFound,
    NameExit,
}

impl User {
    pub fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string()
        }
    }
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


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum State {
    Queueing,
    Running,
    Finished,
    Canceled
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
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
