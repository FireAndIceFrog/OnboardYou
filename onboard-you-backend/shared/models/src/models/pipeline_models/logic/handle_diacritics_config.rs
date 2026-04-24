//! Configuration model for the handle-diacritics engine.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Configuration for the handle-diacritics action.
///
/// | Field           | Type      | Default   | Description                                          |
/// |-----------------|-----------|-----------|------------------------------------------------------|
/// | `columns`       | [string]  | `[]`      | Columns to transliterate                             |
/// | `output_suffix` | string?   | `null`    | Suffix for output columns; null = in-place replace   |
#[derive(Serialize, Deserialize, Debug, Clone, Default, ToSchema)]
#[serde(default)]
pub struct HandleDiacriticsConfig {
    pub columns: Vec<String>,
    pub output_suffix: Option<String>,
}
