//! Generic trait for external HRIS systems (Workday, BambooHR, etc)

use crate::domain::{Result};
use polars::prelude::*;

/// Generic trait for HRIS system connectors
///
/// Implementors fetch data from a specific HRIS system and return it as a
/// Polars `LazyFrame` ready for pipeline processing.
pub trait HrisConnector: Send + Sync {
    /// Fetch data from the HRIS system and return a LazyFrame
    fn fetch_data(&self) -> Result<LazyFrame>;
}
