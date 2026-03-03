//! Domain engine: Concrete implementations of core data structures
//!
//! - errors: Unified error handling
//! - manifest: Versioned, declarative pipeline configuration
//! - roster: The RosterContext wrapping Polars LazyFrame + metadata

pub mod errors;
pub mod manifest;
pub mod org_settings;
pub mod pipeline_config;
pub mod plan_prompt;
pub mod plan_summary;
pub mod roster;
pub mod scheduled_event;
pub mod pipeline_models;
pub mod schema_diff;

pub use pipeline_models::*;
pub use scheduled_event::*;
pub use errors::*;
pub use manifest::*;
pub use org_settings::*;
pub use pipeline_config::*;
pub use plan_prompt::*;
pub use plan_summary::*;
pub use roster::*;
pub use schema_diff::*;
