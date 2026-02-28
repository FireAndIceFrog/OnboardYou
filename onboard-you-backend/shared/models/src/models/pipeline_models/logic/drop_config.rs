//! Configuration model for the drop-column engine.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use utoipa::ToSchema;

/// Configuration for the drop-column action.
///
/// # JSON config
///
/// ```json
/// {
///   "columns": ["col1", "col2"]
/// }
/// ```
///
/// | Field     | Type          | Description                |
/// |-----------|---------------|----------------------------|
/// | `columns` | `[string]`    | List of column names to drop|
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DropConfig {
    /// List of column names to drop.
    pub columns: Vec<String>,
}

impl DropConfig {
    /// Validate that all column names are unique.
    pub fn validate(&self) -> Result<()> {
        let mut seen = HashSet::with_capacity(self.columns.len());
        for col in &self.columns {
            if !seen.insert(col) {
                return Err(Error::LogicError(format!(
                    "drop_column: duplicate column name '{col}'"
                )));
            }
        }
        Ok(())
    }
}
