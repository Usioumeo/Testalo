use orchestrator::{
    default_memory::DefaultMemory,
    prelude::*,
    GenerateState,
};

use crate::*;

#[tokio::test]
async fn test_orchestrator() {
    GenerateState!(
        RustExercise,
        RustGeneratedFiles,
        ExerciseResult,
        RustCompiled,
        DummyExercise
    );
    let m = DefaultMemory::init();
    let mut o: Orchestrator<State> = Orchestrator::new(16, m);
    let example = r#"
        fn string_reverse(inp: &str)->String{
            inp.chars().rev().collect()
        }

        #[runtest]
        fn test_inverse(){
            assert_eq!(string_reverse("ciao"), "oaic".to_string());
        }
        "#;
    o.add_plugin(RustDefaultPlugin::default().set_activate_default())
        .await
        .unwrap();

    o.add_exercise::<RustExercise>("es1", example)
        .await
        .unwrap();
    let mut def = DefaultTest::new_default();
    def.set_exercise(
        "es1".to_string(),
        "
        fn string_reverse(inp: &str)->String{
            inp.chars().rev().collect()
        }"
        .to_string(),
    );
    o.add_plugin(def).await.unwrap();
    let _ = o.run().await;
}
#[cfg(feature = "docker")]
#[tokio::test]
async fn test_orchestrator_docker() {
    use std::sync::Arc;

    use crate::docker::{DockerCompile, DockerRun};

    let mut orchestrator = Orchestrator::new(12);

    let t = RustExercise::load("../template.rs").await.unwrap();

    let f = move |e: RustExercise, s| async {
        let file = e.generate_files(s).await?;
        let compiled = DockerCompile::compile(file).await?;
        let runned = DockerRun::run(compiled).await?; //DockerRun::run(compiled).await?;
        Ok(runned)
    };
    orchestrator.add_exercise(t, f).await.unwrap();
    let orchestrator = Arc::new(orchestrator);
    let source = tokio::fs::read("../source.rs").await.unwrap();
    let source = String::from_utf8(source).unwrap();
    let s = source.clone();
    orchestrator.process("es1", s).await.unwrap();
}
