//! Contains Plugin definitions

#[cfg(feature="cli")]
pub(crate) mod cli;
#[cfg(feature="cli")]
pub(crate) mod cli_v2;
pub(crate) mod rust_default;
pub(crate) mod rust_default_v2;
#[cfg(feature="cli")]
pub(crate) mod stateless;

/*//TODO ADD DOCKER
pub async fn register_docker_rust<S>(_o: &mut Orchestrator<S>)->Result<(), Box<dyn Error + Send + Sync + 'static>>
where
    S: ExecutorGlobalState
        + From<RustExercise>
        + From<RustGeneratedFiles>
        + From<RustCompiled>
        + From<ExerciseResult>,
{
    time cargo build --release --timings
    Ok(())
}*/
