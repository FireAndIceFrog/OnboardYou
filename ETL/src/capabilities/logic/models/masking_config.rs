//! Configuration models for PII masking.

/// The masking strategy for a single column.
///
/// | Strategy | JSON key   | Effect                                                    |
/// |----------|------------|-----------------------------------------------------------|
/// | `Redact` | `"redact"` | Keeps the last N chars, replaces prefix with a mask string |
/// | `Zero`   | `"zero"`   | Replaces all numeric values with `0`                       |
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct ColumnMask {
    pub name: String,
    pub strategy: MaskStrategy,
}

/// Configuration for PII masking, extracted from manifest JSON.
///
/// # New format (recommended)
///
/// ```json
/// {
///   "columns": [
///     { "name": "ssn",    "strategy": "redact", "keep_last": 4, "mask_prefix": "***-**-" },
///     { "name": "salary", "strategy": "zero" }
///   ]
/// }
/// ```
///
/// # Legacy format (still supported)
///
/// ```json
/// { "mask_ssn": true, "mask_salary": true }
/// ```
#[derive(Debug, Clone)]
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

impl PIIMaskingConfig {
    /// Build from manifest `ActionConfig.config` JSON.
    ///
    /// Supports both the new `columns` array format and the legacy
    /// `mask_ssn` / `mask_salary` boolean format.
    pub fn from_json(value: &serde_json::Value) -> Self {
        // New format: { "columns": [...] }
        if let Some(arr) = value.get("columns").and_then(|v| v.as_array()) {
            let columns = arr
                .iter()
                .filter_map(|entry| {
                    let name = entry.get("name")?.as_str()?.to_string();
                    let strategy_str = entry
                        .get("strategy")
                        .and_then(|v| v.as_str())
                        .unwrap_or("redact");
                    let strategy = match strategy_str {
                        "zero" => MaskStrategy::Zero,
                        _ => MaskStrategy::Redact {
                            keep_last: entry
                                .get("keep_last")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(4)
                                as usize,
                            mask_prefix: entry
                                .get("mask_prefix")
                                .and_then(|v| v.as_str())
                                .unwrap_or("***-**-")
                                .to_string(),
                        },
                    };
                    Some(ColumnMask { name, strategy })
                })
                .collect();
            return Self { columns };
        }

        // Legacy format: { "mask_ssn": bool, "mask_salary": bool }
        let mut columns = Vec::new();
        let mask_ssn = value
            .get("mask_ssn")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let mask_salary = value
            .get("mask_salary")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if mask_ssn {
            columns.push(ColumnMask {
                name: "ssn".into(),
                strategy: MaskStrategy::Redact {
                    keep_last: 4,
                    mask_prefix: "***-**-".into(),
                },
            });
        }
        if mask_salary {
            columns.push(ColumnMask {
                name: "salary".into(),
                strategy: MaskStrategy::Zero,
            });
        }
        Self { columns }
    }
}
