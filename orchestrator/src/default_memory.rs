use crate::prelude::*;

use async_trait::async_trait;
use chrono::Local;
use rand::{distributions::Alphanumeric, Rng};
use std::{
    any::TypeId,
    cell::RefCell,
    collections::{HashMap, HashSet},
    error::Error as StdError,
};
use tokio::sync::Mutex;

struct InnerMemory {
    id: i64,
    users: HashMap<String, User<Unauthenticated>>,
    /// name, (variant, source)
    exercises: HashMap<String, (String, String)>,
    /// from, (into, data)
    activated_executors: HashMap<String, (String, String)>,
    /// submission:
    /// access by id (usize), user_id, problem_name, source, Option<Result>
    submissions: Vec<(i64, String, String, Option<ExerciseResult>)>,
}

/// MUST be used only for testing, not recomended in production
///
/// saves password as it is, and doesn't do any sanification...
///
/// in addition to that all users are admin by default
pub struct DefaultMemory {
    inner: Mutex<RefCell<InnerMemory>>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("already present")]
    AlreadyPresent,
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthoraized,
    #[error("cycle detected")]
    CycleDetected,
}

impl DefaultMemory {
    pub fn init<S: ExecutorGlobalState>() -> Box<dyn Memory<S>> {
        Box::new(Self {
            inner: Mutex::new(RefCell::new(InnerMemory {
                id: 0,
                users: HashMap::new(),
                exercises: HashMap::new(),
                activated_executors: HashMap::new(),
                submissions: Vec::new(),
            })),
        })
    }
}
/// generate a new random token, it has a lenght of 20 alphanumeric characters
pub fn new_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(20)
        .collect()
}
#[async_trait]
impl StatelessMemory for DefaultMemory {
    async fn register(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User<Unauthenticated>, Box<dyn StdError>> {
        let mut binding = self.inner.lock().await;
        let inner = binding.get_mut();
        if inner.users.contains_key(username) {
            Err(Error::AlreadyPresent)?;
        }
        let user_id = inner.id;
        inner.id += 1;

        let user = User {
            ph: std::marker::PhantomData,
            user_id,
            username: username.to_string(),
            password_hash: password.to_string(),
            logged_in_time: None,
            logged_in_token: None,
            is_admin: true,
        };
        inner.users.insert(username.to_string(), user.clone());
        Ok(user)
    }

    //tries to login the current user, returning an authenticated user instance
    async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User<Authenticated>, Box<dyn StdError>> {
        let user = self.get_by_username(username).await?;
        if user.password_hash != password {
            Err(Error::Unauthoraized)?
        }
        let mut user = user.clone().transmute();
        user.logged_in_time = Some(Local::now().to_utc());
        user.logged_in_token = Some(new_token());
        let mut binding = self.inner.lock().await;
        let inner = binding.get_mut();
        let _ = inner
            .users
            .insert(user.username.clone(), user.clone().transmute())
            .unwrap();
        Ok(user)
    }

    async fn get_by_username(
        &self,
        username: &str,
    ) -> Result<User<Unauthenticated>, Box<dyn StdError>> {
        let mut binding = self.inner.lock().await;
        let inner = binding.get_mut();
        let user = inner.users.get(username).ok_or(Error::NotFound)?;
        Ok(user.clone())
    }

    async fn get_all_users(&self) -> Result<Vec<User<Unauthenticated>>, Box<dyn StdError>> {
        let mut binding = self.inner.lock().await;
        let inner = binding.get_mut();
        let user: Vec<User<Unauthenticated>> = inner.users.values().cloned().collect();
        Ok(user)
    }
    ///check if the given token is valid, and if so returns the correct user
    async fn get_authenticate(
        &self,
        token: &str,
    ) -> Result<User<Authenticated>, Box<dyn StdError>> {
        let mut binding = self.inner.lock().await;
        let inner = binding.get_mut();
        //let token = token.to_string();
        let user = inner
            .users
            .values()
            .filter_map(|x| {
                if let Some(t) = &x.logged_in_token {
                    if t == token {
                        Some(x.clone().transmute())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .next()
            .ok_or(Error::Unauthoraized)?;
        Ok(user)
    }

    async fn get_admin(&self, token: &str) -> Result<User<Admin>, Box<dyn StdError>> {
        let user = self.get_authenticate(token).await?;
        if user.is_admin {
            Ok(user.transmute())
        } else {
            Err(Error::Unauthoraized.into())
        }
    }

    async fn list_exercise_names(&self) -> Result<Vec<String>, Box<dyn StdError>> {
        let mut lock = self.inner.lock().await;
        Ok(lock.get_mut().exercises.keys().cloned().collect())
    }

    ///add submission (on success returns submission id)
    async fn add_submission(
        &self,
        exercise_name: String,
        source: String,
        user: User<Authenticated>,
    ) -> Result<i64, Box<dyn StdError + Send + Sync>> {
        let mut lock = self.inner.lock().await;
        if !lock.get_mut().exercises.contains_key(&exercise_name) {
            Err("unknown problem")?
        }
        let submissions = &mut lock.get_mut().submissions;

        submissions.push((user.user_id, exercise_name, source, None));
        let len = submissions.len() - 1;
        Ok(len as i64)
    }

    ///add exercise result
    async fn add_exercise_result(
        &self,
        submission_id: i64,
        user: User<Authenticated>,
        result: ExerciseResult,
    ) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let mut lock = self.inner.lock().await;
        let submission = lock
            .get_mut()
            .submissions
            .get_mut(submission_id as usize)
            .ok_or(format!("invalid submission id ({})", submission_id).as_str())?;
        if submission.0 != user.user_id {
            Err("incorrect user id")?
        }
        submission.3 = Some(result);
        Ok(())
    }
}

#[async_trait]
impl<S: ExecutorGlobalState> StateMemory<S> for DefaultMemory {
    /// enable an executor. It should be already checked if it is a valid executor or not
    /// checks if adding this element creates a cycle or not
    async fn enable_executor(
        &self,
        input: &S,
        output: &S,
        data: String,
    ) -> Result<(), Box<dyn StdError + Send + Sync + 'static>> {
        let mut lock = self.inner.lock().await;
        //get all correspondence
        let t = &mut lock.get_mut().activated_executors;
        let inp = input.serialize_variant();
        let out = output.serialize_variant();

        let mut copy = t.clone();
        copy.insert(inp.clone(), (out.clone(), data.clone()));
        let map: HashMap<String, String> = copy
            .into_iter()
            .map(|(from, (into, _))| (from, into))
            .collect();
        // if adding this element we get a cycle, we return an error
        if has_cycles(&map) {
            return Err(Error::CycleDetected.into());
        }
        t.insert(inp, (out, data));
        Ok(())
    }
    async fn get_execution_plan(
        &self,
        input: &S,
    ) -> Result<Vec<(TypeId, TypeId, String)>, Box<dyn StdError + Send + Sync + 'static>> {
        let mut lock = self.inner.lock().await;
        let t = &lock.get_mut().activated_executors;
        let mut cur = input.serialize_variant();
        let mut ret = Vec::new();
        while let Some((next, data)) = t.get(&cur) {
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
        let ex_type = exercise_type.serialize_variant();
        let mut lock = self.inner.lock().await;
        lock.get_mut().exercises.insert(name, (ex_type, source));
        Ok(())
    }

    /// get an exercise from memory
    /// type, source
    async fn get_exercise(
        &self,
        name: String,
    ) -> Result<(TypeId, String), Box<dyn StdError + Send + Sync + 'static>> {
        let mut lock = self.inner.lock().await;
        let (ty, source) = lock
            .get_mut()
            .exercises
            .get(&name)
            .ok_or("not found in memory")?;
        let ty = S::deserialize_variant(ty)?;
        Ok((ty, source.clone()))
    }
}

pub fn has_cycles(vertex: &HashMap<String, String>) -> bool {
    let nodes: Vec<&String> = vertex
        .iter()
        .flat_map(|(a, b)| vec![a, b].into_iter())
        .collect();
    let mut checked: HashSet<&String> = HashSet::new();
    for mut cur in nodes {
        let mut visited = HashSet::new();
        visited.insert(cur);
        while let Some(next) = vertex.get(cur) {
            //already checked
            if checked.contains(next) {
                break;
            }
            //check for cycle
            if visited.contains(next) {
                return true;
            }
            visited.insert(next);
            cur = next;
        }
        checked.extend(visited);
    }
    false
}
#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::has_cycles;

    #[test]
    fn test_has_cycles() {
        let mut t = HashMap::new();
        t.insert("1", "2");
        t.insert("2", "3");
        t.insert("4", "3");
        t.insert("5", "6");
        let to: HashMap<String, String> = t
            .clone()
            .into_iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        assert_eq!(has_cycles(&to), false);
        t.insert("6", "2");
        t.insert("3", "4");
        let to: HashMap<String, String> = t
            .clone()
            .into_iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        assert_eq!(has_cycles(&to), true);
    }
}
