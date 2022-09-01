
use actix_web::{post, web, Responder};
use log;

use crate::job::JobInfo;


#[post("/jobs")]
pub async fn post_jobs(info: web::Json<JobInfo>) -> impl Responder {
    let temp = info.into_inner();
    log::info!(target: "post_jobs_handler", "Post jobs {:?}", temp);
    format!("Post jobs!")
}
