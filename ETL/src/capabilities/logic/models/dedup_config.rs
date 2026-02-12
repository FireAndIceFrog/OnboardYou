//! Configuration model for the identity deduplicator engine.

/// Configuration for the identity deduplicator.
///
/// Columns are inspected in priority order — the first non-null value across
/// the listed columns becomes the dedup key for that row.
///
/// # JSON config
///
/// ```json
/// {
///   "columns": ["national_id", "email"],
///   "employee_id_column": "employee_id"
/// }
/// ```
///
/// | Field                | Type     | Default                    | Description                                          |
/// |----------------------|----------|----------------------------|------------------------------------------------------|
/// | `columns`            | string[] | `["national_id", "email"]` | Columns to inspect for the dedup key (priority order) |
/// | `employee_id_column` | string   | `"employee_id"`            | Column whose value is used as the canonical ID        |
#[derive(Debug, Clone)]
pub struct DedupConfig {
    /// Columns to inspect (in priority order) when building the dedup key.
    /// The first non-null value across these columns becomes the key.
    pub columns: Vec<String>,
    /// The column that holds the employee identifier (used for canonical_id).
    pub employee_id_column: String,
}

impl Default for DedupConfig {
    fn default() -> Self {
        Self {
            columns: vec!["national_id".into(), "email".into()],
            employee_id_column: "employee_id".into(),
        }
    }
}

impl DedupConfig {
    /// Build from manifest `ActionConfig.config` JSON.
    pub fn from_json(value: &serde_json::Value) -> Self {
        let columns = value
            .get("columns")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_else(|| vec!["national_id".into(), "email".into()]);

        let employee_id_column = value
            .get("employee_id_column")
            .and_then(|v| v.as_str())
            .unwrap_or("employee_id")
            .to_string();

        Self {
            columns,
            employee_id_column,
        }
    }
}
