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

pub use scheduled_event::{ScheduledEtlEvent, ScheduledEvent, ScheduledDynamicApiEvent};
pub use errors::{Error, Result};
pub use manifest::{ActionConfig, ActionType, Manifest};
pub use org_settings::OrgSettings;
pub use pipeline_config::PipelineConfig;
pub use roster::{FieldMetadata, RosterContext};
