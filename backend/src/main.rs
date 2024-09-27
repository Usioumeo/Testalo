use backend::WebServer;
use orchestrator::memory::StatelessMemory;
use orchestrator::orchestrator::Orchestrator;
use orchestrator::GenerateState;
use rocket::tokio;
use rust_default::*;
use sql_abstractor::Postgres;

GenerateState!(
    RustExercise,
    RustGeneratedFiles,
    RustCompiled,
    ExerciseResult
);

#[tokio::main]
/// function used to test if the component works as it should
async fn main() {
    let memory = Postgres::clean_init("postgresql://postgres:test@localhost:5432/thesis")
        .await
        .unwrap();
    memory.register("ciao", "mondo").await.unwrap();
    let mut o: Orchestrator<State> = Orchestrator::new(16, true, Box::new(memory));
    /*
    o.add_executor(RustGeneratedFiles::compile, None)
        .await
        .unwrap();
    o.add_executor(|x, _| RustCompiled::run(x), ())
        .await
        .unwrap();

    o.enable_executor::<RustGeneratedFiles, RustCompiled, _>(None::<Option<PathBuf>>)
        .await
        .unwrap();
    o.enable_executor::<RustCompiled, ExerciseResult, _>(())
        .await
        .unwrap();

    let f1 = |c: String| async move { Ok(RustExercise::parse(&c)?) };
    let f2 =
        |def: RustExercise, source: String| async move { Ok(def.generate_files(source).await?) };
    o.add_exercise_generators(f1, f2).await;*/

    o.add_plugin(WebServer).await.unwrap();
    o.add_plugin(RustDefaultPlugin::default().set_activate_default())
        .await
        .unwrap();
    o.add_exercise::<RustExercise>(
        "es1",
        include_str!("../../student_delivery/src/exercise/es1.rs"),
    )
    .await
    .unwrap();
    o.add_exercise::<RustExercise>(
        "es2",
        include_str!("../../student_delivery/src/exercise/es2.rs"),
    )
    .await
    .unwrap();

    let _ = o.run().await;
}
