

use actix_web::{
    delete, put, get, post, web, 
    Responder, 
    HttpResponse, HttpResponseBuilder,
    http::StatusCode
};
use chrono::DateTime;
use log;
use serde::{Serialize, Deserialize};

use crate::config::Config;
use crate::job::{JobInfo, Job};
use crate::{JOBDATA, State, RunResult, Response};
use crate::ErrorResponse;


#[post("/jobs")]
pub async fn post_jobs(info: web::Json<JobInfo>, config: web::Data<Config>) -> impl Responder {
    let info = info.into_inner();
    let job_data = JOBDATA.clone();
    let mut job_data_inner = job_data.lock().unwrap();
    let id = job_data_inner.total_jobs;
    let user_name = job_data_inner.user_list.iter().find(|x| {
        x.id == info.user_id
    }).unwrap();
    let job = Job::new(&user_name.name, id, info);
    log::info!(target: "post_jobs_handler", "Post jobs");
    if !job.is_valid(&config) {
        log::info!(target: "post_jobs_handler", "ERR_INVALID_ARGUMENT");
        return HttpResponseBuilder::new(StatusCode::NOT_FOUND)
            .json(ErrorResponse::new(1, "ERR_INVALID_ARGUMENT"));
    }
    job_data_inner.job_list.push(job);
    job_data_inner.total_jobs += 1;
    let res = job_data_inner.job_list.last_mut().unwrap().run(&config);
    println!("{}", serde_json::to_string_pretty(&res).unwrap());
    log::info!(target: "post_jobs_handler", "job run success");
    return HttpResponse::Ok().json(res);
}

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

#[get("/jobs")]
pub async fn get_jobs(query: web::Query<UrlQuery>) -> impl Responder {
    let query = query.into_inner();
    let job_data = JOBDATA.clone();
    let job_data_inner = job_data.lock().unwrap();
    let mut temp_job_list: Vec<&Job> = job_data_inner.job_list.iter().collect();
    temp_job_list.sort_by_key(|x| {x.created_time});

    let res: Vec<Response> = temp_job_list.iter().filter(|job| {
        query.predicate(job)
    }).map(|x| {
        x.response()
    }).collect();
    drop(job_data_inner);
    return HttpResponse::Ok().json(res);
}

#[get("/jobs/{jobid}")]
pub async fn get_jobs_id(jobid: web::Path<u32>) -> impl Responder {
    let job_data = JOBDATA.clone();
    let job_data_inner = job_data.lock().unwrap();
    let response = job_data_inner.job_list.iter().find(|x| {
        x.job_id==*jobid
    }).map(|x| {
        x.response()
    });
    if response.is_none() {
        return HttpResponseBuilder::new(StatusCode::NOT_FOUND)
            .reason( "Job 123456 not found." )
            .json(ErrorResponse::new(3, "ERR_NOT_FOUND"));
    }
    return HttpResponse::Ok().json(response.unwrap());
}
#[put("/jobs/{jobid}")]
pub async fn put_job(jobid: web::Path<u32>, config: web::Data<Config>) -> impl Responder {
    let job_data = JOBDATA.clone();
    let mut job_data_inner = job_data.lock().unwrap();
    let job = job_data_inner.job_list.iter_mut().find(|x| {
        x.job_id==*jobid
    });
    if job.is_none() {
        return HttpResponseBuilder::new(StatusCode::NOT_FOUND)
            .reason( "Job 123456 not found." )
            .json(ErrorResponse::new(3, "ERR_NOT_FOUND"));
    }
    let job = job.unwrap();
    if job.state != State::Finished {
        return HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
            .reason( "Job 123456 not finished." )
            .json(ErrorResponse::new(2, "ERR_INVALID_STATE"));
    }
    job.run(&config);

    return HttpResponse::Ok().json(job.response());
}

#[delete("/jobs/{jobid}")]
pub async fn delete_job(jobid: web::Path<u32>, config: web::Data<Config>) -> impl Responder {
    let job_data = JOBDATA.clone();
    let mut job_data_inner = job_data.lock().unwrap();
    let job = job_data_inner.job_list.iter_mut().find(|x| {
        x.job_id==*jobid
    });
    if job.is_none() {
        return HttpResponseBuilder::new(StatusCode::NOT_FOUND)
            .reason( "Job 123456 not found." )
            .json(ErrorResponse::new(3, "ERR_NOT_FOUND"));
    }
    let job = job.unwrap();
    if job.state != State::Queueing {
        return HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
            .reason( "Job 123456 not queuing." )
            .json(ErrorResponse::new(2, "ERR_INVALID_STATE"));
    }
    job.run(&config);
    job.state = State::Canceled;

    return HttpResponse::Ok().finish();
}

