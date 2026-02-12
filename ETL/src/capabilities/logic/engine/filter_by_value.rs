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

use crate::capabilities::logic::traits::ColumnCalculator;
use crate::domain::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;
use regex::Regex;

// ---------------------------------------------------------------------------
// Hard limits (compile-time constants — not user-configurable)
// ---------------------------------------------------------------------------

/// Maximum length of the raw pattern string.
const MAX_PATTERN_LEN: usize = 128;

/// Maximum compiled NFA/DFA size in bytes (64 KiB).
const MAX_COMPILED_SIZE: usize = 64 * 1024;

/// Maximum nesting depth of parenthesised groups.
const MAX_NESTING_DEPTH: usize = 3;

/// Maximum number of capture groups (excluding the implicit group 0).
const MAX_CAPTURE_GROUPS: usize = 1;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the filter-by-value action.
///
/// | Field    | Type   | Description                                             |
/// |----------|--------|---------------------------------------------------------|
/// | `column` | string | Target column whose values are tested against the regex |
/// | `pattern`| string | Regex pattern (Rust `regex` syntax); rows that match    |
/// |          |        | are **kept**, non-matching rows are dropped              |
#[derive(Debug, Clone)]
pub struct FilterByValueConfig {
    /// Column to filter on.
    pub column: String,
    /// The raw regex pattern.
    pub pattern: String,
}

// ---------------------------------------------------------------------------
// Validation helpers (shared logic with regex_replace)
// ---------------------------------------------------------------------------

/// Count the maximum nesting depth of parenthesised groups in a pattern.
///
/// Only counts *un-escaped* `(` / `)` pairs.  Escaped parens (`\(`) and
/// character-class contents (`[()]`) are skipped.
fn nesting_depth(pattern: &str) -> usize {
    let mut max_depth: usize = 0;
    let mut current: usize = 0;
    let mut chars = pattern.chars().peekable();
    let mut in_char_class = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                let _ = chars.next();
            }
            '[' if !in_char_class => {
                in_char_class = true;
            }
            ']' if in_char_class => {
                in_char_class = false;
            }
            '(' if !in_char_class => {
                current += 1;
                if current > max_depth {
                    max_depth = current;
                }
            }
            ')' if !in_char_class => {
                current = current.saturating_sub(1);
            }
            _ => {}
        }
    }
    max_depth
}

/// Count explicit capture groups (groups that are **not** non-capturing `(?:…)`).
fn capture_group_count(pattern: &str) -> usize {
    let mut count: usize = 0;
    let mut chars = pattern.chars().peekable();
    let mut in_char_class = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                let _ = chars.next();
            }
            '[' if !in_char_class => {
                in_char_class = true;
            }
            ']' if in_char_class => {
                in_char_class = false;
            }
            '(' if !in_char_class => {
                if chars.peek() == Some(&'?') {
                    // Non-capturing or flag group — don't count.
                } else {
                    count += 1;
                }
            }
            _ => {}
        }
    }
    count
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

impl FilterByValueConfig {
    /// Validate all safety invariants.  Called at construction time so that
    /// an invalid config never reaches `execute`.
    pub fn validate(&self) -> Result<()> {
        // 1. Column name must be non-empty
        if self.column.is_empty() {
            return Err(Error::ConfigurationError(
                "filter_by_value: 'column' must not be empty".into(),
            ));
        }

        // 2. Pattern must be non-empty
        if self.pattern.is_empty() {
            return Err(Error::ConfigurationError(
                "filter_by_value: 'pattern' must not be empty".into(),
            ));
        }

        // 3. Pattern length
        if self.pattern.len() > MAX_PATTERN_LEN {
            return Err(Error::ConfigurationError(format!(
                "filter_by_value: pattern length {} exceeds maximum of {MAX_PATTERN_LEN}",
                self.pattern.len()
            )));
        }

        // 4. Nesting depth
        let depth = nesting_depth(&self.pattern);
        if depth > MAX_NESTING_DEPTH {
            return Err(Error::ConfigurationError(format!(
                "filter_by_value: pattern nesting depth {depth} exceeds maximum of {MAX_NESTING_DEPTH}"
            )));
        }

        // 5. Capture group count
        let groups = capture_group_count(&self.pattern);
        if groups > MAX_CAPTURE_GROUPS {
            return Err(Error::ConfigurationError(format!(
                "filter_by_value: pattern has {groups} capture group(s); maximum is {MAX_CAPTURE_GROUPS}"
            )));
        }

        // 6. Compile the regex with a size limit to catch remaining edge cases
        regex::RegexBuilder::new(&self.pattern)
            .size_limit(MAX_COMPILED_SIZE)
            .build()
            .map_err(|e| {
                Error::ConfigurationError(format!(
                    "filter_by_value: invalid pattern '{}': {e}",
                    self.pattern
                ))
            })?;

        Ok(())
    }
}

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
    /// Pre-compiled regex — validated once at construction.
    compiled: Regex,
}

impl FilterByValue {
    /// Construct from a pre-validated config.
    fn new(config: FilterByValueConfig) -> Result<Self> {
        let compiled = regex::RegexBuilder::new(&config.pattern)
            .size_limit(MAX_COMPILED_SIZE)
            .build()
            .map_err(|e| {
                Error::ConfigurationError(format!(
                    "filter_by_value: failed to compile pattern '{}': {e}",
                    config.pattern
                ))
            })?;
        Ok(Self { config, compiled })
    }

    /// Deserialise and construct from manifest JSON.
    pub fn from_action_config(value: &serde_json::Value) -> Result<Self> {
        let column = value
            .get("column")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::ConfigurationError(
                    "filter_by_value: missing required field 'column'".into(),
                )
            })?
            .to_string();

        let pattern = value
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::ConfigurationError(
                    "filter_by_value: missing required field 'pattern'".into(),
                )
            })?
            .to_string();

        let config = FilterByValueConfig { column, pattern };
        config.validate()?;
        Self::new(config)
    }

    /// Test whether a single value matches the filter pattern.
    fn matches(&self, value: &str) -> bool {
        self.compiled.is_match(value)
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
            .data
            .clone()
            .collect()
            .map_err(|e| Error::LogicError(format!("Failed to collect LazyFrame: {e}")))?;

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

        context.data = filtered.lazy();

        Ok(context)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
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
        let json = serde_json::json!({
            "column": "department",
            "pattern": "Engineering"
        });
        let action = FilterByValue::from_action_config(&json).unwrap();
        assert_eq!(action.id(), "filter_by_value");
    }

    // -----------------------------------------------------------------------
    // Functional: basic filtering
    // -----------------------------------------------------------------------

    #[test]
    fn test_basic_filter() {
        let json = serde_json::json!({
            "column": "department",
            "pattern": "^Engineering$"
        });
        let action = FilterByValue::from_action_config(&json).unwrap();
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).unwrap();
        let df = result.data.collect().unwrap();

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
        let json = serde_json::json!({
            "column": "location",
            "pattern": "on"
        });
        let action = FilterByValue::from_action_config(&json).unwrap();
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).unwrap();
        let df = result.data.collect().unwrap();

        // "London" contains "on" (matches twice), no other city contains "on"
        assert_eq!(df.height(), 2);
        let loc = df.column("location").unwrap().str().unwrap();
        assert_eq!(loc.get(0).unwrap(), "London");
        assert_eq!(loc.get(1).unwrap(), "London");
    }

    #[test]
    fn test_no_match_returns_empty() {
        let json = serde_json::json!({
            "column": "department",
            "pattern": "^Finance$"
        });
        let action = FilterByValue::from_action_config(&json).unwrap();
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).unwrap();
        let df = result.data.collect().unwrap();

        assert_eq!(df.height(), 0);
    }

    #[test]
    fn test_all_match() {
        let df = df! {
            "val" => &["abc", "abcdef", "xabc"],
        }
        .unwrap();
        let json = serde_json::json!({
            "column": "val",
            "pattern": "abc"
        });
        let action = FilterByValue::from_action_config(&json).unwrap();
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).unwrap();
        let collected = result.data.collect().unwrap();
        assert_eq!(collected.height(), 3);
    }

    // -----------------------------------------------------------------------
    // Null handling
    // -----------------------------------------------------------------------

    #[test]
    fn test_null_values_dropped() {
        let s = Series::new("val".into(), &[Some("abc"), None, Some("def")]);
        let df = DataFrame::new_infer_height(vec![s.into()]).unwrap();
        let json = serde_json::json!({
            "column": "val",
            "pattern": "."
        });
        let action = FilterByValue::from_action_config(&json).unwrap();
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).unwrap();
        let collected = result.data.collect().unwrap();

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
        let json = serde_json::json!({
            "column": "val",
            "pattern": "^.+$"
        });
        let action = FilterByValue::from_action_config(&json).unwrap();
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).unwrap();
        let collected = result.data.collect().unwrap();

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
        let json = serde_json::json!({
            "pattern": "x"
        });
        assert!(FilterByValue::from_action_config(&json).is_err());
    }

    #[test]
    fn test_missing_pattern_field() {
        let json = serde_json::json!({
            "column": "department"
        });
        assert!(FilterByValue::from_action_config(&json).is_err());
    }

    #[test]
    fn test_empty_pattern_rejected() {
        let json = serde_json::json!({
            "column": "department",
            "pattern": ""
        });
        assert!(FilterByValue::from_action_config(&json).is_err());
    }

    #[test]
    fn test_empty_column_rejected() {
        let json = serde_json::json!({
            "column": "",
            "pattern": "x"
        });
        assert!(FilterByValue::from_action_config(&json).is_err());
    }

    // -----------------------------------------------------------------------
    // Security: attack-vector rejection
    // -----------------------------------------------------------------------

    #[test]
    fn test_pattern_length_limit() {
        let long_pattern = "a".repeat(MAX_PATTERN_LEN + 1);
        let json = serde_json::json!({
            "column": "department",
            "pattern": long_pattern
        });
        let result = FilterByValue::from_action_config(&json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("pattern length"), "unexpected error: {err}");
    }

    #[test]
    fn test_excessive_nesting_rejected() {
        let json = serde_json::json!({
            "column": "department",
            "pattern": "((((a))))"
        });
        let result = FilterByValue::from_action_config(&json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nesting depth"), "unexpected error: {err}");
    }

    #[test]
    fn test_multiple_capture_groups_rejected() {
        let json = serde_json::json!({
            "column": "department",
            "pattern": "(a)(b)(c)"
        });
        let result = FilterByValue::from_action_config(&json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("capture group"), "unexpected error: {err}");
    }

    #[test]
    fn test_non_capturing_groups_allowed() {
        let json = serde_json::json!({
            "column": "department",
            "pattern": "(?:Eng|Sale)s?"
        });
        let action = FilterByValue::from_action_config(&json);
        assert!(action.is_ok(), "non-capturing groups should be accepted");
    }

    #[test]
    fn test_invalid_regex_rejected() {
        let json = serde_json::json!({
            "column": "department",
            "pattern": "[invalid"
        });
        let result = FilterByValue::from_action_config(&json);
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
        let json = serde_json::json!({
            "column": "department",
            "pattern": "x"
        });
        let action = FilterByValue::from_action_config(&json).unwrap();
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx);
        assert!(result.is_err());
    }
}
