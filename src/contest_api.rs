
use std::cmp::{Ord, Ordering};
use actix_web::{
    get, post, web, 
    Responder, 
    HttpResponse
};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::{JOBDATA, config, AppError, job::{JobInfo, Job}, User};


// this struct represent the json content of the contest http request
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
        if !self.problem_ids.contains(&jobinfo.problem_id) { return false;}
        if !self.user_ids.contains(&jobinfo.user_id) { return false;}
        let from : DateTime<Utc> = self.from.parse().unwrap();
        if from > Utc::now() { return false;}
        let to : DateTime<Utc> = self.to.parse().unwrap();
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

// post a contest
#[post("/contests")]
pub async fn post_contests(info: web::Json<HttpcomInfo>, config: web::Data<config::Config>) -> Result<HttpResponse, AppError> {
    let job_data = JOBDATA.clone();
    let mut job_data_inner = job_data.lock().unwrap();

    let res = job_data_inner.post_contest(info.into_inner(), &config)?;
    log::info!(target: "post_contests", "Post contest {}", res.id);
    return Ok(HttpResponse::Ok().json(res));
}

// get contest list
#[get("/contests")]
pub async fn get_contests() -> impl Responder {
    let job_data = JOBDATA.clone();
    let job_data_inner = job_data.lock().unwrap();

    let mut temp_user_list: Vec<ContestInfo> = job_data_inner.contests_list.iter().map(|x| {x.0.clone()}).collect();
    temp_user_list.sort_by_key(|x| {x.id});
    log::info!(target: "get_contests", "Get contests list");
    return HttpResponse::Ok().json(temp_user_list);
}

// get the contest of id
#[get("/contests/{contestid}")]
pub async fn get_contest_id(id: web::Path<u32>) -> Result<HttpResponse, AppError> {
    let job_data = JOBDATA.clone();
    let job_data_inner = job_data.lock().unwrap();
    let res = job_data_inner.find_contest(*id)?;

    log::info!(target: "get_contest_id", "Get contest {}", res.0.id);
    return Ok(HttpResponse::Ok().json(res.0.clone()));
}

// the ranklist argument
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RanklistArg {
    #[serde(default = "_default_scoring_rule")]
    scoring_rule: Scorerule,
    #[serde(default = "_default_tie_breaker")]
    tie_breaker: Tiebreaderarg
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
enum Scorerule {
    latest,
    highest
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
enum Tiebreaderarg {
    submission_time,
    submission_count,
    user_id,
    none
}
fn _default_scoring_rule() -> Scorerule {
    Scorerule::latest
}
fn _default_tie_breaker() -> Tiebreaderarg {
    Tiebreaderarg::none
}

// the result of a user in the contest
#[derive(Debug, Serialize, Deserialize)]
struct ContestRes {
    user: User,
    rank: u32,
    scores: Vec<f32>
}

impl ContestRes {
    fn new(user: &User, scores: Vec<f32>) -> Self {
        Self {
            user: user.clone(),
            rank: 0,
            scores,
        }
    }
}


// use this key with tie_breaker to sort the contest result
#[derive(Debug)]
struct SortKey {
    flag: Tiebreaderarg,
    submission_time: DateTime<Utc>,
    submission_count: u32,
    user_id: u32,
    total_score: f32,
}

impl SortKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // first compare the total_score
        if self.total_score==other.total_score {
            // use tie_breaker to compare
            match self.flag {
                Tiebreaderarg::submission_time => {
                    return self.submission_time.cmp(&other.submission_time);
                },
                Tiebreaderarg::submission_count => {
                    return self.submission_count.cmp(&other.submission_count);
                },
                Tiebreaderarg::user_id => {
                    return self.user_id.cmp(&other.user_id);
                },
                Tiebreaderarg::none => {
                    return Ordering::Equal;
                },
            }
        }
        return other.total_score.partial_cmp(&self.total_score).unwrap();
    }
}



// get the ranklist 
#[get("/contests/{contestid}/ranklist")]
pub async fn get_contest_ranklist(
    id: web::Path<u32>,
    query: web::Query<RanklistArg>,
    config: web::Data<config::Config>
    ) -> Result<HttpResponse, AppError> {
    let job_data = JOBDATA.clone();
    let job_data_inner = job_data.lock().unwrap();

    let mut user: Vec<u32> = Vec::new();
    let mut problem: Vec<u32> = Vec::new();

    // global contest list
    if *id==0 {
        job_data_inner.user_list.iter().for_each(|x| {
            user.push(x.id);
        });
        config.problems.iter().for_each(|x| {
            problem.push(x.id);
        });
    } else { // specify a contest id
        let contest = job_data_inner.find_contest(*id)?;
        user = contest.0.user_ids.clone();
        problem = contest.0.problem_ids.clone();

    }

    log::info!(target: "get_contest_ranklist", "Get contest {} ranklist", id);

    // a closure the get the score of one user
    let score_rule = |a: &Job, b: &Job| {
        match query.scoring_rule {
            Scorerule::latest => {
                return a.created_time.cmp(&b.created_time);
            },
            Scorerule::highest => {
                if a.score==b.score {
                    return b.created_time.cmp(&a.created_time);
                }
                return a.score.partial_cmp(&b.score).unwrap();
            },
        }
    };


    let mut res: Vec<(ContestRes, SortKey)> = Vec::new();
    for user_id in user.iter() {
        let user = job_data_inner.find_user(*user_id).unwrap();
        let mut score: Vec<f32> = Vec::new();
        let mut total_score = 0.0;
        let mut time: Option<DateTime<Utc>> = None;
        let mut submission_count = 0;
        for problem_id in problem.iter() {

            // find add submission of the user and the problem
            let submission_set = job_data_inner.job_list.iter().filter(|x| {
                x.info.user_id==*user_id && x.info.problem_id==*problem_id && x.info.contest_id == *id
            });
            // use the score_rule to get the result from the submission_set
            let (pro_score, created_time) = submission_set.clone()
            .max_by( |a, b| { score_rule(a, b) } )
            .map_or((0.0, None), |job| {
                (job.score, Some(job.created_time))
            });

            if let Some(update) = created_time {
                match time {
                    Some(latest) => {
                        if update > latest {
                            time = Some(update)
                        }
                    },
                    None => {
                        time = Some(update)
                    }
                }
            }
            submission_count += submission_set.count();
            score.push(pro_score);
            total_score += pro_score;
        }
        // construct the information used to sort and rank
        let time = time.unwrap_or(Utc::now());
        let tie_breaker = SortKey { 
            submission_time: time, 
            submission_count: submission_count as u32, 
            user_id: user.id,
            flag: query.tie_breaker.clone(),
            total_score,
        };
        res.push((ContestRes::new(user, score), tie_breaker));
    }

    res.sort_by(|a, b| {
        // use sort key to sort
        if a.1.cmp(&b.1) == Ordering::Equal {
            return a.1.user_id.cmp(&b.1.user_id);
        }
        return a.1.cmp(&b.1);
    });

    let mut before_rank = 1;
    let mut before_score: Option<SortKey> = None;
    // set the rank
    let res: Vec<ContestRes> = res.into_iter().enumerate()
        .map( |(idx, mut x)| {

            // set the rank to the index in the list
            x.0.rank = (idx + 1) as u32;

            // if have the same sort key with the before user 
            // set the same rank of the before user
            if let Some(before) = &before_score {
                if x.1.cmp(before)==Ordering::Equal {
                    x.0.rank = before_rank;
                }
            }
            before_rank = x.0.rank;
            before_score = Some(x.1);
            x.0
        }).collect();

    return Ok(HttpResponse::Ok().json(res));
}
