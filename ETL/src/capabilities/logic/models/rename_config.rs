//! Configuration model for the rename-column engine.

use crate::domain::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Configuration for the rename-column action.
///
/// # JSON config
///
/// ```json
/// {
///   "mapping": {
///     "first_name": "given_name",
///     "last_name": "surname"
///   }
/// }
/// ```
///
/// | Field     | Type                    | Description                               |
/// |-----------|-------------------------|-------------------------------------------|
/// | `mapping` | `{ from: to, … }`       | Dictionary of source → target column names |
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RenameConfig {
    /// Source → target column name mapping.
    pub mapping: HashMap<String, String>,
}

impl RenameConfig {
    /// Validate that all target column names are unique.
    ///
    /// Returns `Err` if two or more source columns map to the same target name.
    pub fn validate(&self) -> Result<()> {
        let mut seen = HashSet::with_capacity(self.mapping.len());
        for target in self.mapping.values() {
            if !seen.insert(target) {
                return Err(Error::LogicError(format!(
                    "rename_column: duplicate target column name '{target}'"
                )));
            }
        }
        Ok(())
    }
}
