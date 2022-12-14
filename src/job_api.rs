

use actix_web::{
    delete, put, get, post, web, 
    Responder, 
    HttpResponse,
};
use chrono::DateTime;
use log;
use serde::{Serialize, Deserialize};

use crate::config::Config;
use crate::job::{JobInfo, Job};
use crate::{JOBDATA, State, RunResult, Response, AppError};


// post a job
#[post("/jobs")]
pub async fn post_jobs(info: web::Json<JobInfo>, config: web::Data<Config>) -> Result<HttpResponse, AppError> {

    let info = info.into_inner();
    let job_data = JOBDATA.clone();
    let mut job_data_inner = job_data.lock().unwrap();

    let res = job_data_inner.add_job(&info, &config)?;

    log::info!(target: "post_jobs_handler", "post job {}", res.id);
    return Ok(HttpResponse::Ok().json(res));
}

// use this struct to filter job
#[derive(Debug, Serialize, Deserialize)]
pub struct UrlQuery {
    user_id: Option<u32>,
    user_name: Option<String>,
    contest_id: Option<u32>,
    problem_id: Option<u32>,
    language: Option<String>,
    from: Option<String>,
    to: Option<String>,
    state: Option<State>,
    result: Option<RunResult>
}

impl UrlQuery {
    // decide whether the job meets the predicate
    pub fn predicate(&self, job: &Job) -> bool {
        if !self.user_id.map_or(true, |x| {
            job.info.user_id==x
        }) { return false;}
        if !self.user_name.as_ref().map_or(true, |x| {
            job.user_name==*x
        }) { return false;}
        if !self.contest_id.map_or(true, |x| {
            job.info.contest_id==x
        }) { return false;}
        if !self.problem_id.map_or(true, |x| {
            job.info.problem_id==x
        }) { return false;}
        if !self.language.as_ref().map_or(true, |x| {
            job.info.language==*x
        }) { return false;}
        if !self.from.as_ref().map_or(true, |x| {
            let time = DateTime::parse_from_str(x, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
            return job.created_time > time;
        }) { return false;}
        if !self.to.as_ref().map_or(true, |x| {
            let time = DateTime::parse_from_str(x, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
            return job.created_time < time;
        }) { return false;}
        if !self.state.map_or(true, |x| {
            job.state==x
        }) { return false;}
        if !self.result.map_or(true, |x| {
            job.result==x
        }) { return false;}
        return true;
    }
}

// get the job list the a query
#[get("/jobs")]
pub async fn get_jobs(query: web::Query<UrlQuery>) -> impl Responder {
    let query = query.into_inner();
    let job_data = JOBDATA.clone();
    let job_data_inner = job_data.lock().unwrap();
    let mut temp_job_list: Vec<&Job> = job_data_inner.job_list.iter().collect();
    temp_job_list.sort_by_key(|x| {x.created_time});

    let res: Vec<Response> = temp_job_list.iter().filter(|job| {
        query.predicate(job)   // use this to filter
    }).map(|x| {
        x.response()
    }).collect();
    drop(job_data_inner);
    log::info!(target: "get jobs", "get jobs list");
    return HttpResponse::Ok().json(res);
}

// get job with the id
#[get("/jobs/{jobid}")]
pub async fn get_jobs_id(jobid: web::Path<u32>) -> Result<HttpResponse, AppError> {
    let job_data = JOBDATA.clone();
    let job_data_inner = job_data.lock().unwrap();
    let response = job_data_inner.get_job_response(*jobid)?;
    log::info!(target: "get_jobs_id", "get jobs {}", response.id);
    return Ok(HttpResponse::Ok().json(response));
}

// rerun the job of the id
#[put("/jobs/{jobid}")]
pub async fn put_job(jobid: web::Path<u32>, config: web::Data<Config>) -> Result<HttpResponse, AppError> {
    let job_data = JOBDATA.clone();
    let mut job_data_inner = job_data.lock().unwrap();
    let job = job_data_inner.find_job_mut(*jobid)?;
    if job.state != State::Finished {
        return Err(AppError::ERR_INVALID_STATE);
    }
    job.run(&config);

    log::info!(target: "put_job", "put jobs {}", job.job_id);
    return Ok(HttpResponse::Ok().json(job.response()));
}

// delete the job
#[delete("/jobs/{jobid}")]
pub async fn delete_job(jobid: web::Path<u32>) -> Result<HttpResponse, AppError> {
    let job_data = JOBDATA.clone();
    let mut job_data_inner = job_data.lock().unwrap();
    let job = job_data_inner.job_list.iter_mut().enumerate().find(|x| {
        x.1.job_id==*jobid
    });
    if job.is_none() {
        return Err(AppError::ERR_NOT_FOUND);
    }
    let (idx, job) = job.unwrap();
    if job.state != State::Queueing {
        return Err(AppError::ERR_INVALID_STATE);
    }
    log::info!(target: "delete_job", "delete job {}", job.job_id);
    job_data_inner.job_list.remove(idx);
    return Ok(HttpResponse::Ok().finish());
}

