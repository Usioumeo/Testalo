//! This module contains some unit test for this crate.
//! Other than that contains an abstraction to have modularized test for plugins and extensions.
//!
use std::{error::Error, future::Future, pin::Pin, sync::Arc};

use crate::{
    executor::{AddExecutor, ExecutorGlobalState},
    executor_trait::{ExerciseDef, ExerciseResult, RunResult, TestResult},
    memory::{Authenticated, User},
    orchestrator::{Orchestrator, OrchestratorReference, ReferenceWithoutState},
    plugin::Plugin,
};
use async_trait::async_trait;
use rand::{rngs::OsRng, RngCore};
use tokio::sync::Notify;

/// This is a dummy exercise, and is used in testing, and should not be used outside tests
#[derive(Clone, Default)]
pub struct DummyExercise {}
impl ExerciseDef for DummyExercise {
    fn description(&self) -> &str {
        "not an exercise, but a dummy exercise (used in testing)"
    }

    fn get_generator_src(&self) -> &str {
        ""
    }

    fn list(&self) -> Vec<crate::executor_trait::TestDefinition> {
        Vec::new()
    }
}
/// Generates DummyExercise, it ignores the string input
async fn gen_dummy(_: String) -> Result<DummyExercise, Box<dyn Error + Send + Sync>> {
    Ok(DummyExercise {})
}
/// add some DummyExercise with different names
async fn add_source(
    _: DummyExercise,
    _: String,
) -> Result<ExerciseResult, Box<dyn Error + Send + Sync>> {
    let mut d = ExerciseResult::default();
    use crate::executor_trait::CompilationResult::*;
    let t1 = TestResult {
        compiled: Built,
        runned: RunResult::Ok,
        points_given: 1.0,
    };
    d.tests.insert("test1".to_string(), t1);

    let t2 = TestResult {
        compiled: Built,
        runned: RunResult::Ok,
        points_given: 1.0,
    };
    d.tests.insert("test2".to_string(), t2);

    Ok(d)
}
/// This plugin is used to register DummyExercises.
/// It should be used only in tests
pub struct DummyExercisePlugin;

impl<S> Plugin<S> for DummyExercisePlugin
where
    S: ExecutorGlobalState + From<DummyExercise> + From<ExerciseResult>,
    DummyExercise: TryFrom<S>,
{
    fn name(&self) -> &str {
        "DummyExercisePlugin"
    }

    fn desctiption(&self) -> &str {
        "Adds code for handling dummy exercises"
    }
    async fn on_add<'a>(
        &'a mut self,
        o: &'a mut Orchestrator<S>,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        o.add_exercise_generators(gen_dummy, add_source).await;
        async fn f(
            _: DummyExercise,
            _: (),
        ) -> Result<ExerciseResult, Box<dyn Error + Send + Sync>> {
            Ok(ExerciseResult::default())
        }
        o.add_executor(f, ()).await?;
        o.enable_executor::<DummyExercise, ExerciseResult, _>(())
            .await?;
        o.add_exercise::<DummyExercise>("DummyExercise", "").await
        //Ok(())
    }
}

#[async_trait]
/// TestInterface, it describes how to connect to the orchestrator in order to execute exercises.
/// Normally DefaultInterface is enough for most cases
pub trait TestInterface: Send + Sync {
    /// register a new user, it takes username and password
    async fn register(&mut self, username: &str, password: &str);

    /// login the user with the provided username and password
    async fn login(&mut self, name: &str, password: &str) -> Result<(), Box<dyn Error>>;

    /// submit an exercise
    async fn submit(
        &mut self,
        exercise: String,
        code: String,
    ) -> Result<ExerciseResult, Box<dyn Error + Send + Sync + 'static>>;
    /// list all available exercises
    async fn list_exercise(&mut self) -> Result<Vec<String>, Box<dyn Error + 'static>>;
}

/// The Default Interface, it connects with an OrchestratorReference (taken as every Plugin does).
struct DefaultInterface<S: ExecutorGlobalState> {
    o: OrchestratorReference<S>,
    user: Option<User<Authenticated>>,
}
impl<S: ExecutorGlobalState> DefaultInterface<S> {
    fn new(o: OrchestratorReference<S>) -> Box<Self> {
        Box::new(Self { o, user: None })
    }
}

#[async_trait]
impl<S: ExecutorGlobalState> TestInterface for DefaultInterface<S> {
    async fn register(&mut self, username: &str, password: &str) {
        self.o.memory().register(username, password).await.unwrap();
    }

    async fn login(&mut self, username: &str, password: &str) -> Result<(), Box<dyn Error>> {
        let u = self.o.memory().login(username, password).await?;
        self.user = Some(u);
        Ok(())
    }

    async fn submit(
        &mut self,
        exercise: String,
        code: String,
    ) -> Result<ExerciseResult, Box<dyn Error + Send + Sync + 'static>> {
        self.o
            .process_exercise(
                exercise,
                code,
                self.user.clone().ok_or("Not Authenticated")?,
            )
            .await
    }
    async fn list_exercise(&mut self) -> Result<Vec<String>, Box<dyn Error + 'static>> {
        self.o.memory().list_exercise_names().await
    }
}

/**
function used to test implementations
 - creates an account with a random-name
 - logs in that account
 - execute an exercise, and expect a full score
*/
pub struct DefaultTest {
    t: Option<Box<dyn TestInterface>>,
    ///name source
    es: Option<(String, String)>,
    additional_function: Option<OptionalFunction>,
}
//Pin<Box<dyn Send + Sync + 'static + Future<Output=Result<(), Box<dyn Error + Send + Sync + 'static>>>>>;
type E = Box<dyn Error + Send + Sync + 'static>;
type BoxedFuture = Pin<Box<dyn Send + Future<Output = Result<(), E>>>>;
type OptionalFunction = Box<dyn Send + Sync + Fn(Box<dyn TestInterface>) -> BoxedFuture>;

impl DefaultTest {
    /// create a new Test suite, which connects to the orchestrator with the provided TestInterface
    pub fn new(t: impl TestInterface + 'static) -> Self {
        Self {
            t: Some(Box::new(t)),
            es: None,
            additional_function: None,
        }
    }
    /// this create a new DefaultTest with DefaultTestInterface
    pub fn new_default() -> Self {
        Self {
            t: None,
            es: None,
            additional_function: None,
        }
    }
    /// Use this if you want to test an additional exercise called "name" with the source "code"
    pub fn set_exercise(&mut self, name: String, code: String) {
        self.es = Some((name, code));
    }
    //F: Future<Output = Result<Output, E>> + 'static + Send + Sync,
    //E: Into<Box<dyn StdError + Send + Sync + 'static>> ,
    pub fn set_additional_function<Fut: Send + Future<Output = Result<(), E>> + 'static>(
        &mut self,
        f: impl Send + Sync + 'static + Fn(Box<dyn TestInterface>) -> Fut,
    ) {
        let inner = move |inp: Box<dyn TestInterface>| {
            let t = f(inp);

            let t: Pin<Box<dyn Send + Future<Output = Result<(), E>>>> = Box::pin(t);
            t
        };
        self.additional_function = Some(Box::new(inner));
    }
}

/// private function used to crash tokio when this executor panics
fn override_panic() {
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        orig_hook(panic_info);
        std::process::exit(1);
    }));
}

impl<S> Plugin<S> for DefaultTest
where
    S: ExecutorGlobalState + From<DummyExercise> + From<ExerciseResult>,
    DummyExercise: TryFrom<S>,
{
    fn name(&self) -> &str {
        "tests"
    }

    fn desctiption(&self) -> &str {
        "Runs a common execution pattern"
    }
    async fn run(self, o: OrchestratorReference<S>, should_stop: Arc<Notify>) {
        // this is needed or the panics wouldn't work
        override_panic();

        let mut interface = if let Some(x) = self.t {
            x
        } else {
            DefaultInterface::new(o)
        };
        //fail login on purpose
        let name = format!("Test_{}", OsRng.next_u32());
        let name = name.as_str();
        assert!(interface.login(name, "mondo").await.is_err());

        //register user
        interface.register(name, "mondo").await;
        interface.login(name, "mondo").await.unwrap();

        //submit an exercise that does not exist:
        let x = interface.submit(name.to_string(), String::default()).await;
        assert!(
            x.is_err(),
            "When call an unexisting exercise it doesn't fail"
        );

        // try DummyExercise
        let x = interface
            .submit("DummyExercise".to_string(), "".to_string())
            .await;

        assert!(
            x.is_ok(),
            "Error while computing dummy exercise: {}",
            x.unwrap_err()
        );
        let _ = x.unwrap().to_string();
        //assert!(x.unwrap()>DummyExercise::default()., "invalid error");

        if let Some((name, source)) = self.es {
            let x = interface.submit(name, source).await;
            assert!(x.is_ok(), "Unexpected Error: {}", x.unwrap_err());

            //assert_eq!(interface.list_exercise().await.unwrap().len(), 2);
        } else {
            //assert_eq!(interface.list_exercise().await.unwrap().len(), 1);
        }
        interface.list_exercise().await.unwrap().len();
        if let Some(f) = self.additional_function {
            f(interface).await.unwrap();
        }

        //x.is_ok();
        //assert!(x.is_ok());

        should_stop.notify_one();
    }
    async fn on_add<'a>(
        &'a mut self,
        o: &'a mut Orchestrator<S>,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        o.add_plugin(DummyExercisePlugin).await?;
        Ok(())
    }
}
