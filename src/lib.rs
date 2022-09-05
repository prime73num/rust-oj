
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



// the global variable
// use this as the database
lazy_static!(
    pub static ref JOBDATA: Arc<Mutex<JobData>> = Arc::default();
);


// define all kinds of the errors as specified in the doc
// and implement the ResponseError trait for AppError
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
    // map the enum to the ErrorResponse struct
    // which will be send as json content in the HttpResponse
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
    // map the enum to HttpResponse StatusCode
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



// the jobdata has a job list, user list and a contest list
pub struct JobData {
    job_list: Vec<job::Job>,
    total_jobs: u32,
    user_list: Vec<User>,
    total_users: u32,
    contests_list: Vec<(ContestInfo, HashMap<(u32, u32), u32>)>,
    total_contests: u32,
}

impl JobData {
    // add job to the job list
    // first check if it is valid
    // if valid add it to the list and return the run response
    // otherwise return error
    pub fn add_job(&mut self, info: &JobInfo, config: &config::Config) -> Result<Response, AppError> {
        let id = self.total_jobs;

        // check the user id 
        let res = self.find_user(info.user_id)?;

        let user_name = res.name.clone();
        let mut job = Job::new(&user_name, id, info);

        // check if the config has problem_id and language
        if !job.is_valid(config) {
            return Err(AppError::ERR_NOT_FOUND);
        }

        let mut temp = 0;
        let mut submission_time = &mut temp;
        // contest check
        if info.contest_id != 0 {
            // check contest id
            let contest = self.find_contest_mut(info.contest_id)?;
            if !contest.0.is_valid(&info) {  // check user id and problem id in the contest
                return Err(AppError::ERR_INVALID_ARGUMENT);
            }
            // check submission_time
            let entry = contest.1.entry((info.user_id, info.problem_id)).or_insert(0);
            if *entry >= contest.0.submission_limit {
                return Err(AppError::ERR_RATE_LIMIT);
            }
            submission_time = entry;
        }
        // run the job and get the response
        job.run(config);
        let res = job.response();

        *submission_time += 1;
        self.total_jobs += 1;
        self.job_list.push(job);
        Ok(res)
    }
    // find the contest
    pub fn find_contest(&self, contest_id: u32) -> Result<&(ContestInfo, HashMap<(u32,u32), u32>), AppError> {
        let response = self.contests_list.iter().find(|x| {
            x.0.id==contest_id
        });
        return response.map_or(
            Err(AppError::ERR_NOT_FOUND),
            |x| { Ok(x) }
            );
    }
    // find the contest
    pub fn find_contest_mut(&mut self, contest_id: u32) -> Result<&mut (ContestInfo, HashMap<(u32,u32), u32>), AppError> {
        let response = self.contests_list.iter_mut().find(|x| {
            x.0.id==contest_id
        });
        return response.map_or(
            Err(AppError::ERR_NOT_FOUND),
            |x| { Ok(x) }
            );
    }
    // find the user
    pub fn find_user_mut(&mut self, user_id: u32) -> Result<&mut User, AppError> {
        let response = self.user_list.iter_mut().find(|x| {
            x.id==user_id
        });
        return response.map_or(
            Err(AppError::ERR_NOT_FOUND),
            |x| { Ok(x) }
            );
    }
    // find the user
    pub fn find_user(&self, user_id: u32) -> Result<&User, AppError> {
        let response = self.user_list.iter().find(|x| {
            x.id==user_id
        });
        return response.map_or(
            Err(AppError::ERR_NOT_FOUND),
            |x| { Ok(x) }
            );
    }
    // find the job
    pub fn find_job_mut(&mut self, jobid: u32) -> Result<&mut Job, AppError> {
        let response = self.job_list.iter_mut().find(|x| {
            x.job_id==jobid
        });
        return response.map_or(
            Err(AppError::ERR_NOT_FOUND),
            |x| { Ok(x) }
            );
    }
    // find the job
    pub fn find_job(&self, jobid: u32) -> Result<&Job, AppError> {
        let response = self.job_list.iter().find(|x| {
            x.job_id==jobid
        });
        return response.map_or(
            Err(AppError::ERR_NOT_FOUND),
            |x| { Ok(x) }
            );
    }
    // get the job response
    pub fn get_job_response(&self, jobid: u32) -> Result<Response, AppError> {
        let response = self.find_job(jobid);
        return response.map_or(
            Err(AppError::ERR_NOT_FOUND),
            |x| { Ok(x.response()) }
            );
    }
    // post a user 
    pub fn post_user(&mut self, mut info: UserInfo) -> Result<User, AppError> {
        // check valid
        if let Some(id) = info.id {
            self.find_user(id)?;
        }
        if self.user_list.iter().any(|x| {
            x.name == info.name
        }) { return Err(AppError::ERR_INVALID_ARGUMENT);}
        // add the user
        match info.id {
            // update user
            Some(id) => {
                let user = self.find_user_mut(id)?;
                *user = User::from(info);
                Ok(user.clone())
            },
            // new user
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
        // check valid
        if let Some(id) = info.id {
            self.find_contest(id)?;
        }
        let res = info.user_ids.iter().all(|x| {
            self.user_list.iter().any(|user| {user.id==*x})
        });
        if !res {
            return Err(AppError::ERR_NOT_FOUND);
        }
        let res = info.problem_ids.iter().all(|x| {
            config.problems.iter().any(|problem| {problem.id==*x})
        });
        if !res {
            return Err(AppError::ERR_NOT_FOUND);
        }

        // add contest
        match info.id {
            // update contest
            Some(id) => {
                let contest = ContestInfo::from(info);
                let pos = self.find_contest_mut(id)?;
                pos.0 = contest.clone();
                return Ok(contest);
            },
            // new contest
            None => {
                info.id = Some(self.total_contests);
                self.total_contests += 1;
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
            total_contests: 1
        }
    }
}

// represent a user with id and name
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    id: u32,
    name: String
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

// error_response
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

// the job response
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


// job state
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum State {
    Queueing,
    Running,
    Finished,
    Canceled
}

// job result
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum RunResult {
    Waiting,
    Running,
    Accepted,
    #[serde(rename(serialize = "Compilation Error", deserialize = "Compilation Error"))]
    CompilationError,
    #[serde(rename(serialize = "Compilation Success", deserialize = "Compilation Success"))]
    CompilationSuccess,
    #[serde(rename(serialize = "Wrong Answer", deserialize = "Wrong Answer"))]
    WrongAnswer,
    #[serde(rename(serialize = "Runtime Error", deserialize = "Runtime Error"))]
    RuntimeError,
    #[serde(rename(serialize = "Time Limit Exceeded", deserialize = "Time Limit Exceeded"))]
    TimeLimitExceeded,
    #[serde(rename(serialize = "Memory Limit Exceeded", deserialize = "Memory Limit Exceeded"))]
    MemoryLimitExceeded,
    #[serde(rename(serialize = "System Error", deserialize = "System Error"))]
    SystemError,
    #[serde(rename(serialize = "SPJ Error", deserialize = "SPJ Error"))]
    SpjError,
    Skipped
}
