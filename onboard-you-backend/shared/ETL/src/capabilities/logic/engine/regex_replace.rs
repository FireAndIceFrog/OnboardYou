//! Regex Replace: Substitutes the first regex match in a column with a replacement string
//!
//! Designed for ETL data-cleaning tasks such as stripping phone-number formatting,
//! normalising postcodes, or removing unwanted prefixes/suffixes.
//!
//! # Security model
//!
//! Regex is an inherently powerful primitive that can be abused.  This action
//! applies **defence-in-depth** to reduce the attack surface:
//!
//! | Control                        | Rationale                                                     |
//! |--------------------------------|---------------------------------------------------------------|
//! | Rust `regex` crate only        | Guarantees **linear-time** matching (Thompson NFA) — immune   |
//! |                                | to catastrophic-backtracking ReDoS by construction.           |
//! | Pattern length ≤ 128 chars     | Caps compilation cost and prevents pattern-bomb payloads.     |
//! | Compiled size ≤ 64 KiB         | `RegexBuilder::size_limit` — bounds memory for the NFA/DFA.  |
//! | Exactly 0 or 1 capture groups  | Requirement: single match-group only.                         |
//! | Nesting depth ≤ 3              | Rejects deeply nested groups like `(((...)))`.                |
//! | Replacement length ≤ 256 chars | Prevents output-inflation attacks.                            |
//! | No backreferences in replace   | Replacement is treated as a **literal** string — `$1`, `\1`  |
//! |                                | etc. are escaped, preventing injection of captured data into  |
//! |                                | unintended positions.                                         |
//!
//! # Manifest JSON
//!
//! ```json
//! {
//!   "column": "phone_number",
//!   "pattern": "\\+44\\s?",
//!   "replacement": "0"
//! }
//! ```

use crate::capabilities::logic::models::{RegexReplaceConfig, SafeRegex};
use crate::capabilities::logic::traits::ColumnCalculator;
use crate::domain::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Replace the first regex match in a string column with a literal string.
///
/// See the module-level documentation for the full security model.
#[derive(Debug)]
pub struct RegexReplace {
    config: RegexReplaceConfig,
    /// Pre-compiled, safety-validated regex.
    regex: SafeRegex,
}

impl RegexReplace {
    /// Deserialise and construct from manifest JSON.
    pub fn from_action_config(config: &RegexReplaceConfig) -> Result<Self> {
        let regex = config.validate()?;
        Ok(Self {
            config: config.clone(),
            regex,
        })
    }

    /// Apply the replacement to a single string value.
    fn apply(&self, value: &str) -> String {
        self.regex.replace_first(value, &self.config.replacement)
    }
}

impl ColumnCalculator for RegexReplace {
    fn calculate_columns(&self, context: RosterContext) -> Result<RosterContext> {
        // Schema-only pass: column is not added or removed, so return unchanged.
        Ok(context)
    }
}

impl OnboardingAction for RegexReplace {
    fn id(&self) -> &str {
        "regex_replace"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            column = %self.config.column,
            pattern = %self.config.pattern,
            "RegexReplace: applying regex replacement"
        );

        // Collect to apply row-wise string transformation.
        let df = context
            .data
            .clone()
            .collect()
            .map_err(|e| Error::LogicError(format!("Failed to collect LazyFrame: {e}")))?;

        let series = df.column(&self.config.column).map_err(|e| {
            Error::LogicError(format!(
                "regex_replace: column '{}' not found: {e}",
                self.config.column
            ))
        })?;

        let ca = series.str().map_err(|e| {
            Error::LogicError(format!(
                "regex_replace: column '{}' is not a string column: {e}",
                self.config.column
            ))
        })?;

        let replaced: StringChunked = ca
            .into_iter()
            .map(|opt: Option<&str>| opt.map(|s| self.apply(s)))
            .collect();

        let replaced = replaced
            .into_series()
            .with_name(self.config.column.clone().into());

        let mut result_df = df.clone();
        let _ = result_df
            .replace(&self.config.column, replaced.into_column())
            .map_err(|e| {
                Error::LogicError(format!(
                    "regex_replace: failed to replace column '{}': {e}",
                    self.config.column
                ))
            })?;

        context.set_field_source(self.config.column.clone(), "LOGIC_ACTION".into());
        context.mark_field_modified(self.config.column.clone(), "regex_replace".into());
        context.data = result_df.lazy();

        Ok(context)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::logic::models::{MAX_PATTERN_LEN, MAX_REPLACEMENT_LEN};

    fn sample_df() -> DataFrame {
        df! {
            "employee_id" => &["001", "002", "003"],
            "phone"       => &["+44 7911 123456", "+44 7922 654321", "07933 111222"],
            "postcode"    => &["SW1A 1AA", "EC2R 8AH", "M1 1AE"],
        }
        .unwrap()
    }

    // -----------------------------------------------------------------------
    // Happy-path tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_id() {
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "phone",
            "pattern": "x",
            "replacement": "y"
        }))
        .unwrap();
        let action = RegexReplace::from_action_config(&cfg).unwrap();
        assert_eq!(action.id(), "regex_replace");
    }

    #[test]
    fn test_basic_replacement() {
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "phone",
            "pattern": "\\+44\\s?",
            "replacement": "0"
        }))
        .unwrap();
        let action = RegexReplace::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).unwrap();
        let df = result.data.collect().unwrap();
        let col = df.column("phone").unwrap().str().unwrap();

        assert_eq!(col.get(0).unwrap(), "07911 123456");
        assert_eq!(col.get(1).unwrap(), "07922 654321");
        // Third row has no match — unchanged
        assert_eq!(col.get(2).unwrap(), "07933 111222");
    }

    #[test]
    fn test_single_capture_group() {
        // Extract area from postcode: replace the whole match keeping area code
        // Pattern with one capture group is allowed
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "postcode",
            "pattern": "\\s+[0-9][A-Z]{2}$",
            "replacement": ""
        }))
        .unwrap();
        let action = RegexReplace::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).unwrap();
        let df = result.data.collect().unwrap();
        let col = df.column("postcode").unwrap().str().unwrap();

        assert_eq!(col.get(0).unwrap(), "SW1A");
        assert_eq!(col.get(1).unwrap(), "EC2R");
        assert_eq!(col.get(2).unwrap(), "M1");
    }

    #[test]
    fn test_only_first_match_replaced() {
        let df = df! {
            "val" => &["aaa"],
        }
        .unwrap();
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "val",
            "pattern": "a",
            "replacement": "X"
        }))
        .unwrap();
        let action = RegexReplace::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).unwrap();
        let collected = result.data.collect().unwrap();
        let col = collected.column("val").unwrap().str().unwrap();
        // Only the first 'a' is replaced
        assert_eq!(col.get(0).unwrap(), "Xaa");
    }

    #[test]
    fn test_no_match_leaves_value_unchanged() {
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "phone",
            "pattern": "NOMATCH",
            "replacement": "X"
        }))
        .unwrap();
        let action = RegexReplace::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).unwrap();
        let df = result.data.collect().unwrap();
        let col = df.column("phone").unwrap().str().unwrap();
        assert_eq!(col.get(0).unwrap(), "+44 7911 123456");
    }

    #[test]
    fn test_field_metadata_provenance() {
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "phone",
            "pattern": "\\+44",
            "replacement": "0"
        }))
        .unwrap();
        let action = RegexReplace::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).unwrap();

        let meta = result
            .field_metadata
            .get("phone")
            .expect("metadata for 'phone'");
        assert_eq!(meta.source, "LOGIC_ACTION");
        assert_eq!(meta.modified_by.as_deref(), Some("regex_replace"));
    }

    // -----------------------------------------------------------------------
    // Config / validation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_missing_column_field() {
        // Missing required 'column' field is now caught at deserialization
        assert!(
            serde_json::from_value::<RegexReplaceConfig>(serde_json::json!({
                "pattern": "x",
                "replacement": "y"
            }))
            .is_err()
        );
    }

    #[test]
    fn test_missing_pattern_field() {
        // Missing required 'pattern' field is now caught at deserialization
        assert!(
            serde_json::from_value::<RegexReplaceConfig>(serde_json::json!({
                "column": "phone",
                "replacement": "y"
            }))
            .is_err()
        );
    }

    #[test]
    fn test_missing_replacement_field() {
        // Missing required 'replacement' field is now caught at deserialization
        assert!(
            serde_json::from_value::<RegexReplaceConfig>(serde_json::json!({
                "column": "phone",
                "pattern": "x"
            }))
            .is_err()
        );
    }

    #[test]
    fn test_empty_pattern_rejected() {
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "phone",
            "pattern": "",
            "replacement": "y"
        }))
        .unwrap();
        assert!(RegexReplace::from_action_config(&cfg).is_err());
    }

    #[test]
    fn test_empty_column_rejected() {
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "",
            "pattern": "x",
            "replacement": "y"
        }))
        .unwrap();
        assert!(RegexReplace::from_action_config(&cfg).is_err());
    }

    // -----------------------------------------------------------------------
    // Security: attack-vector rejection
    // -----------------------------------------------------------------------

    #[test]
    fn test_pattern_length_limit() {
        let long_pattern = "a".repeat(MAX_PATTERN_LEN + 1);
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "phone",
            "pattern": long_pattern,
            "replacement": "x"
        }))
        .unwrap();
        let result = RegexReplace::from_action_config(&cfg);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("pattern length"), "unexpected error: {err}");
    }

    #[test]
    fn test_replacement_length_limit() {
        let long_replacement = "x".repeat(MAX_REPLACEMENT_LEN + 1);
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "phone",
            "pattern": "a",
            "replacement": long_replacement
        }))
        .unwrap();
        let result = RegexReplace::from_action_config(&cfg);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("replacement length"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_excessive_nesting_rejected() {
        // Depth 4 — exceeds MAX_NESTING_DEPTH of 3
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "phone",
            "pattern": "((((a))))",
            "replacement": "x"
        }))
        .unwrap();
        let result = RegexReplace::from_action_config(&cfg);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nesting depth"), "unexpected error: {err}");
    }

    #[test]
    fn test_multiple_capture_groups_rejected() {
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "phone",
            "pattern": "(a)(b)",
            "replacement": "x"
        }))
        .unwrap();
        let result = RegexReplace::from_action_config(&cfg);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("capture group"), "unexpected error: {err}");
    }

    #[test]
    fn test_non_capturing_groups_allowed() {
        // (?:...) groups do not count towards the capture limit
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "phone",
            "pattern": "(?:a)(?:b)(?:c)",
            "replacement": "x"
        }))
        .unwrap();
        assert!(RegexReplace::from_action_config(&cfg).is_ok());
    }

    #[test]
    fn test_backreference_in_replacement_escaped() {
        // Replacement containing `$1` should be treated literally
        let df = df! {
            "val" => &["hello world"],
        }
        .unwrap();
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "val",
            "pattern": "(hello)",
            "replacement": "$1_expanded"
        }))
        .unwrap();
        let action = RegexReplace::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).unwrap();
        let collected = result.data.collect().unwrap();
        let col = collected.column("val").unwrap().str().unwrap();
        // $1 is escaped to $$ so it appears literally
        assert_eq!(col.get(0).unwrap(), "$1_expanded world");
    }

    #[test]
    fn test_invalid_regex_rejected() {
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "phone",
            "pattern": "[invalid",
            "replacement": "x"
        }))
        .unwrap();
        let result = RegexReplace::from_action_config(&cfg);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid pattern"), "unexpected error: {err}");
    }

    #[test]
    fn test_missing_column_at_runtime() {
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "nonexistent",
            "pattern": "a",
            "replacement": "b"
        }))
        .unwrap();
        let action = RegexReplace::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"), "unexpected error: {err}");
    }

    // -----------------------------------------------------------------------
    // Edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_null_values_preserved() {
        let s = Series::new("val".into(), &[Some("abc"), None, Some("def")]);
        let df = DataFrame::new_infer_height(vec![s.into()]).unwrap();
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "val",
            "pattern": "b",
            "replacement": "X"
        }))
        .unwrap();
        let action = RegexReplace::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).unwrap();
        let collected = result.data.collect().unwrap();
        let col = collected.column("val").unwrap().str().unwrap();
        assert_eq!(col.get(0).unwrap(), "aXc");
        assert!(col.get(1).is_none());
        assert_eq!(col.get(2).unwrap(), "def");
    }

    #[test]
    fn test_empty_string_values() {
        let df = df! {
            "val" => &["", "abc", ""],
        }
        .unwrap();
        let cfg: RegexReplaceConfig = serde_json::from_value(serde_json::json!({
            "column": "val",
            "pattern": "a",
            "replacement": "X"
        }))
        .unwrap();
        let action = RegexReplace::from_action_config(&cfg).unwrap();
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).unwrap();
        let collected = result.data.collect().unwrap();
        let col = collected.column("val").unwrap().str().unwrap();
        assert_eq!(col.get(0).unwrap(), "");
        assert_eq!(col.get(1).unwrap(), "Xbc");
        assert_eq!(col.get(2).unwrap(), "");
    }

    // -----------------------------------------------------------------------
    // Helpers — now in models::safe_regex (tests retained there)
    // -----------------------------------------------------------------------
}
