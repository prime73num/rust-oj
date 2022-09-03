
pub mod config;
pub mod job;

pub mod job_api;
pub mod user_api;
pub mod contest_api;

use std::{sync::{Mutex, Arc}, collections::HashMap};

use actix_web::{
    ResponseError, http::StatusCode, body::BoxBody,
    HttpResponse, HttpResponseBuilder
};
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use derive_more::{Display, Error};

use user_api::UserInfo;
use job::{JobInfo, Job};
use contest_api::{ContestInfo, HttpcomInfo};



lazy_static!(
    pub static ref JOBDATA: Arc<Mutex<JobData>> = Arc::default();
);


#[derive(Debug, Display, Error)]
#[allow(non_camel_case_types)]
pub enum AppError {
    ERR_INVALID_ARGUMENT,
    ERR_INVALID_STATE,
    ERR_NOT_FOUND,
    ERR_RATE_LIMIT,
    ERR_EXTERNAL,
    ERR_INTERNAL
}

impl AppError {
    fn to_response(&self) -> ErrorResponse {
        match self {
            AppError::ERR_INVALID_ARGUMENT => { ErrorResponse::new(1, &self.to_string()) },
            AppError::ERR_INVALID_STATE => { ErrorResponse::new(2, &self.to_string()) },
            AppError::ERR_NOT_FOUND => { ErrorResponse::new(3, &self.to_string()) },
            AppError::ERR_RATE_LIMIT => { ErrorResponse::new(4, &self.to_string() ) },
            AppError::ERR_EXTERNAL => { ErrorResponse::new(5,&self.to_string() ) },
            AppError::ERR_INTERNAL => { ErrorResponse::new(6, &self.to_string()) },
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponseBuilder::new(self.status_code()).json(self.to_response())
    }
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::ERR_INVALID_ARGUMENT => { StatusCode::BAD_REQUEST },
            AppError::ERR_INVALID_STATE => { StatusCode::BAD_REQUEST },
            AppError::ERR_NOT_FOUND => { StatusCode::NOT_FOUND },
            AppError::ERR_RATE_LIMIT => { StatusCode::BAD_REQUEST},
            AppError::ERR_EXTERNAL => { StatusCode::INTERNAL_SERVER_ERROR },
            AppError::ERR_INTERNAL => { StatusCode::INTERNAL_SERVER_ERROR },
        }
    }
}



pub struct JobData {
    job_list: Vec<job::Job>,
    total_jobs: u32,
    user_list: Vec<User>,
    total_users: u32,
    contests_list: Vec<(ContestInfo, HashMap<(u32, u32), u32>)>,
    total_contests: u32,
}

impl JobData {
    pub fn add_job(&mut self, info: JobInfo, config: &config::Config) -> Result<Response, AppError> {
        let id = self.total_jobs;
        let res = self.user_list.iter().find(|x| x.id==info.user_id);
        if res.is_none() {
            return Err(AppError::ERR_NOT_FOUND);
        }
        let user_name = &res.unwrap().name;
        let mut job = Job::new(&user_name, id, info);
        if !job.is_valid(config) {
            return Err(AppError::ERR_NOT_FOUND);
        }
        job.run(config);
        let res = job.response();
        self.total_jobs += 1;
        self.job_list.push(job);
        Ok(res)
    }
    pub fn find_job_mut(&mut self, jobid: u32) -> Result<&mut Job, AppError> {
        let response = self.job_list.iter_mut().find(|x| {
            x.job_id==jobid
        });
        return response.map_or(
            Err(AppError::ERR_NOT_FOUND),
            |x| { Ok(x) }
            );
    }
    pub fn find_job(&self, jobid: u32) -> Result<&Job, AppError> {
        let response = self.job_list.iter().find(|x| {
            x.job_id==jobid
        });
        return response.map_or(
            Err(AppError::ERR_NOT_FOUND),
            |x| { Ok(x) }
            );
    }
    pub fn get_job_response(&self, jobid: u32) -> Result<Response, AppError> {
        let response = self.find_job(jobid);
        return response.map_or(
            Err(AppError::ERR_NOT_FOUND),
            |x| { Ok(x.response()) }
            );
    }
    pub fn post_user(&mut self, mut info: UserInfo) -> Result<User, AppError> {
        if let Some(id) = info.id {
            let res = self.user_list.iter().find(|x| {
                x.id == id
            });
            if res.is_none() { return Err(AppError::ERR_NOT_FOUND); }
        }
        if self.user_list.iter().find(|x| {
            x.name == info.name
        }).is_some() { return Err(AppError::ERR_INVALID_ARGUMENT);}
        match info.id {
            Some(id) => {
                let user = self.user_list.iter_mut().find(|x| {
                    x.id == id
                }).unwrap();
                *user = User::from(info);
                Ok(user.clone())
            },
            None => {
                info.id = Some(self.total_users);
                let temp = User::from(info);
                self.user_list.push(temp.clone());
                self.total_users += 1;
                Ok(temp)
            },
        }
    }
    pub fn post_contest(&mut self, mut info: HttpcomInfo, config: &config::Config) -> Result<ContestInfo, AppError> {
        if let Some(id) = info.id {
            if self.contests_list.iter().find(|x| x.0.id==id).is_none() {
                return Err(AppError::ERR_NOT_FOUND);
            }
        }
        let res = info.user_ids.iter().all(|x| {
            self.user_list.iter().find(|user| {user.id==*x}).is_some()
        });
        if !res {
            return Err(AppError::ERR_NOT_FOUND);
        }
        let res = info.problem_ids.iter().all(|x| {
            config.problems.iter().find(|problem| {problem.id==*x}).is_some()
        });
        if !res {
            return Err(AppError::ERR_NOT_FOUND);
        }

        match info.id {
            Some(id) => {
                let contest = ContestInfo::from(info);
                let pos = self.contests_list.iter_mut().find(|x| x.0.id==id).unwrap();
                pos.0 = contest.clone();
                return Ok(contest);
            },
            None => {
                info.id = Some(self.total_contests);
                let contest = ContestInfo::from(info);
                self.contests_list.push((contest.clone(), HashMap::new()));
                return Ok(contest);
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
