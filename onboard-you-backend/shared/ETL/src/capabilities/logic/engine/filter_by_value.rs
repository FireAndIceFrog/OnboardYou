//! Filter By Value: Retains only rows where a column matches a regex pattern
//!
//! Designed for ETL filtering tasks such as keeping only employees in a
//! specific department, matching a location pattern, or selecting records
//! that meet a business-rule criterion.
//!
//! # Security model
//!
//! Inherits the same **defence-in-depth** controls as `regex_replace`:
//!
//! | Control                        | Rationale                                                     |
//! |--------------------------------|---------------------------------------------------------------|
//! | Rust `regex` crate only        | Guarantees **linear-time** matching (Thompson NFA) — immune   |
//! |                                | to catastrophic-backtracking ReDoS by construction.           |
//! | Pattern length ≤ 128 chars     | Caps compilation cost and prevents pattern-bomb payloads.     |
//! | Compiled size ≤ 64 KiB         | `RegexBuilder::size_limit` — bounds memory for the NFA/DFA.  |
//! | Exactly 0 or 1 capture groups  | Requirement: single match-group only.                         |
//! | Nesting depth ≤ 3              | Rejects deeply nested groups like `(((...)))`.                |
//!
//! # Manifest JSON
//!
//! ```json
//! {
//!   "column": "department",
//!   "pattern": "^Engineering$"
//! }
//! ```

use onboard_you_models::{FilterByValueConfig, SafeRegex};
use onboard_you_models::ColumnCalculator;
use onboard_you_models::PipelineWarning;
use onboard_you_models::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Filter rows in a DataFrame, keeping only those where a string column
/// matches the given regex pattern.
///
/// See the module-level documentation for the full security model.
#[derive(Debug)]
pub struct FilterByValue {
    config: FilterByValueConfig,
    /// Pre-compiled, safety-validated regex.
    regex: SafeRegex,
}

impl FilterByValue {
    /// Construct from a pre-validated config and its compiled regex.
    fn new(config: FilterByValueConfig, regex: SafeRegex) -> Self {
        Self { config, regex }
    }

    /// Construct from a deserialised config.
    pub fn from_action_config(config: &FilterByValueConfig) -> Result<Self> {
        let regex = config.validate()?;
        Ok(Self::new(config.clone(), regex))
    }

    /// Test whether a single value matches the filter pattern.
    fn matches(&self, value: &str) -> bool {
        self.regex.is_match(value)
    }
}

impl ColumnCalculator for FilterByValue {
    fn calculate_columns(&self, context: RosterContext) -> Result<RosterContext> {
        // Schema-only pass: filtering does not add or remove columns, so
        // return unchanged.
        Ok(context)
    }
}

impl OnboardingAction for FilterByValue {
    fn id(&self) -> &str {
        "filter_by_value"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            column = %self.config.column,
            pattern = %self.config.pattern,
            "FilterByValue: filtering rows by column regex match"
        );

        // Collect to apply row-wise string matching.
        let df = context
            .get_data()
            .clone()
            .collect()
            .map_err(|e| Error::LogicError(format!("Failed to collect LazyFrame: {e}")))?;

        let total_rows = df.height();

        let series = df.column(&self.config.column).map_err(|e| {
            Error::LogicError(format!(
                "filter_by_value: column '{}' not found: {e}",
                self.config.column
            ))
        })?;

        let ca = series.str().map_err(|e| {
            Error::LogicError(format!(
                "filter_by_value: column '{}' is not a string column: {e}",
                self.config.column
            ))
        })?;

        // Build a boolean mask: true for rows whose value matches the regex.
        // Null values never match (they are dropped).
        let mask: BooleanChunked = ca
            .into_iter()
            .map(|opt: Option<&str>| opt.map(|s| self.matches(s)).unwrap_or(false))
            .collect();

        let filtered = df.filter(&mask).map_err(|e| {
            Error::LogicError(format!(
                "filter_by_value: failed to filter by column '{}': {e}",
                self.config.column
            ))
        })?;

        let kept = filtered.height();
        let dropped = total_rows - kept;
        if dropped > 0 {
                context.deps.logger.warn(PipelineWarning {
                    action_id: self.id().to_string(),
                    message: format!(
                        "{dropped} of {total_rows} row(s) did not match filter on column '{}'",
                        self.config.column
                    ),
                    count: dropped,
                    detail: None,
                });
        }

        context.set_data(filtered.lazy());

        Ok(context)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use onboard_you_models::ETLDependancies;
    use super::*;
    use onboard_you_models::MAX_PATTERN_LEN;
    use polars::df;

    fn sample_df() -> DataFrame {
        df! {
            "employee_id" => &["001", "002", "003", "004"],
            "department"  => &["Engineering", "Sales", "Engineering", "HR"],
            "location"    => &["London", "New York", "London", "Berlin"],
        }
        .unwrap()
    }

    // -----------------------------------------------------------------------
    // Construction & ID
    // -----------------------------------------------------------------------

    #[test]
    fn test_id() {
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "department",
            "pattern": "Engineering"
        }))
        .unwrap();
        let action = FilterByValue::from_action_config(&cfg).unwrap();
        assert_eq!(action.id(), "filter_by_value");
    }

    // -----------------------------------------------------------------------
    // Functional: basic filtering
    // -----------------------------------------------------------------------

    #[test]
    fn test_basic_filter() {
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "department",
            "pattern": "^Engineering$"
        }))
        .unwrap();
        let action = FilterByValue::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::with_deps(sample_df().lazy(), ETLDependancies::default());
        let result = action.execute(ctx).unwrap();
        let df = result.get_data().collect().unwrap();

        assert_eq!(df.height(), 2);
        let dept = df.column("department").unwrap().str().unwrap();
        assert_eq!(dept.get(0).unwrap(), "Engineering");
        assert_eq!(dept.get(1).unwrap(), "Engineering");

        let ids = df.column("employee_id").unwrap().str().unwrap();
        assert_eq!(ids.get(0).unwrap(), "001");
        assert_eq!(ids.get(1).unwrap(), "003");
    }

    #[test]
    fn test_partial_match_filter() {
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "location",
            "pattern": "on"
        }))
        .unwrap();
        let action = FilterByValue::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::with_deps(sample_df().lazy(), ETLDependancies::default());
        let result = action.execute(ctx).unwrap();
        let df = result.get_data().collect().unwrap();

        // "London" contains "on" (matches twice), no other city contains "on"
        assert_eq!(df.height(), 2);
        let loc = df.column("location").unwrap().str().unwrap();
        assert_eq!(loc.get(0).unwrap(), "London");
        assert_eq!(loc.get(1).unwrap(), "London");
    }

    #[test]
    fn test_no_match_returns_empty() {
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "department",
            "pattern": "^Finance$"
        }))
        .unwrap();
        let action = FilterByValue::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::with_deps(sample_df().lazy(), ETLDependancies::default());
        let result = action.execute(ctx).unwrap();
        let df = result.get_data().collect().unwrap();

        assert_eq!(df.height(), 0);
    }

    #[test]
    fn test_all_match() {
        let df = df! {
            "val" => &["abc", "abcdef", "xabc"],
        }
        .unwrap();
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "val",
            "pattern": "abc"
        }))
        .unwrap();
        let action = FilterByValue::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::with_deps(df.lazy(), ETLDependancies::default());
        let result = action.execute(ctx).unwrap();
        let collected = result.get_data().collect().unwrap();
        assert_eq!(collected.height(), 3);
    }

    // -----------------------------------------------------------------------
    // Null handling
    // -----------------------------------------------------------------------

    #[test]
    fn test_null_values_dropped() {
        let s = Series::new("val".into(), &[Some("abc"), None, Some("def")]);
        let df = DataFrame::new_infer_height(vec![s.into()]).unwrap();
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "val",
            "pattern": "."
        }))
        .unwrap();
        let action = FilterByValue::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::with_deps(df.lazy(), ETLDependancies::default());
        let result = action.execute(ctx).unwrap();
        let collected = result.get_data().collect().unwrap();

        // Null row is dropped; the two non-null rows match "."
        assert_eq!(collected.height(), 2);
        let col = collected.column("val").unwrap().str().unwrap();
        assert_eq!(col.get(0).unwrap(), "abc");
        assert_eq!(col.get(1).unwrap(), "def");
    }

    // -----------------------------------------------------------------------
    // Empty strings
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_string_values() {
        let df = df! {
            "val" => &["", "abc", ""],
        }
        .unwrap();
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "val",
            "pattern": "^.+$"
        }))
        .unwrap();
        let action = FilterByValue::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::with_deps(df.lazy(), ETLDependancies::default());
        let result = action.execute(ctx).unwrap();
        let collected = result.get_data().collect().unwrap();

        // Only "abc" matches ^.+$ (non-empty)
        assert_eq!(collected.height(), 1);
        let col = collected.column("val").unwrap().str().unwrap();
        assert_eq!(col.get(0).unwrap(), "abc");
    }

    // -----------------------------------------------------------------------
    // Configuration validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_missing_column_field() {
        // Missing required 'column' field is now caught at deserialization
        assert!(
            serde_json::from_value::<FilterByValueConfig>(serde_json::json!({
                "pattern": "x"
            }))
            .is_err()
        );
    }

    #[test]
    fn test_missing_pattern_field() {
        // Missing required 'pattern' field is now caught at deserialization
        assert!(
            serde_json::from_value::<FilterByValueConfig>(serde_json::json!({
                "column": "department"
            }))
            .is_err()
        );
    }

    #[test]
    fn test_empty_pattern_rejected() {
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "department",
            "pattern": ""
        }))
        .unwrap();
        assert!(FilterByValue::from_action_config(&cfg).is_err());
    }

    #[test]
    fn test_empty_column_rejected() {
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "",
            "pattern": "x"
        }))
        .unwrap();
        assert!(FilterByValue::from_action_config(&cfg).is_err());
    }

    // -----------------------------------------------------------------------
    // Security: attack-vector rejection
    // -----------------------------------------------------------------------

    #[test]
    fn test_pattern_length_limit() {
        let long_pattern = "a".repeat(MAX_PATTERN_LEN + 1);
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "department",
            "pattern": long_pattern
        }))
        .unwrap();
        let result = FilterByValue::from_action_config(&cfg);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("pattern length"), "unexpected error: {err}");
    }

    #[test]
    fn test_excessive_nesting_rejected() {
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "department",
            "pattern": "((((a))))"
        }))
        .unwrap();
        let result = FilterByValue::from_action_config(&cfg);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nesting depth"), "unexpected error: {err}");
    }

    #[test]
    fn test_multiple_capture_groups_rejected() {
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "department",
            "pattern": "(a)(b)(c)"
        }))
        .unwrap();
        let result = FilterByValue::from_action_config(&cfg);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("capture group"), "unexpected error: {err}");
    }

    #[test]
    fn test_non_capturing_groups_allowed() {
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "department",
            "pattern": "(?:Eng|Sale)s?"
        }))
        .unwrap();
        let action = FilterByValue::from_action_config(&cfg);
        assert!(action.is_ok(), "non-capturing groups should be accepted");
    }

    #[test]
    fn test_invalid_regex_rejected() {
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "department",
            "pattern": "[invalid"
        }))
        .unwrap();
        let result = FilterByValue::from_action_config(&cfg);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid pattern"), "unexpected error: {err}");
    }

    #[test]
    fn test_missing_column_at_runtime() {
        let df = df! {
            "other" => &["x"],
        }
        .unwrap();
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "department",
            "pattern": "x"
        }))
        .unwrap();
        let action = FilterByValue::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::with_deps(df.lazy(), ETLDependancies::default());
        let result = action.execute(ctx);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Warning tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_filter_emits_warning_for_dropped_rows() {
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "department",
            "pattern": "^Engineering$"
        }))
        .unwrap();
        let action = FilterByValue::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::with_deps(sample_df().lazy(), ETLDependancies::default());
        let result = action.execute(ctx).unwrap();
        let df = result.get_data().collect().unwrap();

        // 2 of 4 rows matched Engineering, so 2 were dropped
        assert_eq!(df.height(), 2);
        let warnings = result.deps.logger.drain_deferred_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].action_id, "filter_by_value");
        assert_eq!(warnings[0].count, 2);
        assert!(warnings[0].message.contains("2 of 4"));
        assert!(warnings[0].message.contains("department"));
    }

    #[test]
    fn test_filter_all_match_no_warning() {
        let df = df! {
            "val" => &["abc", "abc"],
        }
        .unwrap();
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "val",
            "pattern": "abc"
        }))
        .unwrap();
        let action = FilterByValue::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::with_deps(df.lazy(), ETLDependancies::default());
        let result = action.execute(ctx).unwrap();

        let warnings = result.deps.logger.drain_deferred_warnings();
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_filter_no_match_emits_warning_for_all_rows() {
        let cfg: FilterByValueConfig = serde_json::from_value(serde_json::json!({
            "column": "department",
            "pattern": "^Finance$"
        }))
        .unwrap();
        let action = FilterByValue::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::with_deps(sample_df().lazy(), ETLDependancies::default());
        let result = action.execute(ctx).unwrap();
        let df = result.get_data().collect().unwrap();

        assert_eq!(df.height(), 0);
        let warnings = result.deps.logger.drain_deferred_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].count, 4);
        assert!(warnings[0].message.contains("4 of 4"));
    }
}
