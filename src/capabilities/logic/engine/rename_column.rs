//! Rename columns according to a mapping
//!
//! Configurable via manifest JSON:
//! ```json
//! {
//!   "mapping": {
//!     "old_name": "new_name",
//!     "another_old": "another_new"
//!   }
//! }
//! ```

use crate::domain::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;
use serde::Deserialize;

use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

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
#[derive(Debug, Clone, Deserialize)]
pub struct RenameConfig {
    /// Source → target column name mapping.
    pub mapping: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Rename columns according to a `{ from: to }` mapping.
///
/// Validates that all target names are unique, then applies a lazy rename
/// so the operation is folded into the Polars query plan without
/// materialising the frame.
#[derive(Debug, Clone)]
pub struct RenameColumn {
    config: RenameConfig,
}

impl RenameColumn {
    pub fn new(config: RenameConfig) -> Self {
        Self { config }
    }

    /// Deserialise and construct from manifest JSON.
    pub fn from_action_config(value: &serde_json::Value) -> Result<Self> {
        let config: RenameConfig = serde_json::from_value(value.clone())?;
        config.validate()?;
        Ok(Self::new(config))
    }
}

impl OnboardingAction for RenameColumn {
    fn id(&self) -> &str {
        "rename_column"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!(mapping = ?self.config.mapping, "RenameColumn: applying mappings");

        let lf = std::mem::replace(&mut context.data, LazyFrame::default());

        let old: Vec<&str> = self.config.mapping.keys().map(|k| k.as_str()).collect();
        let new: Vec<&str> = self.config.mapping.values().map(|v| v.as_str()).collect();

        let lf = lf.rename(old, new, true);

        for to in self.config.mapping.values() {
            context.set_field_source(to.clone(), "LOGIC_ACTION".into());
            context.mark_field_modified(to.clone(), "rename_column".into());
        }

        context.data = lf;
        Ok(context)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_df() -> DataFrame {
        df! {
            "employee_id" => &["001", "002"],
            "first_name"  => &["John", "Jane"],
            "last_name"   => &["Doe", "Roe"],
        }
        .unwrap()
    }

    #[test]
    fn test_id() {
        let config = RenameConfig { mapping: HashMap::new() };
        let act = RenameColumn::new(config);
        assert_eq!(act.id(), "rename_column");
    }

    #[test]
    fn test_rename_columns() {
        let json = serde_json::json!({
            "mapping": { "first_name": "given_name", "last_name": "surname" }
        });
        let action = RenameColumn::from_action_config(&json).expect("valid config");
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        assert!(df.column("given_name").is_ok());
        assert!(df.column("surname").is_ok());
        assert!(df.column("first_name").is_err());
        assert!(df.column("last_name").is_err());
    }

    #[test]
    fn test_duplicate_targets_rejected_at_construction() {
        let json = serde_json::json!({
            "mapping": { "first_name": "name", "last_name": "name" }
        });
        let res = RenameColumn::from_action_config(&json);
        assert!(res.is_err());
    }

    #[test]
    fn test_missing_mapping_key_rejected() {
        let json = serde_json::json!({ "first_name": "given_name" });
        let res = RenameColumn::from_action_config(&json);
        assert!(res.is_err(), "should fail without a 'mapping' key");
    }

    #[test]
    fn test_field_metadata() {
        let json = serde_json::json!({
            "mapping": { "first_name": "given_name" }
        });
        let action = RenameColumn::from_action_config(&json).expect("valid config");
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");

        let meta = result.field_metadata.get("given_name")
            .expect("metadata for 'given_name'");
        assert_eq!(meta.source, "LOGIC_ACTION");
        assert_eq!(meta.modified_by.as_deref(), Some("rename_column"));
    }

    #[test]
    fn test_config_deserialise() {
        let json = serde_json::json!({
            "mapping": { "a": "b", "c": "d" }
        });
        let config: RenameConfig = serde_json::from_value(json).expect("deserialise");
        assert_eq!(config.mapping.len(), 2);
        assert_eq!(config.mapping.get("a").unwrap(), "b");
    }

    #[test]
    fn test_validate_ok() {
        let config = RenameConfig {
            mapping: HashMap::from([
                ("a".into(), "b".into()),
                ("c".into(), "d".into()),
            ]),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_duplicate_targets() {
        let config = RenameConfig {
            mapping: HashMap::from([
                ("a".into(), "z".into()),
                ("b".into(), "z".into()),
            ]),
        };
        assert!(config.validate().is_err());
    }
}
