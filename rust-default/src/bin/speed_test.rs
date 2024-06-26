use cap::Cap;
use tikv_jemallocator::Jemalloc;
#[global_allocator]
static ALLOCATOR: Cap<Jemalloc> = Cap::new(Jemalloc, usize::MAX);

use std::{mem, time::Duration};

use indicatif::ProgressBar;
use orchestrator::{default_memory::DefaultMemory, prelude::*, GenerateState};
//use rust_default::{docker::DockerRun, plugins::cli::CLIPlugin, rust_parser::RustExercise};
use std::error::Error;

use tokio::task::JoinSet;
#[derive(Clone, Default)]
struct ParallelTestInput;
impl ExerciseDef for ParallelTestInput {
    fn description(&self) -> &str {
        ""
    }

    fn get_generator_src(&self) -> &str {
        ""
    }

    fn list(&self) -> Vec<TestDefinition> {
        Vec::new()
    }
}
#[derive(Clone, Default)]
struct SerialTestInput;
impl ExerciseDef for SerialTestInput {
    fn description(&self) -> &str {
        ""
    }

    fn get_generator_src(&self) -> &str {
        ""
    }

    fn list(&self) -> Vec<TestDefinition> {
        Vec::new()
    }
}

GenerateState!(ParallelTestInput, SerialTestInput, ExerciseResult);
async fn run_parallel(
    _x: ParallelTestInput,
    _y: (),
) -> Result<ExerciseResult, Box<dyn Error + Send + Sync + 'static>> {
    tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
    Ok(ExerciseResult::default())
}
async fn run_serial(
    _x: SerialTestInput,
    _y: (),
) -> Result<ExerciseResult, Box<dyn Error + Send + Sync + 'static>> {
    // tokio::time::sleep(Duration::from_secs_f32(0.0)).await;
    Ok(ExerciseResult::default())
}
#[tokio::main]
async fn main() {
    let mut o: Orchestrator<State> = Orchestrator::new(100000000, DefaultMemory::init());
    o.add_executor(run_parallel, ()).await.unwrap();
    o.add_executor(run_serial, ()).await.unwrap();
    o.enable_executor::<ParallelTestInput, ExerciseResult, ()>(())
        .await
        .unwrap();
    o.enable_executor::<SerialTestInput, ExerciseResult, ()>(())
        .await
        .unwrap();
    o.add_exercise_generators(
        |_x| async { Ok(ParallelTestInput) },
        |_, _| async { Ok(ParallelTestInput) },
    )
    .await;
    o.add_exercise_generators(
        |_x| async { Ok(SerialTestInput) },
        |_, _| async { Ok(SerialTestInput) },
    )
    .await;
    // ADDING FUNCTIONALITY
    //o.add_plugin(LogMemory) .await.unwrap();
    o.add_plugin(Run).await.unwrap();
    o.add_exercise::<ParallelTestInput>("template_parallel", "")
        .await
        .unwrap();
    o.add_exercise::<SerialTestInput>("template_serial", "")
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

    async fn run(self, o: OrchestratorReference<S>, _should_stop: std::sync::Arc<Notify>) {
        let _id = o.memory().register("ciao", "mondo").await.unwrap();
        let auth = o.memory().login("ciao", "mondo").await.unwrap();
        //let source = tokio::fs::read("source.rs").await.unwrap();
        tokio::time::sleep(Duration::from_secs_f32(5.0)).await;
        println!("Currently allocated: {}B", ALLOCATOR.allocated());
        let n: u64 = 3000000;
        for _ in 0..10 {
            parallel_test(n, o.clone(), auth.clone()).await;
            println!("Currently allocated: {}B", ALLOCATOR.allocated());
        }

        //should_stop.notify_one();
        /*{

        }*/
    }
}
async fn parallel_test<S: ExecutorGlobalState>(
    n: u64,
    o: OrchestratorReference<S>,
    auth: User<Authenticated>,
) {
    let mut set: JoinSet<ExerciseResult> = JoinSet::new();
    println!("spawning:");
    let bar = ProgressBar::new(n);
    let time = tokio::time::Instant::now();
    for _ in 0..n {
        let o = o.clone();
        let s = String::new();
        let auth = auth.clone();
        let _join = set.spawn(async move {
            /*tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
            ExerciseResult::default()*/
            o.process_exercise("template_parallel".to_string(), s, auth)
                .await
                .unwrap()
        });
        bar.inc(1);
    }
    mem::drop(bar);
    println!("Currently allocated: {}B", ALLOCATOR.allocated());
    println!("collecting:");
    let bar = ProgressBar::new(n);
    while let Some(x) = set.join_next().await {
        mem::drop(x);
        bar.inc(1);
    }
    mem::drop(bar);
    let elapsed = time.elapsed().as_secs_f32();
    println!("elapsed time {}", elapsed);
    println!("iter/sec {}", n as f32 / elapsed);
    mem::drop(set);
    //should_stop.notify_one();
}
