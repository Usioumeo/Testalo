use std::collections::HashMap;

use orchestrator::{default_memory::DefaultMemory, prelude::*, GenerateState};
use quote::quote;

use crate::*;

GenerateState!(
    RustExercise,
    RustGeneratedFiles,
    ExerciseResult,
    RustCompiled,
    DummyExercise
);

#[tokio::test]
async fn test_orchestrator() {
    let m = DefaultMemory::init();
    let mut o: Orchestrator<State> = Orchestrator::new(16, true, m);
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
//struct S ; impl S { fn new () -> S { S } fn print (& self) -> String { format ! ("Struct S") } } # [runtest (2)] # [doc = r" test if new exists, and works"] fn test_new () { let s : S = S :: new () ; } # [runtest] # [refers_to (S :: print)] fn test_display () { let s = S :: new () ; assert_eq ! (s . print () , format ! ("Struct S")) ; }
#[tokio::test]
async fn test_substitution() {
    let m = DefaultMemory::init();
    let mut o: Orchestrator<State> = Orchestrator::new(16, true,  m);
    let example = quote! {
        struct S;
        impl S{
            fn new()->S{
                S
            }
            fn print(&self)->String{
                format!("Struct S")
            }
        }
        #[runtest(2)]
        /// test if new exists, and works
        fn test_new(){
            let s: S = S::new();
        }

        #[runtest(1)]
        #[refers_to(S::new)]
        fn test_display(){
            let s = S::new();
            assert_eq!(s.print(), format!("Struct S"));
        }
    }
    .to_string();
    let example = syn::parse_file(&example).unwrap();
    let example = prettyplease::unparse(&example);
    println!("{}", example);
    let user_code = quote! {
        struct S;
        impl S{
            fn print(&self)->String{
                format!("Struct S")
            }
        }
    }
    .to_string();
    o.add_plugin(RustDefaultPlugin::default().set_activate_default())
        .await
        .unwrap();

    let t = o.add_exercise::<RustExercise>("es1", &example).await;
    if let Err(t) = t {
        panic!("{}", t);
    }
    let mut def = DefaultTest::new_default();
    /*def.set_exercise(
        "es1".to_string(),
        user_code
    );*/

    def.set_additional_function(move |mut int: Box<dyn TestInterface>| {
            let user_code = user_code.clone();
            async move {
                let t = int.submit("es1".to_string(), user_code).await?;
                let mut e = HashMap::new();
                use orchestrator::prelude::CompilationResult::*;
                use orchestrator::prelude::RunResult;
                e.insert("test_display".to_string(), TestResult { compiled: Built, runned: RunResult::Ok, points_given: 1.0 });
                e.insert("test_new".to_string(), TestResult { compiled: CompilationResult::Error("error[E0599]: no function or associated item named `new` found for struct `S` in the current scope\n  --> src/bin/test_new.rs:10:19\n   |\n2  | struct S;\n   | -------- function or associated item `new` not found for this struct\n...\n10 |     let s: S = S::new();\n   |                   ^^^ function or associated item not found in `S`\n\n\nerror: aborting due to 1 previous error\n\n".to_string()), runned:RunResult::NotRun, points_given: 0.0 });

                assert_eq!(t.tests, e);
                //println!("{t}");
                //todo!()
                Ok(())
            }
    });
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
