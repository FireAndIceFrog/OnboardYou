//! Configuration model for SCD Type 2 effective dating.

use serde::{Deserialize, Serialize};

/// Configuration for SCD Type 2 effective dating.
///
/// # JSON config
///
/// ```json
/// {
///   "entity_column": "employee_id",
///   "date_column": "start_date"
/// }
/// ```
///
/// | Field           | Type   | Default         | Description                                      |
/// |-----------------|--------|-----------------|--------------------------------------------------|
/// | `entity_column` | string | `"employee_id"` | Column that identifies the entity (partition key) |
/// | `date_column`   | string | `"start_date"`  | Column holding the date used for versioning       |
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ScdType2Config {
    /// The column that identifies the entity (partitioning column).
    pub entity_column: String,
    /// The column that holds the date / timestamp to use for versioning.
    pub date_column: String,
}

impl Default for ScdType2Config {
    fn default() -> Self {
        Self {
            entity_column: "employee_id".into(),
            date_column: "start_date".into(),
        }
    }
}
