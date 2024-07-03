use std::{collections::HashSet, error::Error, fs, sync::Arc};

use inquire::{
    autocompletion::Replacement,
    validator::{StringValidator, Validation},
    Autocomplete, Text,
};
use orchestrator::prelude::*;
use tokio::sync::Notify;

#[derive(Clone)]
/// AutoComplete struct
struct AutoComplete {
    valid_exercises: HashSet<String>,
}

impl Autocomplete for AutoComplete {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        let v: Vec<&str> = input.split_ascii_whitespace().collect();

        let cmd_list = vec!["quit".to_string(), "process".to_string()];
        let Some(command) = v.first() else {
            return Ok(cmd_list);
        };
        //cmd_list.contains(&command.to_string());
        Ok(match command {
            //&"quit" => Some(vec!["quit".to_string()]),
            //&"process" if v.len()>2 => {vec![]},
            &"process" => {
                let s = v.get(1).unwrap_or(&"");
                let available_esercise: Vec<String> = self
                    .valid_exercises
                    .iter()
                    .filter(|x| x.starts_with(s))
                    .map(|x| format!("process {}", x))
                    .collect();
                if available_esercise.len() != 1 {
                    return Ok(available_esercise);
                }
                let Some(path) = v.get(2) else {
                    return Ok(available_esercise);
                };
                vec![format!("{} {}", available_esercise[0], path)]
            }
            s => cmd_list.into_iter().filter(|x| x.starts_with(s)).collect(),
        })
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, inquire::CustomUserError> {
        if let Some(suggestion) = highlighted_suggestion {
            return Ok(Some(suggestion));
        }
        let suggestion = self.get_suggestions(input)?;
        if suggestion.len() == 1 {
            Ok(Some(suggestion[0].to_string()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone)]
struct Validator {
    valid_exercises: HashSet<String>,
}

impl StringValidator for Validator {
    fn validate(&self, input: &str) -> Result<Validation, inquire::CustomUserError> {
        let v: Vec<&str> = input.split_ascii_whitespace().collect();
        Ok(match v.first() {
            Some(&"process") if v.len() == 3 => {
                if self.valid_exercises.contains(v[1]) {
                    Validation::Valid
                } else {
                    Validation::Invalid(format!("{} is not a valid exercise", v[1]).into())
                }
            }
            Some(&"process") => Validation::Invalid("invalid parameter count, expect 2".into()),
            Some(&"quit") if v.len() == 1 => Validation::Valid,
            Some(&"quit") => Validation::Invalid("quit doesn't take any parameters".into()),
            Some(_) => Validation::Invalid("not a known command".into()),
            None => Validation::Invalid("empty string".into()),
        })
    }
}
/// Plugin that implement a CLI for the orchestrator
pub struct CLIPlugin;
impl<S: ExecutorGlobalState> Plugin<S> for CLIPlugin {
    fn name(&self) -> &str {
        "Command Line Interface Plugin"
    }

    fn desctiption(&self) -> &str {
        "Interface Plugin. It Expose the inner function throught a CLI"
    }

    async fn run(self, o: OrchestratorReference<S>, should_stop: Arc<Notify>) {
        let _ = o.memory().register("cli_plugin", "cli_plugin").await;
        let login = o.memory().login("cli_plugin", "cli_plugin").await.unwrap();
        let available_names: HashSet<String> = o
            .memory()
            .list_exercise_names()
            .await
            .unwrap()
            .into_iter()
            .collect();
        let auto_complete = AutoComplete {
            valid_exercises: available_names.clone(),
        };
        let validator = Validator {
            valid_exercises: available_names,
        };
        loop {
            let Ok(cmd) = Text::new("")
                .with_validator(validator.clone())
                .with_autocomplete(auto_complete.clone())
                .prompt()
            else {
                println!("Something wrong appened while parsing your command");
                continue;
            };
            let v: Vec<&str> = cmd.split_ascii_whitespace().collect();
            match v[0] {
                "process" => {
                    let login = login.clone();
                    let test = async {
                        let file = fs::read(v[2])?;
                        let file = String::from_utf8(file)?;
                        let result = o.process_exercise(v[1].to_string(), file, login).await?;
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
                }
                "quit" => {
                    should_stop.notify_one();
                    break;
                }
                _ => unreachable!(),
            }
        }
    }
}
