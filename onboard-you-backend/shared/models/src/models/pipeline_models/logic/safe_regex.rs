//! SafeRegex: A validated, pre-compiled regex with defence-in-depth controls
//!
//! Consolidates the regex validation logic previously duplicated across
//! `regex_replace` and `filter_by_value` into a single reusable type.
//!
//! # Security model
//!
//! | Control                        | Rationale                                                     |
//! |--------------------------------|---------------------------------------------------------------|
//! | Rust `regex` crate only        | Guarantees **linear-time** matching (Thompson NFA) — immune   |
//! |                                | to catastrophic-backtracking ReDoS by construction.           |
//! | Pattern length ≤ 128 chars     | Caps compilation cost and prevents pattern-bomb payloads.     |
//! | Compiled size ≤ 64 KiB         | `RegexBuilder::size_limit` — bounds memory for the NFA/DFA.  |
//! | Exactly 0 or 1 capture groups  | Requirement: single match-group only.                         |
//! | Nesting depth ≤ 3              | Rejects deeply nested groups like `(((...)))`.                |

use crate::{Error, Result};
use regex::Regex;
use std::fmt;

// ---------------------------------------------------------------------------
// Hard limits (compile-time constants — not user-configurable)
// ---------------------------------------------------------------------------

/// Maximum length of the raw pattern string.
pub const MAX_PATTERN_LEN: usize = 128;

/// Maximum compiled NFA/DFA size in bytes (64 KiB).
pub const MAX_COMPILED_SIZE: usize = 64 * 1024;

/// Maximum nesting depth of parenthesised groups.
pub const MAX_NESTING_DEPTH: usize = 3;

/// Maximum number of capture groups (excluding the implicit group 0).
pub const MAX_CAPTURE_GROUPS: usize = 1;

// ---------------------------------------------------------------------------
// Analysis helpers
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

/// Escape backreference syntax (`$0`, `$1`, `${name}`, etc.) in a
/// replacement string so it is treated as a pure literal by
/// `regex::Regex::replace`.
fn escape_replacement(replacement: &str) -> String {
    replacement.replace('$', "$$")
}

// ---------------------------------------------------------------------------
// SafeRegex
// ---------------------------------------------------------------------------

/// A regex that has been sanity-checked against the security constraints
/// before compilation.
///
/// Construction via [`SafeRegex::new`] validates:
///
/// 1. Pattern length ≤ [`MAX_PATTERN_LEN`]
/// 2. Nesting depth ≤ [`MAX_NESTING_DEPTH`]
/// 3. Capture group count ≤ [`MAX_CAPTURE_GROUPS`]
/// 4. Successful compilation with a size limit of [`MAX_COMPILED_SIZE`]
///
/// Once built, the inner [`Regex`] is guaranteed safe against the attack
/// vectors enumerated in the module-level security table.
pub struct SafeRegex {
    /// The original pattern string.
    pattern: String,
    /// Pre-compiled regex — validated at construction.
    compiled: Regex,
}

impl SafeRegex {
    /// Validate and compile a pattern, returning a `SafeRegex` or a
    /// configuration error.
    ///
    /// `context` is a short label (e.g. `"filter_by_value"`) included in
    /// error messages so the caller doesn't have to re-wrap them.
    pub fn new(pattern: &str, context: &str) -> Result<Self> {
        // 1. Non-empty
        if pattern.is_empty() {
            return Err(Error::ConfigurationError(format!(
                "{context}: 'pattern' must not be empty"
            )));
        }

        // 2. Length
        if pattern.len() > MAX_PATTERN_LEN {
            return Err(Error::ConfigurationError(format!(
                "{context}: pattern length {} exceeds maximum of {MAX_PATTERN_LEN}",
                pattern.len()
            )));
        }

        // 3. Nesting depth
        let depth = nesting_depth(pattern);
        if depth > MAX_NESTING_DEPTH {
            return Err(Error::ConfigurationError(format!(
                "{context}: pattern nesting depth {depth} exceeds maximum of {MAX_NESTING_DEPTH}"
            )));
        }

        // 4. Capture group count
        let groups = capture_group_count(pattern);
        if groups > MAX_CAPTURE_GROUPS {
            return Err(Error::ConfigurationError(format!(
                "{context}: pattern has {groups} capture group(s); maximum is {MAX_CAPTURE_GROUPS}"
            )));
        }

        // 5. Compile with size limit
        let compiled = regex::RegexBuilder::new(pattern)
            .size_limit(MAX_COMPILED_SIZE)
            .build()
            .map_err(|e| {
                Error::ConfigurationError(format!("{context}: invalid pattern '{pattern}': {e}"))
            })?;

        Ok(Self {
            pattern: pattern.to_string(),
            compiled,
        })
    }

    /// The original pattern string.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Reference to the compiled `Regex`.
    pub fn compiled(&self) -> &Regex {
        &self.compiled
    }

    /// Test whether the pattern matches anywhere in `text`.
    pub fn is_match(&self, text: &str) -> bool {
        self.compiled.is_match(text)
    }

    /// Replace the first match in `text` with `replacement`.
    ///
    /// The replacement is treated as a **literal** — backreference syntax
    /// (`$1`, `${name}`) is escaped automatically.
    pub fn replace_first(&self, text: &str, replacement: &str) -> String {
        let safe = escape_replacement(replacement);
        self.compiled.replace(text, safe.as_str()).into_owned()
    }
}

impl fmt::Debug for SafeRegex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SafeRegex")
            .field("pattern", &self.pattern)
            .finish()
    }
}

impl fmt::Display for SafeRegex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pattern)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- nesting_depth ----------------------------------------------------

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
        assert_eq!(nesting_depth(r"\(abc\)"), 0);
        assert_eq!(nesting_depth(r"(\(a\))"), 1);
    }

    #[test]
    fn test_nesting_depth_char_class() {
        assert_eq!(nesting_depth("[(]"), 0);
        assert_eq!(nesting_depth("[()]"), 0);
    }

    // ---- capture_group_count ----------------------------------------------

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

    // ---- escape_replacement -----------------------------------------------

    #[test]
    fn test_escape_replacement() {
        assert_eq!(escape_replacement("hello"), "hello");
        assert_eq!(escape_replacement("$1"), "$$1");
        assert_eq!(escape_replacement("${name}"), "$${name}");
        assert_eq!(escape_replacement("a$b$c"), "a$$b$$c");
    }

    // ---- SafeRegex --------------------------------------------------------

    #[test]
    fn test_safe_regex_valid() {
        let re = SafeRegex::new("^hello$", "test").unwrap();
        assert!(re.is_match("hello"));
        assert!(!re.is_match("world"));
        assert_eq!(re.pattern(), "^hello$");
    }

    #[test]
    fn test_safe_regex_empty_rejected() {
        assert!(SafeRegex::new("", "test").is_err());
    }

    #[test]
    fn test_safe_regex_too_long() {
        let long = "a".repeat(MAX_PATTERN_LEN + 1);
        assert!(SafeRegex::new(&long, "test").is_err());
    }

    #[test]
    fn test_safe_regex_deep_nesting_rejected() {
        assert!(SafeRegex::new("((((a))))", "test").is_err());
    }

    #[test]
    fn test_safe_regex_too_many_groups_rejected() {
        assert!(SafeRegex::new("(a)(b)", "test").is_err());
    }

    #[test]
    fn test_safe_regex_non_capturing_allowed() {
        assert!(SafeRegex::new("(?:a)(?:b)(?:c)", "test").is_ok());
    }

    #[test]
    fn test_safe_regex_invalid_pattern() {
        assert!(SafeRegex::new("[invalid", "test").is_err());
    }

    #[test]
    fn test_safe_regex_replace_first() {
        let re = SafeRegex::new("world", "test").unwrap();
        assert_eq!(
            re.replace_first("hello world world", "rust"),
            "hello rust world"
        );
    }

    #[test]
    fn test_safe_regex_replace_escapes_backrefs() {
        let re = SafeRegex::new("(hello)", "test").unwrap();
        let result = re.replace_first("hello world", "$1_expanded");
        assert_eq!(result, "$1_expanded world");
    }
}
