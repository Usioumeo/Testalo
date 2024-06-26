use orchestrator::prelude::*;
use std::{io::stdin, sync::Arc};
use tokio::{fs, sync::Notify};
//use tokio::fs;

pub struct CLIPlugin {}
impl<S: ExecutorGlobalState> Plugin<S> for CLIPlugin {
    fn name(&self) -> &str {
        "CLIPlugin"
    }

    fn desctiption(&self) -> &str {
        "takes commands from CLI"
    }

    async fn run(
        self,
        o: orchestrator::prelude::OrchestratorReference<S>,
        should_stop: Arc<Notify>,
    ) {
        let _ = o.memory().register("cli_plugin", "cli_plugin").await;
        let login = o.memory().login("cli_plugin", "cli_plugin").await.unwrap();
        println!("console started");
        let mut s = String::new();
        while stdin().read_line(&mut s).is_ok() {
            let v: Vec<&str> = s.split_ascii_whitespace().collect();
            if v.first() == Some(&"process") && v.len() == 3 {
                println!("match");
                //let Ok(index) = v[1].parse() else { continue };
                let index = v[1];
                let Some(file) = fs::read(v[2])
                    .await
                    .ok()
                    .and_then(|x| String::from_utf8(x).ok())
                //.and_then(|x| {println!("lol");String::from_utf8(x).ok()})
                else {
                    println!("something wrong");
                    continue;
                };
                println!("{}", file);
                //let file = "lol".to_string();
                //println!("loaded");
                match o
                    .process_exercise(index.to_string(), file, login.clone())
                    .await
                {
                    Ok(t) => {
                        println!("Ok: Result:");
                        for (name, result) in t.tests {
                            println!("{}{}", name, result);
                        }
                    }
                    Err(e) => println!("Got an err: {}", e),
                }
            } else if v.first() == Some(&"q") {
                break;
            }
            s = String::new();
        }
        should_stop.notify_one();
    }
}
