//! This module keeps track of al the Memory related abstraction
//! Memory is considered the layer that keeps track of the exercise, submission, user...
//! It's not important how it is implemented, or how the data are structured. It has to respond to some simple queries in the Memory trait.
//!
//! The Memory trait is actualy formed by two traits: StatelessMemory and StateMemory.
//! This is needed to expose StateMemory only when it is actualy possible to specify the state.
//!
use std::{any::TypeId, error::Error, fmt::Debug, marker::PhantomData};

pub use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::prelude::*;

use private::Privatizer;
mod private {
    /// should remain private, it's needed to privatize what we doesn't want to implement outside this module
    pub trait Privatizer {}
}

/// A valid UserVariant
pub trait UserState: Debug + Privatizer + Clone {}
#[derive(Debug, Clone)]
/// Admin variant, used as a type-state-machine
pub struct Admin;
impl Privatizer for Admin {}
impl UserState for Admin {}

#[derive(Debug, Clone)]
/// Authenticated variant, used as a type-state-machine
pub struct Authenticated;
impl Privatizer for Authenticated {}
impl UserState for Authenticated {}

#[derive(Debug, Clone)]
/// Unauthenticated variant, used as a type-state-machine
pub struct Unauthenticated;
impl Privatizer for Unauthenticated {}
impl UserState for Unauthenticated {}

#[derive(Debug, Clone)]
/// An user, the variant S represent the type of user (if it is Authenticated, Admin, or Unauthenticated).
pub struct User<S: UserState> {
    /// Phantom data used to save the variant
    pub ph: PhantomData<S>,
    /// Univoque identification of a user
    pub user_id: i64,
    /// Univoque Username of a user
    pub username: String,
    /// Hashed password
    pub password_hash: String,
    /// When did it log-in last time?
    pub logged_in_time: Option<DateTime<Utc>>,
    /// Which token should use to authenticate
    pub logged_in_token: Option<String>,
    /// is an admin? Aka can connect to Admin-only parts?
    pub is_admin: bool,
}

impl<S: UserState> User<S> {
    /// WARNING: THIS FUNCTION DOESN'T CHECK IF IS POSSIBLE TO CONVERT, incorrect usage will lead to insecurities
    pub fn transmute<S2: UserState>(self) -> User<S2> {
        User {
            ph: PhantomData,
            user_id: self.user_id,
            username: self.username,
            password_hash: self.password_hash,
            logged_in_time: self.logged_in_time,
            logged_in_token: self.logged_in_token,
            is_admin: self.is_admin,
        }
    }
}

#[async_trait]
/// This is the trait that contains all method of the memory that does not require knowing the state
pub trait StatelessMemory: Sync + Send {
    //USERS

    /// register User, it should not authenticate it
    async fn register(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User<Unauthenticated>, Box<dyn Error>>;

    /// try to log in the relative user
    async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User<Authenticated>, Box<dyn Error>>;

    /// search an user from his username
    async fn get_by_username(
        &self,
        username: &str,
    ) -> Result<User<Unauthenticated>, Box<dyn Error>>;

    /// get the authenticated user from his token
    async fn get_authenticate(&self, token: &str) -> Result<User<Authenticated>, Box<dyn Error>>;

    /// get an authenticated admin from his token
    async fn get_admin(&self, token: &str) -> Result<User<Admin>, Box<dyn Error>>;

    /// prints out all users
    async fn get_all_users(&self) -> Result<Vec<User<Unauthenticated>>, Box<dyn Error>>;

    ///list exercises names
    async fn list_exercise_names(&self) -> Result<Vec<String>, Box<dyn Error>>;

    ///add submission (on success returns submission id)
    async fn add_submission(
        &self,
        exercise_name: String,
        source: String,
        user: User<Authenticated>,
    ) -> Result<i64, Box<dyn Error + Send + Sync>>;

    ///add exercise result
    async fn add_exercise_result(
        &self,
        submission_id: i64,
        user: User<Authenticated>,
        result: ExerciseResult,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}
#[async_trait]
/// This is the trait that contains all method of the memory that does require knowing the state.
/// Is not always available
pub trait StateMemory<S: ExecutorGlobalState> {
    /// used to enable a particular executor
    async fn enable_executor(
        &self,
        input: &S,
        output: &S,
        data: String,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>>;
    /// from a particular state, which executor will be triggered? in which order?
    async fn get_execution_plan(
        &self,
        input: &S,
    ) -> Result<Vec<(TypeId, TypeId, String)>, Box<dyn Error + Send + Sync + 'static>>;

    /// add an exercise to memory
    async fn add_exercise(
        &self,
        name: String,
        exercise_type: S,
        source: String,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>>;

    /// get an exercise from memory
    /// type, source
    async fn get_exercise(
        &self,
        name: String,
    ) -> Result<(TypeId, String), Box<dyn Error + Send + Sync + 'static>>;
}
/// auto trait that rapresent the union of stateless and state Memory
pub trait Memory<S: ExecutorGlobalState>: StateMemory<S> + StatelessMemory {
    /// conversion into a StatelessMemory
    fn as_stateless(&self) -> &dyn StatelessMemory;
}
impl<S: ExecutorGlobalState, Cur: StateMemory<S> + StatelessMemory> Memory<S> for Cur {
    fn as_stateless(&self) -> &dyn StatelessMemory {
        self
    }
}

#[test]
fn test_typesafeness() {
    let _t: Option<Box<dyn StatelessMemory>> = None;
}
