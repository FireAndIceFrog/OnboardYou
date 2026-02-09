//! Rename columns according to a mapping
//!
//! Configurable via manifest JSON:
//! ```json
//! { "mapping": { "old_name": "new_name", "a": "b" } }
//! ```

use crate::domain::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;

use std::collections::HashSet;

/// Configuration for the rename action.
#[derive(Debug, Clone)]
pub struct RenameConfig {
    /// Ordered mapping from source column -> target column
    pub mapping: Vec<(String, String)>,
}

impl Default for RenameConfig {
    fn default() -> Self {
        Self { mapping: Vec::new() }
    }
}

impl RenameConfig {
    /// Build from manifest `ActionConfig.config` JSON.
    /// Supports either `{ "mapping": { ... } }` or a top-level object mapping.
    pub fn from_json(value: &serde_json::Value) -> Self {
        // Prefer explicit `mapping` object
        let obj = if let Some(m) = value.get("mapping") {
            m
        } else {
            value
        };

        let mut mapping = Vec::new();
        if let Some(map) = obj.as_object() {
            for (k, v) in map.iter() {
                if let Some(s) = v.as_str() {
                    mapping.push((k.clone(), s.to_string()));
                }
            }
        }

        Self { mapping }
    }
}

/// Rename columns according to the provided mapping. Validates that all
/// target column names (`to`) are unique before applying.
#[derive(Debug, Clone)]
pub struct RenameColumn {
    config: RenameConfig,
}

impl RenameColumn {
    pub fn new(config: RenameConfig) -> Self {
        Self { config }
    }

    pub fn from_action_config(value: &serde_json::Value) -> Self {
        Self::new(RenameConfig::from_json(value))
    }
}

impl Default for RenameColumn {
    fn default() -> Self {
        Self::new(RenameConfig::default())
    }
}

impl OnboardingAction for RenameColumn {
    fn id(&self) -> &str {
        "rename_column"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!(mapping = ?self.config.mapping, "RenameColumn: applying mappings");

        // Validate uniqueness of target names
        let targets: Vec<&String> = self.config.mapping.iter().map(|(_, to)| to).collect();
        let uniq: HashSet<&&String> = targets.iter().collect();
        if uniq.len() != targets.len() {
            return Err(Error::LogicError("rename_column: target column names must be unique".into()));
        }

        let lf = std::mem::replace(&mut context.data, LazyFrame::default());

        let old: Vec<&str> = self.config.mapping.iter().map(|(f, _)| f.as_str()).collect();
        let new: Vec<&str> = self.config.mapping.iter().map(|(_, t)| t.as_str()).collect();

        let lf = lf.rename(old, new, true);

        for (_, to) in &self.config.mapping {
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
        let act = RenameColumn::default();
        assert_eq!(act.id(), "rename_column");
    }

    #[test]
    fn test_rename_columns() {
        let df = sample_df();
        let ctx = RosterContext::new(df.lazy());

        let json = serde_json::json!({ "mapping": { "first_name": "given_name", "last_name": "surname" } });
        let action = RenameColumn::from_action_config(&json);
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        assert!(df.column("given_name").is_ok());
        assert!(df.column("surname").is_ok());
        assert!(df.column("first_name").is_err());
        assert!(df.column("last_name").is_err());
    }

    #[test]
    fn test_duplicate_targets_error() {
        let df = sample_df();
        let ctx = RosterContext::new(df.lazy());

        let json = serde_json::json!({ "mapping": { "first_name": "name", "last_name": "name" } });
        let action = RenameColumn::from_action_config(&json);
        let res = action.execute(ctx);
        assert!(res.is_err());
    }
}
