//! Main module of this crate. It contains all related modules and re-export the important functionalities

#[cfg(feature = "docker")]
/// Module that contains all docker-related executors
pub mod docker;
/// Module with a macro that permits to embed exerxises into the executable
pub mod embed;
/// Module where all the plugins gets defined
pub (crate) mod plugins;
/// How to generate a rust exercise? how to compile it? ALl of this is present inside this module
pub (crate) mod generator;
#[cfg(test)]
mod test;


pub use crate::plugins::{cli_v2::CLIPlugin, rust_default::RustDefaultPlugin};
pub use crate::generator::{RustExercise, RustGeneratedFiles, RustCompiled};