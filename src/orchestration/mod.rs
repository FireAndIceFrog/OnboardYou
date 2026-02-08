//! Orchestration: Pipeline assembly and execution (The Mediator)
//!
//! - Factory: Maps manifest string IDs to Capability instances
//! - Pipeline Runner: Sequentially executes Actions on the RosterContext

pub mod factory;
pub mod pipeline_runner;

pub use factory::*;
pub use pipeline_runner::*;
