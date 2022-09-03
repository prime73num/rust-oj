
use actix_web::{
    get, post, web, 
    Responder, 
    HttpResponse, HttpResponseBuilder,
    http::StatusCode
};
use serde::{Serialize, Deserialize};

use crate::{JOBDATA, UserRes, ErrorResponse, User};


#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    id: Option<u32>,
    name: String
}

#[post("/users")]
pub async fn post_users(info: web::Json<UserInfo>) -> impl Responder {
    let job_data = JOBDATA.clone();
    let mut job_data_inner = job_data.lock().unwrap();

    let res: UserRes;
    if info.id.is_none() {
        res = job_data_inner.add_user(&info.name);
    } else {
        res = job_data_inner.update_user(info.id.unwrap(), &info.name);
    }

    log::info!("post users result {:?}", &res);
    match res {
        UserRes::Succecc(u) => {
            return HttpResponse::Ok().json(u);
        },
        UserRes::IdNotFound => {
            return HttpResponseBuilder::new(StatusCode::NOT_FOUND)
                .reason("User 123456 not found.")
                .json(ErrorResponse::new(3, "ERR_NOT_FOUND"));
        },
        UserRes::NameExit => {
            return HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
                .reason("User name already exists")
                .json(ErrorResponse::new(1, "ERR_INVALID_ARGUMENT"));
        },
    }
}

#[get("/users")]
pub async fn get_users() -> impl Responder {
    let job_data = JOBDATA.clone();
    let job_data_inner = job_data.lock().unwrap();
    let mut temp_user_list: Vec<User> = job_data_inner.user_list.iter().map(|x| {x.clone()}).collect();
    temp_user_list.sort_by_key(|x| {x.id});
    return HttpResponse::Ok().json(temp_user_list);
}

