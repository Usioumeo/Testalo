/*! Contains all executor-related objects.

*/

use crate::prelude::*;
use colored::Colorize;
use std::any::Any;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Clone, Debug)]
/// test definition:
pub struct TestDefinition {
    /// name of the current test
    pub name: String,
    /// description of the current test
    pub description: String,
    /// how many points is it worth?
    pub points: f64,
    /// is it visible?
    pub is_visible: bool,
}

/// Every exercise must implement this interface.
/// In particular it must implement all the methods and Clone.
///
/// In addition to that it must be Send and Sync.
pub trait ExerciseDef: Any + Send + Sync {
    /// return a description
    fn description(&self) -> &str;
    /// from which source code was it generate?
    fn get_generator_src(&self) -> &str;
    /// which test must it generate?
    fn list(&self) -> Vec<TestDefinition>;
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
/// Results of a common Exercise. It contains a list of the results of each test.
pub struct ExerciseResult {
    /// list of the result of each tests:
    /// The key is the name, and TestResult contains all other informations
    pub tests: HashMap<String, TestResult>,
}
impl Display for ExerciseResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = writeln!(f, "ExerciseResult:");
        let mut v: Vec<(String, TestResult)> = self
            .tests
            .iter()
            .map(|(x, y)| (x.clone(), y.clone()))
            .collect();
        v.sort_by(|x, y| {
            let cmp = x.1.cmp(&y.1);
            if cmp.is_ne() {
                return cmp;
            }
            x.0.cmp(&x.0)
        });
        for (name, result) in v {
            if let CompilationResult::Error(x) = result.compiled {
                let _ = write!(
                    f,
                    "   {}: {} {}",
                    name.green(),
                    "Compilation Error:".red(),
                    x
                );
            } else if let RunResult::Error(x) = result.runned {
                let _ = write!(f, "   {}: {} {}", name.green(), "Run Error:".red(), x);
            } else {
                let _ = write!(f, "   {}: {}", name, result);
            }
        }
        write!(f, "")
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
/// Contains the results of each tests
pub struct TestResult {
    /// Compilation status
    pub compiled: CompilationResult,
    /// Execution status
    pub runned: RunResult,
    /// Points awarded
    pub points_given: f64,
}
impl Eq for TestResult {}
impl Ord for TestResult {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp = self.compiled.cmp(&other.compiled);
        if cmp.is_ne() {
            return cmp;
        }
        let cmp = self.runned.cmp(&other.runned);
        if cmp.is_ne() {
            return cmp;
        }
        self.points_given.total_cmp(&other.points_given)
    }
}
impl PartialOrd for TestResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "TestResult({}, {:<16}, points: {:.2})",
            self.compiled,
            self.runned.to_string(),
            self.points_given
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Status of a compilation
pub enum CompilationResult {
    /// If the compilation succeded
    Built,
    /// if we get an error, the string of the error
    Error(String),
    /// not yet built
    #[default]
    NotBuilt,
}

impl Ord for CompilationResult {
    /// first built, then error, then not_built
    fn cmp(&self, other: &Self) -> Ordering {
        use CompilationResult::*;
        use Ordering::*;
        match (self, other) {
            (Built, Built) => Equal,
            (Built, _) => Less,
            (Error(_), Built) => Greater,
            (Error(x), Error(y)) => x.cmp(y),
            (Error(_), NotBuilt) => Less,
            (NotBuilt, NotBuilt) => Equal,
            (NotBuilt, _) => Greater,
        }
    }
}
impl PartialOrd for CompilationResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for CompilationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationResult::Built => write!(f, "{}", "Built".green()),
            CompilationResult::Error(_) => write!(f, "{}", "Error".red()),
            CompilationResult::NotBuilt => write!(f, "{}", "Not built".yellow()),
        }
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Status of an execution
pub enum RunResult {
    /// Execution completed succesfuly
    Ok,
    /// it did not execute correctly, the returned error is:
    Error(String),
    /// not yet run
    #[default]
    NotRun,
}

impl Ord for RunResult {
    /// first built, then error, then not_built
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;
        use RunResult::*;
        match (self, other) {
            (Ok, Ok) => Equal,
            (Ok, _) => Less,
            (Error(_), Ok) => Greater,
            (Error(x), Error(y)) => x.cmp(y),
            (Error(_), NotRun) => Less,
            (NotRun, NotRun) => Equal,
            (NotRun, _) => Greater,
        }
    }
}
impl PartialOrd for RunResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for RunResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunResult::Ok => write!(f, "{}", "Ok".green()),
            RunResult::Error(_) => write!(f, "{}", "Error".red()),
            RunResult::NotRun => write!(f, "{}", "Not run".yellow()),
        }
    }
}
