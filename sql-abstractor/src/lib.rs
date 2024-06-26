use helpers::add_test_result;
use orchestrator::default_memory::{has_cycles, new_token};
use orchestrator::executor::ExecutorGlobalState;
use orchestrator::prelude::ExerciseResult;
use std::any::TypeId;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::marker::PhantomData;

use async_trait::async_trait;
use orchestrator::memory::*;
use rand::thread_rng;
use scrypt::password_hash::PasswordHasher;
use scrypt::{
    password_hash::{PasswordHash, PasswordVerifier, SaltString},
    Params, Scrypt,
};
use sqlx::prelude::FromRow;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{query, query_as, Pool};
use tokio::task::{spawn_blocking, JoinError};
mod helpers;

#[cfg(test)]
mod test;

#[derive(thiserror::Error, Debug)]
/// All possible error variants from this implementation
pub enum Error {
    #[error("string")]
    String(String),
    #[error("Hash Error {0}")]
    Hash(#[from] scrypt::password_hash::Error),
    #[error("Sqlx Error {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Join Error {0}")]
    TokioJoin(#[from] JoinError),
    #[error("Unauthoraized")]
    Unauthoraized,
    #[error("Already present")]
    AlreadyPresent,
    #[error("Not found")]
    NotFound,
}

///Postgress implementation.
///
/// It's a wrapper around sqlx::postgres
pub struct Postgres {
    pool: Pool<sqlx::Postgres>,
}

#[derive(FromRow)]
/// Struct used to retrive/set user information:
struct UserWrapper {
    pub user_id: i64,
    pub username: String,
    pub password_hash: String,
    pub logged_in_time: Option<DateTime<Utc>>,
    pub logged_in_token: Option<String>,
    pub is_admin: bool,
}
#[derive(FromRow)]
struct Problem {
    name: String,
    ty: String,
    source: String,
}

#[derive(FromRow)]
struct Enabled {
    incoming: String,
    outgoing: String,
    additional_data: String,
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

impl Postgres {
    /// initialize connector
    pub async fn init(builder: &str) -> Result<Self, Error> {
        let pool: Pool<sqlx::Postgres> = Pool::connect(builder).await?;
        //create table users (if not present)
        let _ = query(include_str!("sql/create_db/1_users.sql"))
            .execute(&pool)
            .await
            .map_err(|x| Error::String(format!("Error while creating user table {x}")))?;
        //create table problems (if not present)
        let _ = query(include_str!("sql/create_db/2_problems.sql"))
            .execute(&pool)
            .await
            .map_err(|x| Error::String(format!("Error while creating problems table {x}")))?;

        //create table problems (if not present)
        let _ = query(include_str!("sql/create_db/3_enabled_executors.sql"))
            .execute(&pool)
            .await
            .map_err(|x| {
                Error::String(format!("Error while creating enabled_executors table {x}"))
            })?;

        //create table submissions (if not present)
        let _ = query(include_str!("sql/create_db/4_submissions.sql"))
            .execute(&pool)
            .await
            .map_err(|x| Error::String(format!("Error while creating submission table {x}")))?;

        //create table test_result (if not present)
        let _ = query(include_str!("sql/create_db/5_test_results.sql"))
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }
    /// WARNING: THIS WILL ERASE ALL THE DATA CONTAINED IN THE DATABASE, AND THEN INIT
    pub async fn clean_init(builder: &str) -> Result<Self, Error> {
        let pool: Pool<sqlx::Postgres> = Pool::connect(builder).await?;

        let _ = query("DROP TABLE test_results").execute(&pool).await;
        let _ = query("DROP TABLE submissions").execute(&pool).await;
        let _ = query("DROP TABLE users").execute(&pool).await;
        let _ = query("DROP TABLE problems").execute(&pool).await;
        let _ = query("DROP TABLE enabled_executors").execute(&pool).await;

        Self::init(builder).await
    }
}
#[async_trait]
impl StatelessMemory for Postgres {
    /// register user inside db
    async fn register(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User<Unauthenticated>, Box<dyn StdError>> {
        //Hash password, it's cpu expensive, so it's better execute it in a blocking way
        let password = password.to_string();
        let hash = spawn_blocking(move || {
            let salt = SaltString::generate(thread_rng());
            let params = Params::new(
                10,
                Params::RECOMMENDED_R,
                Params::RECOMMENDED_P,
                Params::RECOMMENDED_LEN,
            )
            .unwrap();
            Scrypt
                .hash_password_customized(password.as_bytes(), None, None, params, &salt)
                .map(|x| x.to_string())
        })
        .await??;
        //insert new user
        query("INSERT INTO users(username, password_hash, is_admin) VALUES ($1, $2, false) RETURNING user_id")
            .bind(username)
            .bind(&hash)
            .execute(&self.pool)
            .await?;
        self.get_by_username(username).await
    }

    /// tries to login the current user, returning an authenticated user instance
    async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User<Authenticated>, Box<dyn StdError>> {
        let user = self.get_by_username(username).await?;
        let hash = user.password_hash.clone();
        let password = password.to_string();
        //check password
        spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&hash)?;
            Scrypt.verify_password(password.as_bytes(), &parsed_hash)
        })
        .await??;

        //update token
        let token = new_token();
        let user: User<Authenticated> = query_as::<sqlx::Postgres, UserWrapper>("UPDATE users SET logged_in_time=NOW(), logged_in_token=$1 WHERE user_id=$2 RETURNING *")
            .bind(token)
            .bind(user.user_id)
            .fetch_one(&self.pool)
            .await?.into();
        Ok(user)
    }
    /// gets a user by his username
    async fn get_by_username(
        &self,
        username: &str,
    ) -> Result<User<Unauthenticated>, Box<dyn StdError>> {
        Ok(
            query_as::<sqlx::Postgres, UserWrapper>(
                "SELECT * FROM users WHERE users.username = $1",
            )
            .bind(username)
            .fetch_one(&self.pool)
            .await?
            .into(),
        )
    }

    ///returns all user present in the DB.
    async fn get_all_users(&self) -> Result<Vec<User<Unauthenticated>>, Box<dyn StdError>> {
        Ok(
            query_as::<sqlx::Postgres, UserWrapper>("SELECT * FROM users")
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|x| x.into())
                .collect(),
        )
    }
    /// check if the given token is valid, and if so returns the correct user
    async fn get_authenticate(
        &self,
        token: &str,
    ) -> Result<User<Authenticated>, Box<dyn StdError>> {
        Ok(sqlx::query_as::<sqlx::Postgres, UserWrapper>(
            "SELECT * FROM users WHERE logged_in_token = $1",
        )
        .bind(token.to_string())
        .fetch_one(&self.pool)
        .await?
        .into())
    }

    /// if the token is valid, it get's the user, and if so checks if it is an Admin
    async fn get_admin(&self, token: &str) -> Result<User<Admin>, Box<dyn StdError>> {
        let user = self.get_authenticate(token).await?;
        if user.is_admin {
            Ok(user.transmute())
        } else {
            Err(Error::Unauthoraized.into())
        }
    }
    async fn list_exercise_names(&self) -> Result<Vec<String>, Box<dyn StdError>> {
        let data: Vec<Problem> =
            sqlx::query_as::<sqlx::Postgres, Problem>("SELECT * FROM problems")
                .fetch_all(&self.pool)
                .await?;
        let res = data.into_iter().map(|x| x.name).collect();
        Ok(res)
    }
    ///add submission (on success returns submission id)
    async fn add_submission(
        &self,
        exercise_name: String,
        source: String,
        user: User<Authenticated>,
    ) -> Result<i64, Box<dyn StdError + Send + Sync>> {
        let id: (i64,) = query_as(
            "INSERT INTO submissions(user_id, name, source) VALUES ($1, $2, $3) RETURNING submission_id",
        )
        .bind(user.user_id as i32)
        .bind(exercise_name)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        Ok(id.0 as i64)
    }

    ///add exercise result
    async fn add_exercise_result(
        &self,
        submission_id: i64,
        user: User<Authenticated>,
        result: ExerciseResult,
    ) -> Result<(), Box<dyn StdError + Send + Sync>> {
        //check if the user owns the current

        for (name, c) in result.tests {
            add_test_result(&self.pool, name, c, submission_id).await?;
        }
        //println!("{}", submission_id);
        let _ = sqlx::query_as::<sqlx::Postgres, (i32,)>(
            "SELECT user_id FROM submissions WHERE submission_id=$1 AND user_id=$2",
        )
        .bind(submission_id)
        .bind(user.user_id)
        .fetch_one(&self.pool)
        .await?;
        //TODO ADD EXERCISE RESULT IN SQL
        Ok(())
    }
}

#[async_trait]
impl<S: ExecutorGlobalState> StateMemory<S> for Postgres {
    async fn enable_executor(
        &self,
        input: &S,
        output: &S,
        data: String,
    ) -> Result<(), Box<dyn StdError + Send + Sync + 'static>> {
        let enabled: Vec<Enabled> =
            sqlx::query_as::<sqlx::Postgres, Enabled>("SELECT * FROM enabled_executors")
                .fetch_all(&self.pool)
                .await?;
        let mut temp: HashMap<String, String> = enabled
            .iter()
            .map(|x| (x.incoming.clone(), x.outgoing.clone()))
            .collect();
        let input = input.serialize_variant();
        let output = output.serialize_variant();
        temp.insert(input.clone(), output.clone());
        if has_cycles(&temp) {
            Err("cycle detected")?
        }
        query("INSERT INTO enabled_executors(incoming, outgoing, additional_data) VALUES ($1, $2, $3)")
            .bind(input)
            .bind(output)
            .bind(data)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_execution_plan(
        &self,
        input: &S,
    ) -> Result<Vec<(TypeId, TypeId, String)>, Box<dyn StdError + Send + Sync + 'static>> {
        let enabled: Vec<Enabled> =
            sqlx::query_as::<sqlx::Postgres, Enabled>("SELECT * FROM enabled_executors")
                .fetch_all(&self.pool)
                .await?;
        let enabled: HashMap<String, (String, String)> = enabled
            .into_iter()
            .map(|x| (x.incoming, (x.outgoing, x.additional_data)))
            .collect();
        let mut cur = input.serialize_variant();
        let mut ret = Vec::new();
        while let Some((next, data)) = enabled.get(&cur) {
            let cur_ty = S::deserialize_variant(&cur).map_err(|_| "Not deserializable")?;
            let next_ty = S::deserialize_variant(next).map_err(|_| "Not deserializable")?;
            ret.push((cur_ty, next_ty, data.clone()));
            cur.clone_from(next);
        }
        Ok(ret)
    }

    /// add an exercise to memory
    async fn add_exercise(
        &self,
        name: String,
        exercise_type: S,
        source: String,
    ) -> Result<(), Box<dyn StdError + Send + Sync + 'static>> {
        let ty = exercise_type.serialize_variant();
        println!("adding {} {}", name, ty);
        query("INSERT INTO problems(name, ty, source) VALUES ($1, $2, $3)")
            .bind(name)
            .bind(ty)
            .bind(source)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// get an exercise from memory
    /// type, source
    async fn get_exercise(
        &self,
        name: String,
    ) -> Result<(TypeId, String), Box<dyn StdError + Send + Sync + 'static>> {
        let data: Problem =
            sqlx::query_as::<sqlx::Postgres, Problem>("SELECT * FROM problems WHERE name = $1")
                .bind(name)
                .fetch_one(&self.pool)
                .await?;
        let ty = S::deserialize_variant(&data.ty)?;

        Ok((ty, data.source))
    }
}
