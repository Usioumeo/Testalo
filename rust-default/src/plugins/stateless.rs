use clap::{Parser, Subcommand};
use orchestrator::prelude::*;
use tokio::fs;
use std::{error::Error, path::PathBuf, sync::Arc};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// does testing things
    Submit {
        /// lists test values
        #[arg(short, long)]
        exercise_name: String,

        /// lists test values
        #[arg(short, long)]
        file_path: PathBuf,
    },
}

pub struct StatelessCLIPlugin;
impl<S: ExecutorGlobalState> Plugin<S> for StatelessCLIPlugin {
    fn name(&self) -> &str {
        "Stateless Cli"
    }

    fn desctiption(&self) -> &str {
        ""
    }

    async fn run(self, o: OrchestratorReference<S>, should_stop: Arc<Notify>) {
        let _ = o.memory().register("cli_plugin", "cli_plugin").await;
        let login = o.memory().login("cli_plugin", "cli_plugin").await.unwrap();
        let a = Args::parse();
        let names = o.memory().list_exercise_names().await.unwrap();
        match  a.command {
            Commands::Submit { exercise_name, file_path } => {
                if !names.contains(&exercise_name){
                    println!("Exercise not found, choose from the following:");
                    for x in names{
                        println!("\t {x}");
                    }
                }
                let test = async {
                    let file = fs::read(file_path).await?;
                    let file = String::from_utf8(file)?;
                    let result = o.process_exercise(exercise_name, file, login).await?;
                    Ok::<ExerciseResult, Box<dyn Error + Send + Sync>>(result)
                };
                match test.await {
                    Ok(v) => {
                        println!("Ok, got: {}", v);
                    }
                    Err(x) => {
                        println!("got error: {x}")
                    }
                }
            },
           // _ => {}
        }
        should_stop.notify_one();
        
    }
    async fn on_add<'a>(
        &'a mut self,
        _o: &'a mut Orchestrator<S>,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        
        Ok(())
    }

}
