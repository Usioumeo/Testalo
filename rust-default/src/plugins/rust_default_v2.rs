use orchestrator::prelude::*;
use std::{error::Error, path::PathBuf};

use crate::prelude::*;
/// adds normal rust compilation pipeline:
/// from RustGeneratedFile to RustCompiled accept where to save the file as a parameter
pub async fn register_rust_exercise<S>(
    o: &mut Orchestrator<S>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>>
where
    S: ExecutorGlobalState
        + From<RustExercise2>
        + From<GeneratedFiles2>
        //+ From<RustCompiled>
        + From<ExerciseResult>,
    //RustGeneratedFiles: TryFrom<S>,
    //RustCompiled: TryFrom<S>,
    RustExercise2: TryFrom<S>,
{
    //add executors
    //o.add_executor(RustGeneratedFiles::compile, None).await?;

    //o.add_executor(|s, _| RustCompiled::run(s), ()).await?;

    // add exercise generators
    let f1 = |c: String| async move {
        let t = RustExercise2::parse(&c)?;
        Ok(t)
    };
    let f2 = |def: RustExercise2, source: String| async move {
        Ok(GeneratedFiles2::generate(def, source)?)
    };
    o.add_exercise_generators(f1, f2).await;
    Ok(())
}

/// Add all it's needed for a normal rust executor
/// It follows the build pattern
#[derive(Default)]
pub struct RustDefaultPlugin2 {
    activate_default: bool,
}

impl RustDefaultPlugin2 {
    /// activate the default implementation (so Rust related executors)
    pub fn set_activate_default(mut self) -> Self {
        self.activate_default = true;
        self
    }
}

impl<S: ExecutorGlobalState> Plugin<S> for RustDefaultPlugin2
where
    S: From<RustExercise2> + From<GeneratedFiles2> + From<RustCompiled2> + From<ExerciseResult>,
    GeneratedFiles2: TryFrom<S>,
    RustCompiled2: TryFrom<S>,
    RustExercise2: TryFrom<S>,
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
            o.enable_executor::<GeneratedFiles2, RustCompiled2, _>(None::<PathBuf>)
                .await
                .unwrap();
            o.enable_executor::<RustCompiled2, ExerciseResult, _>(())
                .await
                .unwrap();
        }
        //register_docker_rust(o).await?;
        Ok(())
    }
}
