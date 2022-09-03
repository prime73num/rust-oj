use std::fs;

use actix_web::{get, middleware::Logger, post, web, App, HttpServer, Responder};
use env_logger;
use log;

use serde_json;
use clap::{Command, arg};


use oj::jobapi;
use oj::userapi;
use oj::config::Config;

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    log::info!(target: "greet_handler", "Greeting {}", name);
    format!("Hello {name}!")
}

// DO NOT REMOVE: used in automatic testing
#[post("/internal/exit")]
#[allow(unreachable_code)]
async fn exit() -> impl Responder {
    log::info!("Shutdown as requested");
    std::process::exit(0);
    format!("Exited")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let cmd = Command::new("cmd")
        .args(&[
            arg!(-c --config <CONFIG> "Specify a config file").required(false),
            arg!(-f --"flush-data" "Fluash data").required(false)
        ]);
    let args = cmd.get_matches();
    let mut file_path = "./config.json";
    if args.contains_id("config") {
        file_path = args.get_one::<String>("config").unwrap();
    }
    let json = fs::read_to_string(file_path).unwrap();
    let config: Config = serde_json::from_str(&json).expect("Parse failed");

    HttpServer::new( move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .route("/hello", web::get().to(|| async { "Hello World!" }))
            .service(greet)
            // DO NOT REMOVE: used in automatic testing
            .service(exit)
            .service(jobapi::post_jobs)
            .service(jobapi::get_jobs)
            .service(jobapi::get_jobs_id)
            .service(jobapi::put_job)
            .service(jobapi::delete_job)
            .service(userapi::post_users)
            .service(userapi::get_users)
    })
    .bind(("127.0.0.1", 12345))?
    .run()
    .await
}
