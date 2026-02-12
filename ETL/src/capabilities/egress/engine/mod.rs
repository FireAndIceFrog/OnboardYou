//! Egress engine: Orchestration layer for authentication and dispatch
//!
//! - **ApiEngine**: Orchestrator — selects the right repository, applies retry policy

pub mod api_engine;

pub use api_engine::*;
