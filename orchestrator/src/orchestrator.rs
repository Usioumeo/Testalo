//! This is the main module, and contains the definition of the orchestrator
use std::{
    any::TypeId, collections::HashMap, error::Error, future::Future, marker::PhantomData, mem,
    ops::Deref, pin::Pin, sync::Arc,
};

use crate::prelude::*;
use async_trait::async_trait;
use tokio::sync::{Notify, Semaphore};

#[derive(Debug, thiserror::Error)]
/// Possible errors returned from the orchestrator
pub enum OrchestratorError {
    /// It was not possible to found the requested resource
    #[error("NotFound")]
    NotFound,
    /// Failed Authentication
    #[error("Failing Autentication: {0}")]
    FailingAutentication(ExerciseResult),

    /// Execution Error
    #[error("Execution Error: {0}")]
    ExecutionError(#[from] Box<dyn Error + Send + Sync>),
}

/// which result should a complete execution return?
pub type ResultOutput = Result<ExerciseResult, Box<dyn Error>>;
/// wrap ResultOutput in a dynamic Future
pub type ResultFuture = Pin<Box<dyn Send + Sync + Future<Output = ResultOutput>>>;

/// Type of a dynamic Function that returns a ResultFuture. It takes as input an ExerciseDefinition and a String
pub type Func = dyn Send + Sync + Fn(&dyn ExerciseDef, String) -> ResultFuture;

/// What does an exercise generator need to return?
pub type ExerciseGeneratorFuture<S> =
    Pin<Box<dyn Send + Sync + Future<Output = Result<S, Box<dyn Error + Send + Sync + 'static>>>>>;
/// Dyncamic function, it returns an ExerciseGeneratorFuture
pub type ExerciseGenerator<S> = Box<dyn Send + Sync + Fn(String) -> ExerciseGeneratorFuture<S>>;

/// Add user source code to the ExerciseDef
pub type UserSrcAdder<S> = Box<dyn Send + Sync + Fn(S, String) -> ExerciseGeneratorFuture<S>>;

/// Which error should the implementation return?
pub type DynError = Box<dyn Error + Send + Sync>;

/// The main struct, it orchestrates all plugins, executors, memory ecc...
pub struct Orchestrator<S: ExecutorGlobalState> {
    ph: PhantomData<S>,
    memory: Box<dyn Memory<S>>,
    /// save all executors
    pub executors: HashMap<(TypeId, TypeId), Executor<S>>,
    /// executor generator saved
    pub exercise_generators: HashMap<TypeId, (ExerciseGenerator<S>, UserSrcAdder<S>)>,
    /// saved plugin, runned with run method
    plugins: Vec<Box<dyn InnerPlugin<S>>>,
    /// semaphore to keep track of concurrent exercise execution
    execution_semaphore: Semaphore,
}

impl<S: ExecutorGlobalState> Orchestrator<S> {
    /// Constructor, it takes as input the total number of permits available for execution and a Memory
    pub fn new(execution_permits: usize, memory: Box<dyn Memory<S>>) -> Self {
        Orchestrator {
            ph: PhantomData,
            executors: HashMap::new(),
            exercise_generators: HashMap::new(),
            memory,
            plugins: Vec::new(),
            execution_semaphore: Semaphore::new(execution_permits),
        }
    }
}
impl<S: ExecutorGlobalState> Orchestrator<S> {
    /// process the given exercise (name), and deliver the source (s). it gives back an ExerciseResult if all is gone well
    pub async fn process_exercise(
        &self,
        name: String,
        source: String,
        user: User<Authenticated>,
    ) -> Result<ExerciseResult, DynError> {
        let id = self
            .memory
            .add_submission(name.clone(), source.clone(), user.clone())
            .await?;
        let lock = self.execution_semaphore.acquire().await?;
        let generated = self
            .generate_exercise(name.to_string(), source.to_string())
            .await?;
        let final_state = self.run_state(generated).await?;
        let result: ExerciseResult =
            TryInto::try_into(final_state).map_err(|_| "wrong result returned")?;
        mem::drop(lock);
        self.memory
            .add_exercise_result(id, user.clone(), result.clone())
            .await?;
        Ok(result)
    }

    /// add exercise,
    /// Then tries to do a normal execution, and check if it does indeed return full score
    /// if not returns an error (and obviusly doesn't add it)
    pub async fn add_exercise<ExerciseType: ExerciseDef + ExecutorState>(
        &mut self,
        name: &str,
        source: &str,
    ) -> Result<(), DynError> {
        let (generator, src_adder) = self
            .exercise_generators
            .get(&TypeId::of::<ExerciseType>())
            .ok_or(OrchestratorError::NotFound)?;
        //test
        let exercise_def = generator(source.to_string()).await?;
        let exercise_with_solution = src_adder(exercise_def.clone(), source.to_string()).await?;
        let results = self.run_state(exercise_with_solution).await?;
        let results: ExerciseResult = results
            .try_into()
            .map_err(|_| "not found an exercise result")?;

        //check if we get all the points
        let all_ok = results
            .tests
            .values()
            .all(|x| x.compiled == CompilationResult::Built && x.runned == RunResult::Ok);
        if !all_ok {
            Err(format!(
                "can't get all the points. Returned this result {:?}",
                results
            ))?
        }
        self.memory
            .add_exercise(name.to_string(), exercise_def, source.to_string())
            .await?;
        Ok(())
    }
    ///get and execute plan
    pub async fn run_state(&self, mut cur: S) -> Result<S, DynError> {
        let plan = self.memory.get_execution_plan(&cur).await?;
        for (from, to, data) in plan {
            let func = self
                .executors
                .get(&(from, to))
                .ok_or("executor not registered")?;
            cur = func(cur, data).await?;
        }
        Ok(cur)
    }

    /// adds an exercise generator to the orchestrator
    ///
    /// NB: it does not check if it's correct or not
    /// if a generator is already present it gets overriten

    pub async fn add_exercise_generators<Definition, DefinitionWithSource, F, F2>(
        &mut self,
        exercise_gen: fn(String) -> F,
        source_add: fn(Definition, String) -> F2,
    ) where
        Definition: ExecutorState + ExerciseDef + Into<S> + TryFrom<S>,
        DefinitionWithSource: ExecutorState + Into<S>,
        F: Future<Output = Result<Definition, Box<dyn Error + Send + Sync + 'static>>>
            + 'static
            + Send
            + Sync,
        F2: Future<Output = Result<DefinitionWithSource, Box<dyn Error + Send + Sync + 'static>>>
            + 'static
            + Send
            + Sync,
    {
        // wrap in a generic function
        let exercise_gen = move |template: String| {
            let t: ExerciseGeneratorFuture<S> = Box::pin(async move {
                let ret = exercise_gen(template).await?;
                let ret: S = ret.into();
                Ok::<S, Box<dyn Error + Send + Sync + 'static>>(ret)
            });
            t
        };
        let source_add = move |definition: S, source: String| {
            let t: ExerciseGeneratorFuture<S> = Box::pin(async move {
                let definition = <S as TryInto<Definition>>::try_into(definition)
                    .map_err(|_| "not a valid input")?;
                let ret = source_add(definition, source).await?;
                let ret: S = ret.into();
                Ok::<S, Box<dyn Error + Send + Sync + 'static>>(ret)
            });
            t
        };

        self.exercise_generators.insert(
            TypeId::of::<Definition>(),
            (Box::new(exercise_gen), Box::new(source_add)),
        );
    }
    /// generate an exercise from a name and a source-code
    async fn generate_exercise(&self, name: String, source: String) -> Result<S, DynError> {
        let (ty, template) = self.memory.get_exercise(name).await?;
        let (generator, source_adder) = self.exercise_generators.get(&ty).ok_or("not found")?;
        let generated = generator(template).await?;
        let added = source_adder(generated, source).await?;
        Ok(added)
    }

    /// Adds a plugin to the orchestrator
    pub async fn add_plugin<P: Plugin<S> + 'static>(&mut self, mut p: P) -> Result<(), DynError> {
        p.on_add(self).await?;
        let to_push = Box::new(PluginStorage::new(p));
        self.plugins.push(to_push);
        Ok(())
    }

    /// Runs the Orchestrator.
    pub async fn run(mut self) -> OrchestratorReference<S> {
        let mut to_run = Vec::new();
        mem::swap(&mut to_run, &mut self.plugins);
        let o = self.as_ref();
        let n = Arc::new(Notify::new());
        for mut cur in to_run {
            let o = o.clone();
            let n = n.clone();
            let to_run = async move {
                cur.run(o.clone(), n).await.unwrap();
            };
            tokio::spawn(to_run);
        }
        n.notified().await;
        o
    }
    /// get a reference to the internal memory
    pub fn memory(&self) -> &dyn Memory<S> {
        self.memory.as_ref()
    }

    /// Enables a particular executor
    pub async fn enable_executor<
        Input: ExecutorState + TryFrom<S> + Into<S>,
        Output: ExecutorState + Into<S>,
        Data: Serialize,
    >(
        &mut self,
        data: Data,
    ) -> Result<(), DynError> {
        use crate::executor::AddExecutor;
        self.enable_executor_typed(
            &Input::async_default().await,
            &Output::async_default().await,
            data,
        )
        .await?;
        Ok(())
    }
}


#[derive(Clone)]
/// A shared reference to the orchestrator
pub struct OrchestratorReference<S: ExecutorGlobalState> {
    inner: Arc<Orchestrator<S>>,
}
impl<S: ExecutorGlobalState> Deref for OrchestratorReference<S> {
    type Target = Orchestrator<S>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<S: ExecutorGlobalState> Orchestrator<S> {
    /// returns a reference to the orchestrator
    pub fn as_ref(self) -> OrchestratorReference<S> {
        OrchestratorReference {
            inner: Arc::new(self),
        }
    }
}
#[async_trait]
/// Reference without state
pub trait ReferenceWithoutState: Send + Sync + 'static {
    /// from exercise name, source string, and user authenticated
    async fn process_exercise(
        &self,
        name: String,
        s: String,
        user: User<Authenticated>,
    ) -> Result<ExerciseResult, DynError>;
    /// returns a memory reference (without state)
    fn memory(&self) -> &dyn StatelessMemory;
    //fn deref(&self) -> &Orchestrator<impl ExecutorState>;
}
#[async_trait]
impl<S: ExecutorGlobalState> ReferenceWithoutState for OrchestratorReference<S> {
    /*fn add_plugin<P: Plugin + 'static>(&mut self, p: P) {
        todo!()
    }*/
    fn memory(&self) -> &dyn StatelessMemory {
        self.memory.as_stateless()
    }

    async fn process_exercise(
        &self,
        name: String,
        s: String,
        user: User<Authenticated>,
    ) -> Result<ExerciseResult, DynError> {
        Ok(self.inner.process_exercise(name, s, user).await?)
    }
}


#[cfg(test)]
mod tests {

    use crate as orchestrator;
    use crate::{
        prelude::{Orchestrator, OrchestratorReference},
        GenerateState,
    };
    GenerateState!(ExerciseResult);

    #[test]
    fn test_syncness() {
        fn is_sync<T: Sync>() {}
        fn is_send<T: Send>() {}
        is_sync::<Orchestrator<State>>();
        is_send::<&Orchestrator<State>>();
        is_sync::<&Orchestrator<State>>();
        is_send::<OrchestratorReference<State>>();
        is_sync::<OrchestratorReference<State>>();
    }
}