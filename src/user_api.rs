
use actix_web::{
    get, post, web, 
    Responder, 
    HttpResponse,
};
use serde::{Serialize, Deserialize};

use crate::{JOBDATA, User, AppError};


#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Option<u32>,
    pub name: String
}

#[post("/users")]
pub async fn post_users(info: web::Json<UserInfo>) -> Result<HttpResponse, AppError> {
    let job_data = JOBDATA.clone();
    let mut job_data_inner = job_data.lock().unwrap();

    let res = job_data_inner.post_user(info.into_inner())?;

    log::info!("post users result {:?}", &res);

    return Ok(HttpResponse::Ok().json(res));
}

#[get("/users")]
pub async fn get_users() -> impl Responder {
    let job_data = JOBDATA.clone();
    let job_data_inner = job_data.lock().unwrap();
    let mut temp_user_list: Vec<User> = job_data_inner.user_list.iter().map(|x| {x.clone()}).collect();
    temp_user_list.sort_by_key(|x| {x.id});
    return HttpResponse::Ok().json(temp_user_list);
}

