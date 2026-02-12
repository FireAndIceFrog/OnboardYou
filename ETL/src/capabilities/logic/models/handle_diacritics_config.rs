//! Configuration model for the handle-diacritics engine.

/// Configuration for the handle-diacritics action.
///
/// | Field           | Type      | Default   | Description                                          |
/// |-----------------|-----------|-----------|------------------------------------------------------|
/// | `columns`       | [string]  | `[]`      | Columns to transliterate                             |
/// | `output_suffix` | string?   | `null`    | Suffix for output columns; null = in-place replace   |
#[derive(Debug, Clone)]
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

impl HandleDiacriticsConfig {
    pub fn from_json(value: &serde_json::Value) -> Self {
        let columns = value
            .get("columns")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let output_suffix = value
            .get("output_suffix")
            .and_then(|v| v.as_str())
            .map(String::from);

        Self {
            columns,
            output_suffix,
        }
    }
}
