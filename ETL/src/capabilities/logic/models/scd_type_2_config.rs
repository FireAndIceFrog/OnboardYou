//! Configuration model for SCD Type 2 effective dating.

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
#[derive(Debug, Clone)]
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

impl ScdType2Config {
    /// Build from manifest `ActionConfig.config` JSON.
    pub fn from_json(value: &serde_json::Value) -> Self {
        let entity_column = value
            .get("entity_column")
            .and_then(|v| v.as_str())
            .unwrap_or("employee_id")
            .to_string();

        let date_column = value
            .get("date_column")
            .and_then(|v| v.as_str())
            .unwrap_or("start_date")
            .to_string();

        Self {
            entity_column,
            date_column,
        }
    }
}
