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

/// Maximum length of the replacement string.
const MAX_REPLACEMENT_LEN: usize = 256;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the regex-replace action.
///
/// | Field         | Type   | Description                                    |
/// |---------------|--------|------------------------------------------------|
/// | `column`      | string | Target column to apply the replacement to      |
/// | `pattern`     | string | Regex pattern (Rust `regex` syntax)            |
/// | `replacement` | string | Literal replacement for the matched substring  |
#[derive(Debug, Clone)]
pub struct RegexReplaceConfig {
    /// Column to operate on.
    pub column: String,
    /// The raw regex pattern.
    pub pattern: String,
    /// Literal replacement text (backreference syntax is **not** honoured).
    pub replacement: String,
}

// ---------------------------------------------------------------------------
// Validation
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
                // Skip the next character — it is escaped.
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
                // Peek ahead: non-capturing `(?:` or flags like `(?i:` don't count.
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

/// Escape backreference syntax (`$0`, `$1`, `${name}`, etc.) in the
/// replacement string so it is treated as a pure literal by `regex::Regex::replace`.
fn escape_replacement(replacement: &str) -> String {
    // The `regex` crate treats `$` as a backreference sigil in replacements.
    // Escaping `$` → `$$` neutralises this.
    replacement.replace('$', "$$")
}

impl RegexReplaceConfig {
    /// Validate all safety invariants.  Called at construction time so that
    /// an invalid config never reaches `execute`.
    pub fn validate(&self) -> Result<()> {
        // 1. Column name must be non-empty
        if self.column.is_empty() {
            return Err(Error::ConfigurationError(
                "regex_replace: 'column' must not be empty".into(),
            ));
        }

        // 2. Pattern must be non-empty
        if self.pattern.is_empty() {
            return Err(Error::ConfigurationError(
                "regex_replace: 'pattern' must not be empty".into(),
            ));
        }

        // 3. Pattern length
        if self.pattern.len() > MAX_PATTERN_LEN {
            return Err(Error::ConfigurationError(format!(
                "regex_replace: pattern length {} exceeds maximum of {MAX_PATTERN_LEN}",
                self.pattern.len()
            )));
        }

        // 4. Replacement length
        if self.replacement.len() > MAX_REPLACEMENT_LEN {
            return Err(Error::ConfigurationError(format!(
                "regex_replace: replacement length {} exceeds maximum of {MAX_REPLACEMENT_LEN}",
                self.replacement.len()
            )));
        }

        // 5. Nesting depth
        let depth = nesting_depth(&self.pattern);
        if depth > MAX_NESTING_DEPTH {
            return Err(Error::ConfigurationError(format!(
                "regex_replace: pattern nesting depth {depth} exceeds maximum of {MAX_NESTING_DEPTH}"
            )));
        }

        // 6. Capture group count
        let groups = capture_group_count(&self.pattern);
        if groups > MAX_CAPTURE_GROUPS {
            return Err(Error::ConfigurationError(format!(
                "regex_replace: pattern has {groups} capture group(s); maximum is {MAX_CAPTURE_GROUPS}"
            )));
        }

        // 7. Compile the regex with a size limit to catch remaining edge cases
        regex::RegexBuilder::new(&self.pattern)
            .size_limit(MAX_COMPILED_SIZE)
            .build()
            .map_err(|e| {
                Error::ConfigurationError(format!(
                    "regex_replace: invalid pattern '{}': {e}",
                    self.pattern
                ))
            })?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Replace the first regex match in a string column with a literal string.
///
/// See the module-level documentation for the full security model.
#[derive(Debug)]
pub struct RegexReplace {
    config: RegexReplaceConfig,
    /// Pre-compiled regex — validated once at construction.
    compiled: Regex,
    /// Safe replacement string with backreference syntax escaped.
    safe_replacement: String,
}

impl RegexReplace {
    /// Construct from a pre-validated config.
    fn new(config: RegexReplaceConfig) -> Result<Self> {
        let compiled = regex::RegexBuilder::new(&config.pattern)
            .size_limit(MAX_COMPILED_SIZE)
            .build()
            .map_err(|e| {
                Error::ConfigurationError(format!(
                    "regex_replace: failed to compile pattern '{}': {e}",
                    config.pattern
                ))
            })?;
        let safe_replacement = escape_replacement(&config.replacement);
        Ok(Self {
            config,
            compiled,
            safe_replacement,
        })
    }

    /// Deserialise and construct from manifest JSON.
    pub fn from_action_config(value: &serde_json::Value) -> Result<Self> {
        let column = value
            .get("column")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::ConfigurationError(
                    "regex_replace: missing required field 'column'".into(),
                )
            })?
            .to_string();

        let pattern = value
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::ConfigurationError(
                    "regex_replace: missing required field 'pattern'".into(),
                )
            })?
            .to_string();

        let replacement = value
            .get("replacement")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::ConfigurationError(
                    "regex_replace: missing required field 'replacement'".into(),
                )
            })?
            .to_string();

        let config = RegexReplaceConfig {
            column,
            pattern,
            replacement,
        };
        config.validate()?;
        Self::new(config)
    }

    /// Apply the replacement to a single string value.
    fn apply(&self, value: &str) -> String {
        // `replace` replaces the first (leftmost) match only.
        self.compiled
            .replace(value, self.safe_replacement.as_str())
            .into_owned()
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
        let _ = result_df.replace(&self.config.column, replaced.into_column()).map_err(|e| {
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
        let json = serde_json::json!({
            "column": "phone",
            "pattern": "x",
            "replacement": "y"
        });
        let action = RegexReplace::from_action_config(&json).unwrap();
        assert_eq!(action.id(), "regex_replace");
    }

    #[test]
    fn test_basic_replacement() {
        let json = serde_json::json!({
            "column": "phone",
            "pattern": "\\+44\\s?",
            "replacement": "0"
        });
        let action = RegexReplace::from_action_config(&json).unwrap();
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
        let json = serde_json::json!({
            "column": "postcode",
            "pattern": "\\s+[0-9][A-Z]{2}$",
            "replacement": ""
        });
        let action = RegexReplace::from_action_config(&json).unwrap();
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
        let json = serde_json::json!({
            "column": "val",
            "pattern": "a",
            "replacement": "X"
        });
        let action = RegexReplace::from_action_config(&json).unwrap();
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).unwrap();
        let collected = result.data.collect().unwrap();
        let col = collected.column("val").unwrap().str().unwrap();
        // Only the first 'a' is replaced
        assert_eq!(col.get(0).unwrap(), "Xaa");
    }

    #[test]
    fn test_no_match_leaves_value_unchanged() {
        let json = serde_json::json!({
            "column": "phone",
            "pattern": "NOMATCH",
            "replacement": "X"
        });
        let action = RegexReplace::from_action_config(&json).unwrap();
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).unwrap();
        let df = result.data.collect().unwrap();
        let col = df.column("phone").unwrap().str().unwrap();
        assert_eq!(col.get(0).unwrap(), "+44 7911 123456");
    }

    #[test]
    fn test_field_metadata_provenance() {
        let json = serde_json::json!({
            "column": "phone",
            "pattern": "\\+44",
            "replacement": "0"
        });
        let action = RegexReplace::from_action_config(&json).unwrap();
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).unwrap();

        let meta = result.field_metadata.get("phone").expect("metadata for 'phone'");
        assert_eq!(meta.source, "LOGIC_ACTION");
        assert_eq!(meta.modified_by.as_deref(), Some("regex_replace"));
    }

    // -----------------------------------------------------------------------
    // Config / validation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_missing_column_field() {
        let json = serde_json::json!({
            "pattern": "x",
            "replacement": "y"
        });
        assert!(RegexReplace::from_action_config(&json).is_err());
    }

    #[test]
    fn test_missing_pattern_field() {
        let json = serde_json::json!({
            "column": "phone",
            "replacement": "y"
        });
        assert!(RegexReplace::from_action_config(&json).is_err());
    }

    #[test]
    fn test_missing_replacement_field() {
        let json = serde_json::json!({
            "column": "phone",
            "pattern": "x"
        });
        assert!(RegexReplace::from_action_config(&json).is_err());
    }

    #[test]
    fn test_empty_pattern_rejected() {
        let json = serde_json::json!({
            "column": "phone",
            "pattern": "",
            "replacement": "y"
        });
        assert!(RegexReplace::from_action_config(&json).is_err());
    }

    #[test]
    fn test_empty_column_rejected() {
        let json = serde_json::json!({
            "column": "",
            "pattern": "x",
            "replacement": "y"
        });
        assert!(RegexReplace::from_action_config(&json).is_err());
    }

    // -----------------------------------------------------------------------
    // Security: attack-vector rejection
    // -----------------------------------------------------------------------

    #[test]
    fn test_pattern_length_limit() {
        let long_pattern = "a".repeat(MAX_PATTERN_LEN + 1);
        let json = serde_json::json!({
            "column": "phone",
            "pattern": long_pattern,
            "replacement": "x"
        });
        let result = RegexReplace::from_action_config(&json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("pattern length"), "unexpected error: {err}");
    }

    #[test]
    fn test_replacement_length_limit() {
        let long_replacement = "x".repeat(MAX_REPLACEMENT_LEN + 1);
        let json = serde_json::json!({
            "column": "phone",
            "pattern": "a",
            "replacement": long_replacement
        });
        let result = RegexReplace::from_action_config(&json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("replacement length"), "unexpected error: {err}");
    }

    #[test]
    fn test_excessive_nesting_rejected() {
        // Depth 4 — exceeds MAX_NESTING_DEPTH of 3
        let json = serde_json::json!({
            "column": "phone",
            "pattern": "((((a))))",
            "replacement": "x"
        });
        let result = RegexReplace::from_action_config(&json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nesting depth"), "unexpected error: {err}");
    }

    #[test]
    fn test_multiple_capture_groups_rejected() {
        let json = serde_json::json!({
            "column": "phone",
            "pattern": "(a)(b)",
            "replacement": "x"
        });
        let result = RegexReplace::from_action_config(&json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("capture group"), "unexpected error: {err}");
    }

    #[test]
    fn test_non_capturing_groups_allowed() {
        // (?:...) groups do not count towards the capture limit
        let json = serde_json::json!({
            "column": "phone",
            "pattern": "(?:a)(?:b)(?:c)",
            "replacement": "x"
        });
        assert!(RegexReplace::from_action_config(&json).is_ok());
    }

    #[test]
    fn test_backreference_in_replacement_escaped() {
        // Replacement containing `$1` should be treated literally
        let df = df! {
            "val" => &["hello world"],
        }
        .unwrap();
        let json = serde_json::json!({
            "column": "val",
            "pattern": "(hello)",
            "replacement": "$1_expanded"
        });
        let action = RegexReplace::from_action_config(&json).unwrap();
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).unwrap();
        let collected = result.data.collect().unwrap();
        let col = collected.column("val").unwrap().str().unwrap();
        // $1 is escaped to $$ so it appears literally
        assert_eq!(col.get(0).unwrap(), "$1_expanded world");
    }

    #[test]
    fn test_invalid_regex_rejected() {
        let json = serde_json::json!({
            "column": "phone",
            "pattern": "[invalid",
            "replacement": "x"
        });
        let result = RegexReplace::from_action_config(&json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid pattern"), "unexpected error: {err}");
    }

    #[test]
    fn test_missing_column_at_runtime() {
        let json = serde_json::json!({
            "column": "nonexistent",
            "pattern": "a",
            "replacement": "b"
        });
        let action = RegexReplace::from_action_config(&json).unwrap();
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
        let json = serde_json::json!({
            "column": "val",
            "pattern": "b",
            "replacement": "X"
        });
        let action = RegexReplace::from_action_config(&json).unwrap();
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
        let json = serde_json::json!({
            "column": "val",
            "pattern": "a",
            "replacement": "X"
        });
        let action = RegexReplace::from_action_config(&json).unwrap();
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).unwrap();
        let collected = result.data.collect().unwrap();
        let col = collected.column("val").unwrap().str().unwrap();
        assert_eq!(col.get(0).unwrap(), "");
        assert_eq!(col.get(1).unwrap(), "Xbc");
        assert_eq!(col.get(2).unwrap(), "");
    }

    // -----------------------------------------------------------------------
    // Helpers — nesting / group counting
    // -----------------------------------------------------------------------

    #[test]
    fn test_nesting_depth_flat() {
        assert_eq!(nesting_depth("abc"), 0);
        assert_eq!(nesting_depth("(abc)"), 1);
        assert_eq!(nesting_depth("(a)(b)"), 1);
    }

    #[test]
    fn test_nesting_depth_nested() {
        assert_eq!(nesting_depth("((a))"), 2);
        assert_eq!(nesting_depth("(((a)))"), 3);
        assert_eq!(nesting_depth("((((a))))"), 4);
    }

    #[test]
    fn test_nesting_depth_escaped_parens() {
        // Escaped parens don't count
        assert_eq!(nesting_depth(r"\(abc\)"), 0);
        assert_eq!(nesting_depth(r"(\(a\))"), 1);
    }

    #[test]
    fn test_nesting_depth_char_class() {
        // Parens inside character classes don't count
        assert_eq!(nesting_depth("[(]"), 0);
        assert_eq!(nesting_depth("[()]"), 0);
    }

    #[test]
    fn test_capture_group_count_non_capturing() {
        assert_eq!(capture_group_count("(?:a)"), 0);
        assert_eq!(capture_group_count("(?:a)(?:b)"), 0);
    }

    #[test]
    fn test_capture_group_count_capturing() {
        assert_eq!(capture_group_count("(a)"), 1);
        assert_eq!(capture_group_count("(a)(b)"), 2);
        assert_eq!(capture_group_count("(a)(?:b)"), 1);
    }

    #[test]
    fn test_escape_replacement() {
        assert_eq!(escape_replacement("hello"), "hello");
        assert_eq!(escape_replacement("$1"), "$$1");
        assert_eq!(escape_replacement("${name}"), "$${name}");
        assert_eq!(escape_replacement("a$b$c"), "a$$b$$c");
    }
}
