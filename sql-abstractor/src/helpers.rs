use orchestrator::prelude::*;
use sqlx::{
    prelude::*,
    query,
    types::chrono::{DateTime, Utc},
    Pool,
};
use std::{error::Error, marker::PhantomData};

/// This is an helper functions
pub async fn add_test_result(
    pool: &Pool<sqlx::Postgres>,
    name: String,
    result: TestResult,
    submission_id: i64,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let compiled = serde_json::to_string(&result.compiled)?;
    let runned = serde_json::to_string(&result.runned)?;
    query("INSERT INTO test_results(name, compiled, runned, points, refers_to) VALUES ($1, $2, $3, $4, $5)")
        .bind(name)
        .bind(compiled)
        .bind(runned)
        .bind(result.points_given)
        .bind(submission_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(FromRow)]
/// Struct used to retrive/set user information:
pub struct UserWrapper {
    pub user_id: i64,
    pub username: String,
    pub password_hash: String,
    pub logged_in_time: Option<DateTime<Utc>>,
    pub logged_in_token: Option<String>,
    pub is_admin: bool,
}

impl<S: UserState> From<UserWrapper> for User<S> {
    fn from(value: UserWrapper) -> Self {
        //let logged_in_time = value.logged_in_time.map(|x| x.);
        Self {
            ph: PhantomData,
            user_id: value.user_id,
            username: value.username,
            password_hash: value.password_hash,
            logged_in_time: value.logged_in_time,
            logged_in_token: value.logged_in_token,
            is_admin: value.is_admin,
        }
    }
}

#[derive(FromRow)]
/// Internal and private struct, used to parse incoming SQL rows
pub struct Problem {
    pub name: String,
    pub ty: String,
    pub source: String,
}

#[derive(FromRow)]
/// Internal and private struct, used to parse incoming SQL rows
pub struct Enabled {
    pub incoming: String,
    pub outgoing: String,
    pub additional_data: String,
}
