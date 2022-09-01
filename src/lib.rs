
pub mod postapi;
pub mod config;
pub mod job;

use std::sync::{Mutex, Arc};
use lazy_static::lazy_static;


lazy_static!(
    static ref JOBDATA: Arc<JobData> = Arc::default();
);

pub struct JobData {
    job_list: Mutex<Vec<job::Job>>,
    total_jobs: Mutex<u32>,
    user_list: Mutex<Vec<User>>
}

impl Default for JobData{
    fn default() -> Self {
        let job_list: Vec<job::Job> = Vec::new();
        let total_jobs = 0;
        let user_list = vec![User{
            id: 0,
            name: "root".to_string()
        }];
        Self {
            job_list: Mutex::new(job_list),
            total_jobs: Mutex::new(total_jobs),
            user_list: Mutex::new(user_list)
        }
    }
}

pub struct User {
    id: u32,
    name: String
}

