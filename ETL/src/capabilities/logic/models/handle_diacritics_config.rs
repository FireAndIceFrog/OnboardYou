//! Configuration model for the handle-diacritics engine.

use serde::Deserialize;

/// Configuration for the handle-diacritics action.
///
/// | Field           | Type      | Default   | Description                                          |
/// |-----------------|-----------|-----------|------------------------------------------------------|
/// | `columns`       | [string]  | `[]`      | Columns to transliterate                             |
/// | `output_suffix` | string?   | `null`    | Suffix for output columns; null = in-place replace   |
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HandleDiacriticsConfig {
    pub columns: Vec<String>,
    pub output_suffix: Option<String>,
}

impl Default for HandleDiacriticsConfig {
    fn default() -> Self {
        Self {
            columns: vec![],
            output_suffix: None,
        }
    }
}
