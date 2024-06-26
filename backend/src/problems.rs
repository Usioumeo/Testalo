// added to make clippy happy, TODO remove it won't be needed anymore
#![allow(clippy::blocks_in_conditions)]
use orchestrator::memory::Authenticated;
use orchestrator::orchestrator::ReferenceWithoutState;
use orchestrator::prelude::serde_json;
use rocket::{form::Form, post, routes, FromForm};
use rocket::{get, serde::json::Json, Route, State};

use crate::auth::User;

#[derive(FromForm)]

/// data required for making a sumbission
///
/// It derives FromForm
struct SubmitInfo {
    problem: String,
    source: String,
}

#[get("/list_problems")]
async fn list_problems(
    reference: &State<Box<dyn ReferenceWithoutState>>,
) -> Option<Json<Vec<String>>> {
    let req = reference.memory().list_exercise_names().await;
    let req = match req {
        Ok(x) => x,
        Err(e) => panic!("{}", e),
    };

    Some(Json(req))
}

#[post("/submit", data = "<submission>")]
async fn submit(
    reference: &State<Box<dyn ReferenceWithoutState>>,
    user: User<Authenticated>,
    submission: Form<SubmitInfo>,
) -> Result<String, String> {
    let res = reference
        .process_exercise(
            submission.problem.clone(),
            submission.source.clone(),
            user.inner,
        )
        .await;
    match res {
        Ok(x) => Ok(serde_json::to_string(&x).map_err(|x| x.to_string())?),
        Err(x) => Err(x.to_string()),
    }
}

/// function used to route all problem- related traffic
pub fn routes() -> Vec<Route> {
    routes![list_problems, submit]
}
