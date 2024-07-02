use std::path::PathBuf;

use orchestrator::default_memory::DefaultMemory;
use orchestrator::executor::ExecutorGlobalState;
use orchestrator::orchestrator::Orchestrator;
use orchestrator::GenerateState;
use rust_default::generator::{RustCompiled, RustExercise, RustGeneratedFiles};
use rust_default::plugins::cli_v2::CLIPlugin;
use rust_default::plugins::RustDefaultPlugin;
// stato->stato
// esercizio->.....->soluzione


GenerateState!(RustExercise, RustGeneratedFiles, RustCompiled, ExerciseResult);

//pub async fn generate_files(self, solution: String) -> Result<RustGeneratedFiles, RustError>;

#[tokio::main]
async fn main() {
    // init memory and orchestrator
    let mut o: Orchestrator<State> = Orchestrator::new(10, DefaultMemory::init());
    
    o.add_plugin(RustDefaultPlugin::default()).await.unwrap();
    // add plugins
    o.add_plugin(CLIPlugin {}).await.unwrap();

    // ADDING DATA

    // enable executors
    o.enable_executor::<RustGeneratedFiles, RustCompiled, _>(None::<PathBuf>).await.unwrap();
    o.enable_executor::<RustCompiled, ExerciseResult, _>(()).await.unwrap();

    // finaly, add exercises
    o.add_exercise::<RustExercise>("es1", include_str!("./exercise/es1.rs")).await.unwrap();
    o.add_exercise::<RustExercise>("es2", include_str!("./exercise/es2.rs")).await.unwrap();

    //run
    println!("all loaded, ready to run");
    o.run().await;
}
