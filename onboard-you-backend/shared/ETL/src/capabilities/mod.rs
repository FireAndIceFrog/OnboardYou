//! Capabilities: Functional logic steps (The Actions/Filters)
//!
//! This module contains all the specific work implementations:
//! - ingestion: Data acquisition from external sources (traits + engine)
//! - logic: Domain-specific data transformations (traits + engine)
//! - egress: Data delivery to destinations

pub mod egress;
pub mod ingestion;
pub mod logic;
pub mod conversion;

// Re-export concrete engine types for ergonomic access
pub use egress::*;
pub use ingestion::engine::*;
pub use ingestion::traits::*;
pub use logic::engine::*;
