//! Identity Resolution: Column-major identity resolution
//!
//! ## Algorithm
//!
//! 1. Build a **dedup key** per row by iterating the configured `columns`
//!    list in priority order — the first non-null value wins.
//! 2. Within each dedup-key group, assign the *first* occurrence as the
//!    canonical record (`is_duplicate = false`) and tag subsequent rows
//!    (`is_duplicate = true`).
//! 3. A `canonical_id` column carries the value of `employee_id_column` for
//!    the canonical record so downstream actions can trace merges.
//!
//! Configurable via manifest JSON:
//! ```json
//! {
//!   "columns": ["national_id", "email"],
//!   "employee_id_column": "employee_id"
//! }
//! ```

use crate::capabilities::logic::traits::Deduplicator;
use crate::domain::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Identity deduplication using column-major approach.
///
/// Iterates the configured columns in priority order to build a dedup key
/// per row, then groups rows sharing the same key. The first occurrence is
/// the canonical record; subsequent rows are flagged as duplicates.
///
/// # Output columns
///
/// | Column         | Type   | Description                                           |
/// |----------------|--------|-------------------------------------------------------|
/// | `canonical_id` | string | The `employee_id_column` value of the first occurrence |
/// | `is_duplicate` | bool   | `true` for every row after the first in a group        |
#[derive(Debug, Clone)]
pub struct IdentityDeduplicator {
    config: DedupConfig,
}

impl IdentityDeduplicator {
    pub fn new(config: DedupConfig) -> Self {
        Self { config }
    }

    /// Convenience constructor from manifest JSON.
    pub fn from_action_config(value: &serde_json::Value) -> Self {
        Self::new(DedupConfig::from_json(value))
    }
}

impl Default for IdentityDeduplicator {
    fn default() -> Self {
        Self::new(DedupConfig::default())
    }
}

impl Deduplicator for IdentityDeduplicator {
    fn deduplicate(&self, context: RosterContext) -> Result<RosterContext> {
        self.execute(context)
    }
}

impl OnboardingAction for IdentityDeduplicator {
    fn id(&self) -> &str {
        "identity_deduplicator"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            columns = ?self.config.columns,
            employee_id = %self.config.employee_id_column,
            "IdentityDeduplicator: resolving duplicates"
        );

        // Collect eagerly — dedup logic requires grouped iteration
        let lf = std::mem::replace(&mut context.data, LazyFrame::default());
        let df = lf.collect().map_err(|e| {
            Error::LogicError(format!("Failed to collect for dedup: {}", e))
        })?;

        let schema = df.schema();

        // Filter configured columns to those actually present in the data
        let available_columns: Vec<&String> = self
            .config
            .columns
            .iter()
            .filter(|c| schema.contains(c.as_str()))
            .collect();

        if available_columns.is_empty() {
            tracing::warn!(
                configured = ?self.config.columns,
                "IdentityDeduplicator: none of the configured columns found — skipping"
            );
            context.data = df.lazy();
            return Ok(context);
        }

        let n = df.height();

        // Pre-extract string chunked arrays for each available column
        let column_arrays: Vec<StringChunked> = available_columns
            .iter()
            .map(|name| df.column(name.as_str()).unwrap().str().unwrap().clone())
            .collect();

        let employee_ids = df
            .column(self.config.employee_id_column.as_str())
            .map_err(|e| {
                Error::LogicError(format!(
                    "Employee ID column '{}' not found: {}",
                    self.config.employee_id_column, e
                ))
            })?
            .str()
            .unwrap();

        // Build dedup keys: iterate columns in priority order, first non-null wins
        let dedup_keys: Vec<String> = (0..n)
            .map(|i| {
                for col_arr in &column_arrays {
                    if let Some(val) = col_arr.get(i) {
                        return val.to_string();
                    }
                }
                format!("__unknown_{}", i)
            })
            .collect();

        // Track first occurrence of each dedup key → canonical employee_id
        let mut first_occurrence: std::collections::HashMap<&str, &str> =
            std::collections::HashMap::new();
        let mut canonical_ids: Vec<String> = Vec::with_capacity(n);
        let mut is_duplicate: Vec<bool> = Vec::with_capacity(n);

        for i in 0..n {
            let key = dedup_keys[i].as_str();
            let emp_id = employee_ids.get(i).unwrap_or("unknown");

            if let Some(&canon) = first_occurrence.get(key) {
                canonical_ids.push(canon.to_string());
                is_duplicate.push(true);
            } else {
                first_occurrence.insert(
                    // Safety: dedup_keys lives for the whole loop
                    unsafe { &*(key as *const str) },
                    unsafe { &*(emp_id as *const str) },
                );
                canonical_ids.push(emp_id.to_string());
                is_duplicate.push(false);
            }
        }

        let canonical_col = Column::new("canonical_id".into(), canonical_ids);
        let is_dup_col = Column::new("is_duplicate".into(), is_duplicate);

        let df = df
            .hstack(&[canonical_col, is_dup_col])
            .map_err(|e| Error::LogicError(format!("Failed to append dedup columns: {}", e)))?;

        for col_name in ["canonical_id", "is_duplicate"] {
            context.set_field_source(col_name.to_string(), "LOGIC_ACTION".into());
            context.mark_field_modified(col_name.to_string(), "identity_deduplicator".into());
        }

        context.data = df.lazy();
        Ok(context)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_df_with_email_dupes() -> DataFrame {
        df! {
            "employee_id" => &["001", "002", "003", "004"],
            "first_name"  => &["John", "John", "Jane", "Alice"],
            "email"       => &["john@co.com", "john@co.com", "jane@co.com", "alice@co.com"],
            "salary"      => &[70_000i64, 72_000, 85_000, 92_000],
        }
        .expect("test df")
    }

    fn test_df_with_national_id() -> DataFrame {
        df! {
            "employee_id" => &["001", "002", "003"],
            "national_id" => &[Some("NID-A"), None, Some("NID-A")],
            "email"       => &["john@co.com", "jane@co.com", "johnny@co.com"],
            "salary"      => &[70_000i64, 85_000, 71_000],
        }
        .expect("test df")
    }

    #[test]
    fn test_identity_deduplicator_id() {
        let action = IdentityDeduplicator::default();
        assert_eq!(action.id(), "identity_deduplicator");
    }

    #[test]
    fn test_dedup_by_email() {
        let config = DedupConfig {
            columns: vec!["email".into()],
            ..Default::default()
        };
        let ctx = RosterContext::new(test_df_with_email_dupes().lazy());
        let action = IdentityDeduplicator::new(config);
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let is_dup: Vec<Option<bool>> = df
            .column("is_duplicate").unwrap()
            .bool().unwrap()
            .into_iter()
            .collect();

        assert_eq!(is_dup, vec![Some(false), Some(true), Some(false), Some(false)]);
    }

    #[test]
    fn test_canonical_id_set() {
        let config = DedupConfig {
            columns: vec!["email".into()],
            ..Default::default()
        };
        let ctx = RosterContext::new(test_df_with_email_dupes().lazy());
        let action = IdentityDeduplicator::new(config);
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let canonical: Vec<Option<&str>> = df
            .column("canonical_id").unwrap()
            .str().unwrap()
            .into_iter()
            .collect();

        assert_eq!(canonical[0], Some("001"));
        assert_eq!(canonical[1], Some("001"));
        assert_eq!(canonical[2], Some("003"));
        assert_eq!(canonical[3], Some("004"));
    }

    #[test]
    fn test_dedup_prefers_first_configured_column() {
        // columns: [national_id, email] → national_id takes priority
        let config = DedupConfig {
            columns: vec!["national_id".into(), "email".into()],
            ..Default::default()
        };
        let ctx = RosterContext::new(test_df_with_national_id().lazy());
        let action = IdentityDeduplicator::new(config);
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let is_dup: Vec<Option<bool>> = df
            .column("is_duplicate").unwrap()
            .bool().unwrap()
            .into_iter()
            .collect();

        // 001 (NID-A) and 003 (NID-A) share national_id → 003 is dup
        // 002 has null national_id, falls back to email → unique
        assert_eq!(is_dup[0], Some(false));
        assert_eq!(is_dup[1], Some(false));
        assert_eq!(is_dup[2], Some(true));
    }

    #[test]
    fn test_no_dedup_columns_skips() {
        let config = DedupConfig {
            columns: vec!["national_id".into(), "email".into()],
            ..Default::default()
        };
        let df = df! {
            "employee_id" => &["001"],
            "salary"      => &[50_000i64],
        }
        .unwrap();
        let ctx = RosterContext::new(df.lazy());
        let action = IdentityDeduplicator::new(config);
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        assert!(df.column("canonical_id").is_err());
        assert!(df.column("is_duplicate").is_err());
    }

    #[test]
    fn test_field_metadata() {
        let config = DedupConfig {
            columns: vec!["email".into()],
            ..Default::default()
        };
        let ctx = RosterContext::new(test_df_with_email_dupes().lazy());
        let action = IdentityDeduplicator::new(config);
        let result = action.execute(ctx).expect("execute");

        for col_name in ["canonical_id", "is_duplicate"] {
            let meta = result.field_metadata.get(col_name)
                .unwrap_or_else(|| panic!("metadata for '{}'", col_name));
            assert_eq!(meta.source, "LOGIC_ACTION");
            assert_eq!(meta.modified_by.as_deref(), Some("identity_deduplicator"));
        }
    }

    #[test]
    fn test_config_from_json() {
        let json = serde_json::json!({
            "columns": ["email", "phone"],
            "employee_id_column": "emp_id"
        });
        let config = DedupConfig::from_json(&json);
        assert_eq!(config.columns, vec!["email", "phone"]);
        assert_eq!(config.employee_id_column, "emp_id");
    }

    #[test]
    fn test_config_defaults() {
        let json = serde_json::json!({});
        let config = DedupConfig::from_json(&json);
        assert_eq!(config.columns, vec!["national_id", "email"]);
        assert_eq!(config.employee_id_column, "employee_id");
    }

    #[test]
    fn test_custom_employee_id_column() {
        let df = df! {
            "emp_code"    => &["A1", "A2", "A3"],
            "email"       => &["a@co.com", "a@co.com", "b@co.com"],
        }
        .unwrap();
        let config = DedupConfig {
            columns: vec!["email".into()],
            employee_id_column: "emp_code".into(),
        };
        let ctx = RosterContext::new(df.lazy());
        let action = IdentityDeduplicator::new(config);
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let canonical: Vec<Option<&str>> = df
            .column("canonical_id").unwrap()
            .str().unwrap()
            .into_iter()
            .collect();
        assert_eq!(canonical[0], Some("A1"));
        assert_eq!(canonical[1], Some("A1")); // dup → points to A1
        assert_eq!(canonical[2], Some("A3"));
    }
}
