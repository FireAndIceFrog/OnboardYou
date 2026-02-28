//! Configuration model for the regex-replace engine.

use super::SafeRegex;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Maximum length of the replacement string.
pub const MAX_REPLACEMENT_LEN: usize = 256;

/// Configuration for the regex-replace action.
///
/// | Field         | Type   | Description                                    |
/// |---------------|--------|------------------------------------------------|
/// | `column`      | string | Target column to apply the replacement to      |
/// | `pattern`     | string | Regex pattern (Rust `regex` syntax)            |
/// | `replacement` | string | Literal replacement for the matched substring  |
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RegexReplaceConfig {
    /// Column to operate on.
    pub column: String,
    /// The raw regex pattern.
    pub pattern: String,
    /// Literal replacement text (backreference syntax is **not** honoured).
    pub replacement: String,
}

impl RegexReplaceConfig {
    /// Validate all safety invariants and return the compiled [`SafeRegex`].
    ///
    /// Called at construction time so that an invalid config never reaches
    /// `execute`.  The returned regex is ready to use — callers should keep
    /// it rather than re-compiling.
    pub fn validate(&self) -> Result<SafeRegex> {
        if self.column.is_empty() {
            return Err(Error::ConfigurationError(
                "regex_replace: 'column' must not be empty".into(),
            ));
        }

        // Replacement length
        if self.replacement.len() > MAX_REPLACEMENT_LEN {
            return Err(Error::ConfigurationError(format!(
                "regex_replace: replacement length {} exceeds maximum of {MAX_REPLACEMENT_LEN}",
                self.replacement.len()
            )));
        }

        // SafeRegex::new handles pattern-level validation (empty, length,
        // nesting, groups, compilation).
        SafeRegex::new(&self.pattern, "regex_replace")
    }
}
