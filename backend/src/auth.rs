#![allow(clippy::blocks_in_conditions)]
use orchestrator::{
    memory::{Admin, Authenticated, UserState},
    orchestrator::ReferenceWithoutState,
};
use std::error::Error as StdError;

use rocket::{
    form::Form,
    http::{CookieJar, Status},
    post,
    request::{FromRequest, Outcome},
    route, routes,
    tokio::task::JoinError,
    FromForm, Request, State,
};

#[derive(thiserror::Error, Debug)]
///authentication error
pub enum Error {
    #[error("token not found")]
    TokenNotFound,
    #[error("Join Error {0}")]
    TokioJoin(#[from] JoinError),
    #[error("std error")]
    Std(#[from] Box<dyn StdError>),
}

///new type for wrapping Inner User and implementing FromRequest of Rocket
#[allow(dead_code)]
pub struct User<S: UserState> {
    pub inner: orchestrator::memory::User<S>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User<Authenticated> {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Outcome::Success(reference) = request
            .guard::<&State<Box<dyn ReferenceWithoutState>>>()
            .await
        else {
            panic!("config not found")
        };
        let Some(token) = request.cookies().get("auth_token") else {
            return Outcome::Error((Status::NotAcceptable, Error::TokenNotFound));
        };
        match reference.memory().get_authenticate(token.value()).await {
            Ok(user) => Outcome::Success(User { inner: user }),
            Err(err) => Outcome::Error((Status::Forbidden, err.into())),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User<Admin> {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Outcome::Success(reference) = request
            .guard::<&State<Box<dyn ReferenceWithoutState>>>()
            .await
        else {
            panic!("config not found")
        };
        let Some(token) = request.cookies().get("auth_token") else {
            return Outcome::Error((Status::NotAcceptable, Error::TokenNotFound));
        };

        match reference.memory().get_admin(&token.to_string()).await {
            Ok(user) => Outcome::Success(User { inner: user }),
            Err(err) => Outcome::Error((Status::Forbidden, err.into())),
        }
    }
}

#[derive(FromForm)]
/// data required for Login authentication.
///
/// It derives FromForm
struct LoginInfo {
    username: String,
    password: String,
}

#[post("/login", data = "<info>")]
/// login request get's routed here.
///
/// It checks if the credentials are ok, and if so authenticate the user modifying his tokens
async fn login(
    info: Form<LoginInfo>,
    reference: &State<Box<dyn ReferenceWithoutState>>,
    jar: &CookieJar<'_>,
) -> Result<(), Status> {
    //TODO sanitize data
    let user = reference
        .memory()
        .login(&info.username, &info.password)
        .await
        .map_err(|_| Status::Unauthorized)?;

    let token = user.logged_in_token.as_ref().unwrap().clone();
    jar.add(("auth_token", token));
    Ok(())
}

#[post("/register", data = "<info>")]
/// register request get's routed here.
///
/// If the credentials are valid a new user get's registered
async fn register(
    info: Form<LoginInfo>,
    reference: &State<Box<dyn ReferenceWithoutState>>,
) -> Result<(), Status> {
    // TODO mail authorization
    let _user = reference
        .memory()
        .register(&info.username, &info.password)
        .await
        .map_err(|_| Status::UnprocessableEntity)?;
    Ok(())
}

/// function used to route all authentication traffic
pub fn routes() -> Vec<route::Route> {
    routes![login, register]
}
