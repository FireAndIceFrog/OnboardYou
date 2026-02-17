//! Configuration models for PII masking.
//!
//! Supports two JSON shapes:
//!
//! **New format** (recommended):
//! ```json
//! {
//!   "columns": [
//!     { "name": "ssn",    "strategy": "redact", "keep_last": 4, "mask_prefix": "***-**-" },
//!     { "name": "salary", "strategy": "zero" }
//!   ]
//! }
//! ```
//!
//! **Legacy format** (still accepted):
//! ```json
//! { "mask_ssn": true, "mask_salary": true }
//! ```
//!
//! The custom [`Deserialize`] impl handles both shapes, so every call-site
//! can simply use `serde_json::from_value::<PIIMaskingConfig>(…)`.

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// The masking strategy for a single column.
///
/// | Strategy | JSON key   | Effect                                                    |
/// |----------|------------|-----------------------------------------------------------|
/// | `Redact` | `"redact"` | Keeps the last N chars, replaces prefix with a mask string |
/// | `Zero`   | `"zero"`   | Replaces all numeric values with `0`                       |
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MaskStrategy {
    /// Keep the last N characters, replace prefix with `mask_prefix`.
    Redact {
        keep_last: usize,
        mask_prefix: String,
    },
    /// Replace all values with zero (for numeric columns).
    Zero,
}

/// Configuration for a single column to mask.
///
/// # JSON shape
///
/// ```json
/// { "name": "ssn", "strategy": "redact", "keep_last": 4, "mask_prefix": "***-**-" }
/// ```
///
/// | Field         | Type   | Default     | Description                             |
/// |---------------|--------|-------------|-----------------------------------------|
/// | `name`        | string | —           | Column name to mask                     |
/// | `strategy`    | string | `"redact"`  | `"redact"` or `"zero"`                 |
/// | `keep_last`   | int    | `4`         | Characters to preserve (redact only)    |
/// | `mask_prefix` | string | `"***-**-"` | Prefix replacing the redacted portion   |
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ColumnMask {
    pub name: String,
    pub strategy: MaskStrategy,
}

/// Configuration for PII masking.
///
/// Accepts both the new `columns`-array format and the legacy boolean
/// format.  See module docs for examples.
#[derive(Serialize, Debug, Clone)]
pub struct PIIMaskingConfig {
    pub columns: Vec<ColumnMask>,
}

impl Default for PIIMaskingConfig {
    fn default() -> Self {
        Self {
            columns: vec![
                ColumnMask {
                    name: "ssn".into(),
                    strategy: MaskStrategy::Redact {
                        keep_last: 4,
                        mask_prefix: "***-**-".into(),
                    },
                },
                ColumnMask {
                    name: "salary".into(),
                    strategy: MaskStrategy::Zero,
                },
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Serde plumbing — private helper structs
// ---------------------------------------------------------------------------

/// Serde-friendly mirror of [`ColumnMask`], used only during
/// deserialisation. The flattened JSON layout (`strategy`, `keep_last`,
/// `mask_prefix` as siblings of `name`) doesn't map cleanly to the public
/// enum, so we deserialise into this raw form first.
#[derive(Deserialize)]
struct RawColumnMask {
    name: String,
    #[serde(default = "default_strategy_str")]
    strategy: String,
    keep_last: Option<usize>,
    mask_prefix: Option<String>,
}

fn default_strategy_str() -> String {
    "redact".into()
}

impl From<RawColumnMask> for ColumnMask {
    fn from(raw: RawColumnMask) -> Self {
        let strategy = match raw.strategy.as_str() {
            "zero" => MaskStrategy::Zero,
            _ => MaskStrategy::Redact {
                keep_last: raw.keep_last.unwrap_or(4),
                mask_prefix: raw.mask_prefix.unwrap_or_else(|| "***-**-".into()),
            },
        };
        Self {
            name: raw.name,
            strategy,
        }
    }
}

/// New format: `{ "columns": [ … ] }`.
#[derive(Deserialize)]
struct NewFormat {
    columns: Vec<RawColumnMask>,
}

/// Legacy format: `{ "mask_ssn": bool, "mask_salary": bool }`.
#[derive(Deserialize)]
struct LegacyFormat {
    #[serde(default = "default_true")]
    mask_ssn: bool,
    #[serde(default = "default_true")]
    mask_salary: bool,
}

fn default_true() -> bool {
    true
}

impl From<LegacyFormat> for PIIMaskingConfig {
    fn from(legacy: LegacyFormat) -> Self {
        let mut columns = Vec::new();
        if legacy.mask_ssn {
            columns.push(ColumnMask {
                name: "ssn".into(),
                strategy: MaskStrategy::Redact {
                    keep_last: 4,
                    mask_prefix: "***-**-".into(),
                },
            });
        }
        if legacy.mask_salary {
            columns.push(ColumnMask {
                name: "salary".into(),
                strategy: MaskStrategy::Zero,
            });
        }
        Self { columns }
    }
}

// ---------------------------------------------------------------------------
// Custom Deserialize — tries new format first, falls back to legacy
// ---------------------------------------------------------------------------

impl<'de> Deserialize<'de> for PIIMaskingConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialise once into a generic Value, then try each format.
        let value = serde_json::Value::deserialize(deserializer)?;

        // New format: { "columns": [ … ] }
        if value.get("columns").is_some() {
            let new: NewFormat = serde_json::from_value(value).map_err(de::Error::custom)?;
            return Ok(Self {
                columns: new.columns.into_iter().map(ColumnMask::from).collect(),
            });
        }

        // Legacy format: { "mask_ssn": bool, "mask_salary": bool }
        let legacy: LegacyFormat = serde_json::from_value(value).map_err(de::Error::custom)?;
        Ok(Self::from(legacy))
    }
}
