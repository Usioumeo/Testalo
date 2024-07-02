//! # The orchestrator crate
//! this is the base crate, and implements lot of trates for other implementation
//! 
mod executor_trait;

/// Include this module to have all the important features of this crate
pub mod prelude;
//pub mod rust_executor;
pub mod orchestrator;



pub mod default_memory;
pub mod executor;
pub mod memory;

pub mod plugin;
mod test;
