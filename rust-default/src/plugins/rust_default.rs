use crate::*;
use orchestrator::prelude::*;
use std::{error::Error, path::PathBuf};
/// adds normal rust compilation pipeline:
/// from RustGeneratedFile to RustCompiled accept where to save the file as a parameter
pub async fn register_rust_exercise<S>(
    o: &mut Orchestrator<S>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>>
where
    S: ExecutorGlobalState
        + From<RustExercise>
        + From<RustGeneratedFiles>
        + From<RustCompiled>
        + From<ExerciseResult>,
    RustGeneratedFiles: TryFrom<S>,
    RustCompiled: TryFrom<S>,
    RustExercise: TryFrom<S>,
{
    //add executors
    o.add_executor(RustGeneratedFiles::compile, None).await?;

    o.add_executor(|s, _| RustCompiled::run(s), ()).await?;

    // add exercise generators
    let f1 = |c: String| async move {
        let t = RustExercise::parse(&c)?;
        Ok(t)
    };
    let f2 =
        |def: RustExercise, source: String| async move { Ok(def.generate_files(source).await?) };
    o.add_exercise_generators(f1, f2).await;
    Ok(())
}

/// Add all it's needed for a normal rust executor
/// It follows the build pattern
#[derive(Default)]
pub struct RustDefaultPlugin {
    activate_default: bool,
}

impl RustDefaultPlugin {
    /// activate the default implementation (so Rust related executors)
    pub fn set_activate_default(mut self) -> Self {
        self.activate_default = true;
        self
    }
}
impl<S: ExecutorGlobalState> Plugin<S> for RustDefaultPlugin
where
    S: From<RustExercise> + From<RustGeneratedFiles> + From<RustCompiled> + From<ExerciseResult>,
    RustGeneratedFiles: TryFrom<S>,
    RustCompiled: TryFrom<S>,
    RustExercise: TryFrom<S>,
{
    fn name(&self) -> &str {
        "rust default plugin"
    }

    fn desctiption(&self) -> &str {
        "Adds the normal plugins"
    }

    async fn on_add<'a>(
        &'a mut self,
        o: &'a mut Orchestrator<S>,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        register_rust_exercise(o).await?;
        if self.activate_default {
            // enable executors
            o.enable_executor::<RustGeneratedFiles, RustCompiled, _>(None::<PathBuf>)
                .await
                .unwrap();
            o.enable_executor::<RustCompiled, ExerciseResult, _>(())
                .await
                .unwrap();
        }
        //register_docker_rust(o).await?;
        Ok(())
    }
}
