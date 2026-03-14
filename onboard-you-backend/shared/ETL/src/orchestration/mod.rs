//! Orchestration: Pipeline assembly and execution (The Mediator)
//!
//! - Factory: Maps manifest string IDs to Capability instances
//! - Pipeline Runner: Sequentially executes Actions on the RosterContext
//! - Clients: Shared HTTP / SOAP client abstractions

pub mod clients;
pub mod factory;

pub use clients::*;
pub use factory::{ActionFactory, ActionFactoryTrait, StepError};
