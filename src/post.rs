
use actix_web::{post, web, Responder};
use log;

use serde::{Serialize, Deserialize};


#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Job {
    source_code: String,
    language: String,
    user_id: u32,
    contest_id: u32,
    problem_id: u32
}



#[post("/jobs")]
pub async fn post_jobs(info: web::Json<Job>) -> impl Responder {
    let temp = info.into_inner();
    log::info!(target: "post_jobs_handler", "Post jobs {:?}", temp);
    format!("Post jobs!")
}
