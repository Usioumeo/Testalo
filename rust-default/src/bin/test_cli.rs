use std::fs;

use orchestrator::{default_memory::DefaultMemory, prelude::*, GenerateState};
use rust_default::*;


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
    o.add_plugin(StatelessCLIPlugin).await.unwrap();

    // add exercise
    let template = fs::read("template.rs").unwrap();
    let template = String::from_utf8(template).unwrap();
    o.add_exercise::<RustExercise>("template", &template)
        .await
        .unwrap();
    let _o = o.run().await;
}

/*fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello {}!", args.name);
    }
}*/