

use actix_web::{
    post, web, 
    Responder, 
    HttpResponse, HttpResponseBuilder,
    http::StatusCode
};
use log;

use crate::config::Config;
use crate::job::{JobInfo, Job};
use crate::JOBDATA;
use crate::ErrorResponse;


#[post("/jobs")]
pub async fn post_jobs(info: web::Json<JobInfo>, config: web::Data<Config>) -> impl Responder {
    let info = info.into_inner();
    let job_data = JOBDATA.clone();
    let mut job_data_inner = job_data.lock().unwrap();
    let id = job_data_inner.total_jobs;
    let job = Job::new(id, info);
    log::info!(target: "post_jobs_handler", "Post jobs");
    if !job.is_valid(&config) {
        log::info!(target: "post_jobs_handler", "ERR_INVALID_ARGUMENT");
        return HttpResponseBuilder::new(StatusCode::NOT_FOUND)
            .json(ErrorResponse::new(1, "ERR_INVALID_ARGUMENT"));
    }
    job_data_inner.job_list.push(job);
    let res = job_data_inner.job_list.last_mut().unwrap().run(&config);
    log::info!(target: "post_jobs_handler", "job run success");
    return HttpResponse::Ok().json(res);
}
