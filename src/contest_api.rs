
use actix_web::{
    get, post, web, 
    Responder, 
    HttpResponse,
};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::{JOBDATA, config, AppError, job::JobInfo};

#[derive(Debug, Serialize, Clone)]
pub struct ContestInfo {
    pub id: u32,
    pub name: String,
    pub from: String,
    pub to: String,
    pub problem_ids: Vec<u32>,
    pub user_ids: Vec<u32>,
    pub submission_limit: u32
}

impl ContestInfo {
    pub fn is_valid(&self, jobinfo: &JobInfo) -> bool {
        if !self.problem_ids.iter().any(|x| {
            *x == jobinfo.problem_id
        }) { return false;}
        if !self.user_ids.iter().any(|x| {
            *x == jobinfo.user_id
        }) { return false;}
        let from = DateTime::parse_from_str(&self.from, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
        if from > Utc::now() { return false;}
        let to = DateTime::parse_from_str(&self.to, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
        if to < Utc::now() { return false;}
        return true;
    }
    pub fn from(info: HttpcomInfo) -> Self {
        Self {
            id: info.id.unwrap(),
            name: info.name,
            from: info.from,
            to: info.to,
            problem_ids: info.problem_ids,
            user_ids: info.user_ids,
            submission_limit: info.submission_limit
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpcomInfo {
    pub id: Option<u32>,
    pub name: String,
    pub from: String,
    pub to: String,
    pub problem_ids: Vec<u32>,
    pub user_ids: Vec<u32>,
    pub submission_limit: u32
}

#[post("/contests")]
pub async fn post_contests(info: web::Json<HttpcomInfo>, config: web::Data<config::Config>) -> Result<HttpResponse, AppError> {
    let job_data = JOBDATA.clone();
    let mut job_data_inner = job_data.lock().unwrap();

    let res = job_data_inner.post_contest(info.into_inner(), &config)?;
    return Ok(HttpResponse::Ok().json(res));
}

#[get("/contests")]
pub async fn get_contests() -> impl Responder {
    let job_data = JOBDATA.clone();
    let job_data_inner = job_data.lock().unwrap();

    let mut temp_user_list: Vec<ContestInfo> = job_data_inner.contests_list.iter().map(|x| {x.0.clone()}).collect();
    temp_user_list.sort_by_key(|x| {x.id});
    return HttpResponse::Ok().json(temp_user_list);
}

#[get("/contests/{contestid}")]
pub async fn get_contest_id(id: web::Path<u32>) -> Result<HttpResponse, AppError> {
    let job_data = JOBDATA.clone();
    let job_data_inner = job_data.lock().unwrap();
    let res = job_data_inner.find_contest(*id)?;

    return Ok(HttpResponse::Ok().json(res.0.clone()));
}

#[get("/contests/{contestid}/ranklist")]
pub async fn get_contest_ranklist(id: web::Path<u32>) -> impl Responder {
    format!("")
}
