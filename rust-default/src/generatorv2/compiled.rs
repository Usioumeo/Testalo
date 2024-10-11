use std::{collections::HashMap, path::{Path, PathBuf}, string::FromUtf8Error};

use dircpy::copy_dir;
use orchestrator::prelude::{AsyncDefault, CompilationResult, RunResult, TestResult};
use serde_json::Value;
use tempdir::TempDir;
use tokio::{fs, process::Command};

use super::file_generator::GeneratedFiles;

/// Error that can get generated in a compilation with cargo
#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    /// From UTF8 Error
    #[error("It's not a valid utf-8 file. Got error: {0}")]
    FromUTF8Error(#[from] FromUtf8Error),
    /// Input Output Error
    #[error("IoError {0}")]
    IoError(#[from] std::io::Error),
}


/// Result of a rust compilation, it contains the path to be used.
#[derive(Debug)]
pub struct RustCompiled {
    /// Temporary directory
    _tmpdir: Option<TempDir>,
    /// path where the project is stored
    pub path: PathBuf,
    /// results of the compilation
    pub results: HashMap<String, TestResult>,
}
impl AsyncDefault for RustCompiled{
    async fn async_default() -> Self {
        // TODO not a real implementation
        RustCompiled{
            _tmpdir: None,
            path: PathBuf::new(),
            results: HashMap::new(),
        }
    }
}

impl Clone for RustCompiled {
    fn clone(&self) -> Self {
        let _tmpdir = match &self._tmpdir {
            Some(dir) => {
                let new_dir = TempDir::new("tmp_compile").unwrap();

                copy_dir(dir, &new_dir).unwrap();
                Some(new_dir)
            }
            None => None,
        };
        Self {
            _tmpdir,
            path: self.path.clone(),
            results: self.results.clone(),
        }
    }
}

/**
    it extracts the various error from the output
*/
fn parse_errors(inp: &str, tests: &mut HashMap<String, TestResult>) {
    let _: Vec<Option<()>> = inp
        .lines()
        .map(|t| -> Option<()> {
            let error = serde_json::from_str::<Value>(t).ok()?;
            let error = error.as_object()?;
            let message = error.get("message")?.as_object()?;
            let level = message.get("level")?;
            let rendered = message.get("rendered")?.as_str()?.to_string();
            let rendered = rendered.replace("\\n", std::str::from_utf8(b"\n").unwrap());

            let target = error.get("target")?.as_object()?;
            let name = target.get("name")?.as_str()?;
            if level != "error" {
                return None;
            }
            if let Some(test_result) = tests.get_mut(name) {
                if let CompilationResult::Error(msg) = &mut test_result.compiled {
                    msg.push('\n');
                    *msg += &rendered;
                } else {
                    test_result.points_given = 0.0;
                    test_result.compiled = CompilationResult::Error(rendered)
                }
            }
            Some(())
        })
        .collect();
}

/// function used to create a valid Cargo Project
async fn create_cargo_project(path: &Path, dependencies: &[String]) -> Result<(), CompileError> {
    //TODO clean well (delete target, overwrite other files)
    if path.exists() {
        fs::remove_dir_all(path).await?;
    }

    fs::create_dir(path).await?;
    let dep = dependencies.iter().fold(String::new(), |a, b| a+b);
    let toml = include_str!("./default_cargo.toml").to_string()+&dep;
    fs::write(path.join("Cargo.toml"), toml).await?;
    fs::create_dir(path.join("src")).await?;
    fs::create_dir(path.join("src/bin")).await?;
    Ok(())
}

impl RustCompiled {
    /// Compiles each files creating a project in a temporary directory, or in path if specified
    pub async fn compile(generated: GeneratedFiles, path: Option<PathBuf>) -> Result<Self, CompileError> {
        let (tmpdir, path) = if let Some(path) = path {
            (None, path)
        } else {
            let tmp_dir = TempDir::new("tmp_compile")?;
            let path = tmp_dir.path().to_owned();
            (Some(tmp_dir), path)
        };

        //generate crate
        create_cargo_project(&path, generated.dependencies.as_slice()).await?;

        for (name, (content, _)) in &generated.files {
            
            fs::write(
                path.join("src").join("bin").join(name.clone() + ".rs"),
                content,
            )
            .await?;
        }
        let compilation_output = Command::new("cargo")
            .arg("+nightly")
            .arg("build")
            .arg("--bins")
            .arg("--manifest-path")
            .arg(path.join("Cargo.toml"))
            .arg("--keep-going")
            .arg("--message-format=json")
            .output()
            .await?;
        let message = String::from_utf8(compilation_output.stdout)?;
        let mut results: HashMap<String, TestResult> = generated
            .files
            .into_iter()
            .map(|(name, (_, points))| {
                let test_result = TestResult {
                    compiled: CompilationResult::Built,
                    runned: RunResult::NotRun,
                    points_given: points as f64,
                };
                (name, test_result)
            })
            .collect();

        //println!("{} {}", message, String::from_utf8(compilation_output.stderr)?);
        parse_errors(&message, &mut results);
        Ok(RustCompiled {
            _tmpdir: tmpdir,
            path,
            results,
        })
    }
}