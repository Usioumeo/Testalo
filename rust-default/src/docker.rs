use std::collections::HashMap;

use bollard::{
    container::{Config, LogOutput, RemoveContainerOptions},
    exec::{CreateExecOptions, StartExecResults},
    secret::HostConfig,
    Docker,
};
use futures_util::StreamExt;

use orchestrator::prelude::{CompilationResult, ExerciseResult, RunResult, TestResult};
use tempdir::TempDir;
use tokio::{
    fs,
    task::{JoinError, JoinSet},
};

use crate::generator::{
    compile::{create_cargo_project, parse_errors, RustCompiled},
    RustGeneratedFiles,
};

pub trait DockerCompile
where
    Self: Sized,
{
    type Output: Sized;
    fn compile(
        self,
    ) -> impl std::future::Future<Output = Result<Self::Output, Error>> + Send + Sync;
}
#[derive(thiserror::Error, Debug)]
pub enum Error {
    //#[error("io error")]
    //IoError(#[from] tokio::io::Error),
    #[error("io error")]
    StdIoError(#[from] std::io::Error),

    #[error("bollard/Docker error")]
    Bollard(String),
    #[error("join error")]
    Join(#[from] JoinError),
}

impl From<bollard::errors::Error> for Error {
    fn from(_: bollard::errors::Error) -> Self {
        todo!()
    }
}

impl DockerCompile for RustGeneratedFiles {
    async fn compile(self) -> Result<RustCompiled, Error> {
        let docker = Docker::connect_with_socket_defaults().unwrap();

        //create tmp dir
        let _tmpdir = TempDir::new("tmp_compile")?;
        let path = _tmpdir.path().to_owned();

        //create cargo project in tmp-dir
        create_cargo_project(&path).await.unwrap();

        //create source for each exercise:
        for (name, (content, _)) in &self.files {
            fs::write(
                path.join("src").join("bin").join(name.clone() + ".rs"),
                content,
            )
            .await?;
        }

        //generate config
        let config = Config {
            image: Some("rust:latest"),
            tty: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            host_config: Some(HostConfig {
                binds: Some(vec![format!("{}:/home/project/", path.to_str().unwrap())]),
                ..Default::default()
            }),
            //cmd: Some(vec!["tail -f /dev/null"]),
            //cmd: Some(vec!["cargo build --bins --mainfest-path /home/project/Cargo.toml --keep-going --message-format=json"]),
            ..Default::default()
        };

        //launch docker
        let id = docker
            .create_container::<&str, &str>(None, config)
            .await?
            .id;

        docker.start_container::<String>(&id, None).await?;

        let exec = docker
            .create_exec(
                &id,
                CreateExecOptions {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    cmd: Some(vec![
                        "cargo",
                        "build",
                        "--bins",
                        "--manifest-path",
                        "/home/project/Cargo.toml",
                        "--keep-going",
                        "--message-format=json",
                    ]),
                    ..Default::default()
                },
            )
            .await?
            .id;

        let docker2 = docker.clone();
        let s = tokio::spawn(async move {
            let mut compilation_output = String::new();
            let output = docker2.start_exec(&exec, None).await.unwrap();
            let StartExecResults::Attached { mut output, .. } = output else {
                unreachable!()
            };
            while let Some(Ok(msg)) = output.next().await {
                compilation_output += &msg.to_string();
            }
            compilation_output
        });
        let compilation_output = s.await?;
        /*if let StartExecResults::Attached { mut output, input } = output{
            let t = output.try_next().await;
            //let t: Vec<Result<LogOutput, _>> = output.map(|x| x.unwrap()).collect();
            while let Some(x) = output.next().await{

            }
        }*/
        //let output = todo!();
        //let StartExecResults::Attached { mut output, .. } = docker.start_exec(&exec, None).await.unwrap() else {unreachable!()};
        /*let mut output = Box::pin(output);
        while let Some(Ok(msg)) = output.next().await {
            compilation_output += &msg.to_string();
        }**/
        let mut exercises: HashMap<String, TestResult> = self
            .files
            .into_iter()
            .map(|(name, (_, points))| {
                let test_result = TestResult {
                    compiled: CompilationResult::Built,
                    runned: RunResult::NotRun,
                    points_given: points,
                };
                (name, test_result)
            })
            .collect();

        parse_errors(&compilation_output, &mut exercises);
        let _ = docker
            .remove_container(
                &id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await;
        Ok(RustCompiled {
            _tmpdir: Some(_tmpdir),
            path,
            results: exercises,
        })
    }

    type Output = RustCompiled;
}

pub trait DockerRun
where
    Self: Sized,
{
    fn run(self) -> impl std::future::Future<Output = Result<ExerciseResult, Error>> + Send + Sync;
}

impl DockerRun for RustCompiled {
    async fn run(self) -> Result<ExerciseResult, Error> {
        let docker = Docker::connect_with_socket_defaults().unwrap();
        //let mut set: JoinSet<(String, TestResult)> = JoinSet::new();
        let bin_path = self.path.join("target").join("debug");
        let time = tokio::time::Instant::now();
        let config = Config {
            image: Some("alpine:latest"),
            tty: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            host_config: Some(HostConfig {
                binds: Some(vec![format!(
                    "{}:/home/project/",
                    bin_path.to_str().unwrap()
                )]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let id = docker
            .create_container::<&str, &str>(None, config)
            .await?
            .id;
        println!("Creation time= {}", time.elapsed().as_secs_f32());
        docker.start_container::<String>(&id, None).await?;
        let mut join = JoinSet::new();

        for (name, mut test_result) in self.results {
            //let exec = self.path.join("target").join("debug").join(&name);
            let command = format!("{name}  && echo \"pass\" || echo \"fail\"");
            let exec = docker
                .create_exec(
                    &id,
                    CreateExecOptions {
                        attach_stdout: Some(true),
                        attach_stderr: Some(true),
                        cmd: Some(vec!["sh", "-c", &command]),
                        ..Default::default()
                    },
                )
                .await?
                .id;

            let docker2 = docker.clone();

            /*set.spawn(async move {
                    //generate config

                    let s = tokio::spawn(async move {
                let mut compilation_output = String::new();

                let StartExecResults::Attached { mut output, .. } = output else {unreachable!()};
                while let Some(Ok(msg)) = output.next().await {
                    compilation_output += &msg.to_string();
                }
                compilation_output
            }); */
            join.spawn(async move {
                if let CompilationResult::Built = test_result.compiled {
                    if let StartExecResults::Attached { mut output, .. } =
                        docker2.start_exec(&exec, None).await.unwrap()
                    {
                        let mut output_string = String::new();
                        //let t = output.next().await;
                        while let Some(Ok(msg)) = output.next().await {
                            match msg {
                                LogOutput::StdErr { message } => {
                                    output_string += &String::from_utf8(message.to_vec()).unwrap()
                                }
                                LogOutput::StdOut { message } => {
                                    output_string += &String::from_utf8(message.to_vec()).unwrap()
                                }
                                _ => {}
                            }
                        }

                        if let Some(m) = output_string.lines().last() {
                            if m == "pass" {
                                //test_result.points_given += run_test.points;
                                test_result.runned = RunResult::Ok;
                            } else if m == "fail" {
                                let mut v: Vec<&str> = output_string.lines().collect();
                                v.pop();
                                let output_string: String =
                                    v.iter().flat_map(|x| x.chars()).collect();
                                test_result.runned = RunResult::Error(output_string);
                                test_result.points_given = 0.0;
                            }
                        }
                    } else {
                        unreachable!();
                    }
                    //todo!()
                    (name, test_result)
                } else {
                    (name, test_result)
                }
            });

            //(name, test_result)
        }
        let mut tests = HashMap::new();
        while let Some(x) = join.join_next().await {
            if let Ok((name, value)) = x {
                tests.insert(name, value);
            }
        }
        let t = tokio::time::Instant::now();
        /*let _ = docker.remove_container(
        &id,
        Some(RemoveContainerOptions {
            force: true,
            ..Default::default()
        }),).await;*/
        println!("remove time {}", t.elapsed().as_secs_f32());
        Ok(ExerciseResult { tests })
        //todo!()
    }
}
