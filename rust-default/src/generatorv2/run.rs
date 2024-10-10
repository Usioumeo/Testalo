use std::collections::HashMap;

use orchestrator::prelude::{CompilationResult, ExerciseResult, RunResult, TestResult};
use tokio::{
    process::Command,
    task::{JoinError, JoinSet},
};

use super::compiled::RustCompiled;
#[derive(thiserror::Error, Debug)]
/// all the errors that could be generated by execution
pub enum RunError {
    #[error("JoinError: {0}")]
    JoinError(#[from] JoinError),
}
impl RustCompiled {
    /// execute and collect results
    pub async fn run(self) -> Result<ExerciseResult, RunError> {
        let mut set: JoinSet<(String, TestResult)> = JoinSet::new();

        //let's start executing all test in parallel
        for (name, mut test_result) in self.results {
            let exec = self.path.join("target").join("debug").join(&name);
            set.spawn(async move {
                if let CompilationResult::Built = test_result.compiled {
                    //let t = Command::new(exec).output().await?;
                    match Command::new(&exec).output().await {
                        Ok(output) if output.status.success() => {
                            //test_result.points_given = test_result.points;
                            test_result.runned = RunResult::Ok;
                        }
                        Ok(output) => {
                            test_result.points_given = 0.0;
                            test_result.runned = RunResult::Error(format!(
                                "Got this error while executing {} {}",
                                exec.to_str().unwrap(),
                                String::from_utf8(output.stderr).unwrap()
                            ));
                        }
                        Err(err) => {
                            test_result.points_given = 0.0;
                            test_result.runned = RunResult::Error(err.to_string())
                        }
                    }
                }
                (name, test_result)
            });
        }
        let mut tests = HashMap::new();
        while let Some(x) = set.join_next().await {
            let (name, result) = x?;
            tests.insert(name, result);
        }
        Ok(ExerciseResult { tests })
    }
}
