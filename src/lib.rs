
pub mod config;
pub mod job;

pub mod job_api;
pub mod user_api;
pub mod contest_api;

use std::sync::{Mutex, Arc};
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};

use user_api::UserInfo;
use job::{JobInfo, Job};
use contest_api::{ContestInfo, HttpcomInfo};


lazy_static!(
    pub static ref JOBDATA: Arc<Mutex<JobData>> = Arc::default();
);

pub struct JobData {
    job_list: Vec<job::Job>,
    total_jobs: u32,
    user_list: Vec<User>,
    total_users: u32,
    contests_list: Vec<ContestInfo>,
    total_contests: u32,
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
    pub fn post_user(&mut self, mut info: UserInfo) -> UserRes {
        if let Some(id) = info.id {
            let res = self.user_list.iter().find(|x| {
                x.id == id
            });
            if res.is_none() { return UserRes::IdNotFound; }
        }
        if self.user_list.iter().find(|x| {
            x.name == info.name
        }).is_some() { return UserRes::NameExit;}
        match info.id {
            Some(id) => {
                let user = self.user_list.iter_mut().find(|x| {
                    x.id == id
                }).unwrap();
                *user = User::from(info);
                UserRes::Succecc(user.clone())
            },
            None => {
                info.id = Some(self.total_users);
                let temp = User::from(info);
                self.user_list.push(temp.clone());
                self.total_users += 1;
                UserRes::Succecc(temp)
            },
        }
    }
    pub fn post_contest(&mut self, mut info: HttpcomInfo, config: &config::Config) -> Option<ContestInfo> {
        if let Some(id) = info.id {
            if self.contests_list.iter().find(|x| x.id==id).is_none() {
                return None;
            }
        }
        let res = info.user_ids.iter().all(|x| {
            self.user_list.iter().find(|user| {user.id==*x}).is_some()
        });
        if !res { return None; }
        let res = info.problem_ids.iter().all(|x| {
            config.problems.iter().find(|problem| {problem.id==*x}).is_some()
        });
        if !res { return None; }

        match info.id {
            Some(id) => {
                let contest = ContestInfo::from(info);
                let pos = self.contests_list.iter_mut().find(|x| x.id==id).unwrap();
                *pos = contest.clone();
                return Some(contest);
            },
            None => {
                info.id = Some(self.total_contests);
                let contest = ContestInfo::from(info);
                self.contests_list.push(contest.clone());
                return Some(contest);
            },
        }
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
            total_users: 1,
            contests_list: Vec::new(),
            total_contests: 0
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
    pub fn from(info: UserInfo) -> Self {
        Self {
            id: info.id.unwrap(),
            name: info.name
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
