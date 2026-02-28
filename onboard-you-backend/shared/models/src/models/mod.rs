//! Domain engine: Concrete implementations of core data structures
//!
//! - errors: Unified error handling
//! - manifest: Versioned, declarative pipeline configuration
//! - roster: The RosterContext wrapping Polars LazyFrame + metadata

pub mod errors;
pub mod manifest;
pub mod org_settings;
pub mod pipeline_config;
pub mod roster;
pub mod scheduled_event;
pub mod pipeline_models;

pub use pipeline_models::*;
pub use scheduled_event::*;
pub use errors::*;
pub use manifest::*;
pub use org_settings::*;
pub use pipeline_config::*;
pub use roster::*;
