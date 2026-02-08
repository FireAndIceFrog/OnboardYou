//! Capabilities: Functional logic steps (The Actions/Filters)
//!
//! This module contains all the specific work implementations:
//! - ingestion: Data acquisition from external sources
//! - logic: Domain-specific data transformations
//! - egress: Data delivery to destinations

pub mod egress;
pub mod ingestion;
pub mod logic;

pub use egress::*;
pub use ingestion::*;
pub use logic::*;
