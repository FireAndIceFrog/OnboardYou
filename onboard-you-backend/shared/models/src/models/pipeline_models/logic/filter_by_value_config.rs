//! Configuration model for the filter-by-value engine.

use super::SafeRegex;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Configuration for the filter-by-value action.
///
/// | Field    | Type   | Description                                             |
/// |----------|--------|---------------------------------------------------------|
/// | `column` | string | Target column whose values are tested against the regex |
/// | `pattern`| string | Regex pattern (Rust `regex` syntax); rows that match    |
/// |          |        | are **kept**, non-matching rows are dropped              |
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FilterByValueConfig {
    /// Column to filter on.
    pub column: String,
    /// The raw regex pattern.
    pub pattern: String,
}

impl FilterByValueConfig {
    /// Validate all safety invariants and return the compiled [`SafeRegex`].
    ///
    /// Called at construction time so that an invalid config never reaches
    /// `execute`.  The returned regex is ready to use — callers should keep
    /// it rather than re-compiling.
    pub fn validate(&self) -> Result<SafeRegex> {
        if self.column.is_empty() {
            return Err(Error::ConfigurationError(
                "filter_by_value: 'column' must not be empty".into(),
            ));
        }

        // SafeRegex::new handles pattern-level validation (empty, length,
        // nesting, groups, compilation).
        SafeRegex::new(&self.pattern, "filter_by_value")
    }
}
