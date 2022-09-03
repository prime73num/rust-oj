
use actix_web::{
    get, post, web, 
    Responder, 
    HttpResponse, HttpResponseBuilder,
    http::StatusCode
};
use serde::{Serialize, Deserialize};

use crate::{JOBDATA, config, ErrorResponse};

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
pub async fn post_contests(info: web::Json<HttpcomInfo>, config: web::Data<config::Config>) -> impl Responder {
    let job_data = JOBDATA.clone();
    let mut job_data_inner = job_data.lock().unwrap();

    let res = job_data_inner.post_contest(info.into_inner(), &config);
    match res {
        Some(contest) => {
            return HttpResponse::Ok().json(contest);
        },
        None => {
            return HttpResponseBuilder::new(StatusCode::NOT_FOUND)
                .reason("Contest 114514 not found.")
                .json(ErrorResponse::new(3, "ERR_NOT_FOUND"));
        }
    }
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
pub async fn get_contest_id(id: web::Path<u32>) -> impl Responder {
    let job_data = JOBDATA.clone();
    let job_data_inner = job_data.lock().unwrap();
    let res = job_data_inner.contests_list.iter().find(|x| {
        x.0.id == *id
    });

    match res {
        Some(contest) => {
            return HttpResponse::Ok().json(contest.clone());
        },
        None => {
            return HttpResponseBuilder::new(StatusCode::NOT_FOUND)
                .reason("Contest 114514 not found.")
                .json(ErrorResponse::new(3, "ERR_NOT_FOUND"));
        }
    }
}

#[get("/contests/{contestid}/ranklist")]
pub async fn get_contest_ranklist(id: web::Path<u32>) -> impl Responder {
    format!("")
}
