use std::fs;

use orchestrator::{default_memory::DefaultMemory, prelude::*, GenerateState};
//use rust_default::{docker::DockerRun, plugins::cli::CLIPlugin, rust_parser::RustExercise};
use rust_default::*;

use tokio::task::JoinSet;

GenerateState!(
    RustExercise,
    RustGeneratedFiles,
    RustCompiled,
    ExerciseResult
);

#[tokio::main]
async fn main() {
    let mut o: Orchestrator<State> = Orchestrator::new(5, true, DefaultMemory::init());

    // ADDING FUNCTIONALITY
    o.add_plugin(RustDefaultPlugin::default().set_activate_default())
        .await
        .unwrap();
    o.add_plugin(Run).await.unwrap();

    // add exercise
    let template = fs::read("template.rs").unwrap();
    let template = String::from_utf8(template).unwrap();
    o.add_exercise::<RustExercise>("template", &template)
        .await
        .unwrap();
    let _o = o.run().await;
}

struct Run;
impl<S: ExecutorGlobalState> Plugin<S> for Run {
    fn name(&self) -> &str {
        "run"
    }

    fn desctiption(&self) -> &str {
        "i'm running some exercise"
    }

    async fn run(self, o: OrchestratorReference<S>, should_stop: std::sync::Arc<Notify>) {
        let _id = o.memory().register("ciao", "mondo").await.unwrap();
        let auth = o.memory().login("ciao", "mondo").await.unwrap();
        let source = tokio::fs::read("source.rs").await.unwrap();
        let source = String::from_utf8(source).unwrap();
        let mut set: JoinSet<ExerciseResult> = JoinSet::new();
        for _ in 0..300 {
            let o = o.clone();
            let s = source.clone();
            let auth = auth.clone();
            let _join = set.spawn(async move {
                o.process_exercise("template".to_string(), s, auth)
                    .await
                    .unwrap()
            });
        }
        while set.join_next().await.is_some() {}
        should_stop.notify_one();
    }
}
