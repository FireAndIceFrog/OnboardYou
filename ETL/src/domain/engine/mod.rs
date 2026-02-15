//! Domain engine: Concrete implementations of core data structures
//!
//! - errors: Unified error handling
//! - manifest: Versioned, declarative pipeline configuration
//! - roster: The RosterContext wrapping Polars LazyFrame + metadata

pub mod errors;
pub mod manifest;
pub mod roster;

pub use errors::{Error, Result};
pub use manifest::{ActionConfig, ActionType, Manifest};
pub use roster::{FieldMetadata, RosterContext};
