//! PII protection: configurable column masking
//!
//! ## Masking rules
//!
//! Each entry in `columns` specifies a column name and a masking strategy:
//!
//! | Strategy   | Effect                                                       |
//! |------------|--------------------------------------------------------------|
//! | `redact`   | Keep last N characters (default 4), replace prefix with mask |
//! | `zero`     | Replace numeric values with 0                                |
//!
//! Configurable via manifest JSON:
//! ```json
//! {
//!   "columns": [
//!     { "name": "ssn",    "strategy": "redact", "keep_last": 4, "mask_prefix": "***-**-" },
//!     { "name": "salary", "strategy": "zero" }
//!   ]
//! }
//! ```
//!
//! For backward compatibility, `{ "mask_ssn": true, "mask_salary": true }` is
//! still accepted and converted to the column-based format.

use crate::capabilities::logic::traits::Masker;
use crate::domain::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// The masking strategy for a single column.
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
#[derive(Debug, Clone)]
pub struct ColumnMask {
    pub name: String,
    pub strategy: MaskStrategy,
}

/// Configuration for PII masking, extracted from manifest JSON.
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

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// PII masking based on residency and regulatory requirements.
#[derive(Debug, Clone)]
pub struct PIIMasking {
    config: PIIMaskingConfig,
}

impl PIIMasking {
    pub fn new(config: PIIMaskingConfig) -> Self {
        Self { config }
    }

    /// Convenience constructor from manifest JSON.
    pub fn from_action_config(value: &serde_json::Value) -> Self {
        Self::new(PIIMaskingConfig::from_json(value))
    }
}

impl Default for PIIMasking {
    fn default() -> Self {
        Self::new(PIIMaskingConfig::default())
    }
}

impl Masker for PIIMasking {
    fn mask(&self, context: RosterContext) -> Result<RosterContext> {
        self.execute(context)
    }
}

impl OnboardingAction for PIIMasking {
    fn id(&self) -> &str {
        "pii_masking"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            columns = ?self.config.columns.iter().map(|c| &c.name).collect::<Vec<_>>(),
            "PIIMasking: applying PII masks"
        );

        // Collect eagerly for reliable column operations
        let lf = std::mem::replace(&mut context.data, LazyFrame::default());
        let mut df = lf.collect().map_err(|e| {
            Error::LogicError(format!("Failed to collect for masking: {}", e))
        })?;

        for col_mask in &self.config.columns {
            if !df.schema().contains(col_mask.name.as_str()) {
                tracing::warn!(
                    column = %col_mask.name,
                    "PIIMasking: column not found in data — skipping"
                );
                continue;
            }

            match &col_mask.strategy {
                MaskStrategy::Redact {
                    keep_last,
                    mask_prefix,
                } => {
                    let src = df.column(&col_mask.name).unwrap().str().unwrap();
                    let keep = *keep_last;
                    let prefix = mask_prefix.clone();
                    let masked: StringChunked = src
                        .into_iter()
                        .map(|opt_val| {
                            opt_val.map(|val| {
                                let len = val.len();
                                if len >= keep {
                                    format!("{}{}", prefix, &val[len - keep..])
                                } else {
                                    format!("{}????", prefix)
                                }
                            })
                        })
                        .collect();
                    let masked = masked.with_name(col_mask.name.as_str().into());
                    let _ = df.replace(col_mask.name.as_str(), masked);
                }
                MaskStrategy::Zero => {
                    let col_ref = df.column(&col_mask.name).unwrap();
                    let n = col_ref.len();
                    let dtype = col_ref.dtype().clone();

                    let zeros = match dtype {
                        DataType::Int32 => Series::new(col_mask.name.as_str().into(), vec![0i32; n]),
                        DataType::Int64 => Series::new(col_mask.name.as_str().into(), vec![0i64; n]),
                        DataType::Float32 => {
                            Series::new(col_mask.name.as_str().into(), vec![0.0f32; n])
                        }
                        DataType::Float64 => {
                            Series::new(col_mask.name.as_str().into(), vec![0.0f64; n])
                        }
                        _ => Series::new(col_mask.name.as_str().into(), vec![0i64; n]),
                    };
                    let _ = df.replace(col_mask.name.as_str(), zeros);
                }
            }

            context.set_field_source(col_mask.name.clone(), "LOGIC_ACTION".into());
            context.mark_field_modified(col_mask.name.clone(), "pii_masking".into());
        }

        context.data = df.lazy();
        Ok(context)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_df() -> DataFrame {
        df! {
            "employee_id" => &["001", "002", "003"],
            "first_name"  => &["John", "Jane", "Alice"],
            "ssn"         => &["123-45-6789", "987-65-4321", "555-12-3456"],
            "salary"      => &[75_000i64, 85_000, 92_000],
        }
        .expect("test df")
    }

    #[test]
    fn test_pii_masking_id() {
        let action = PIIMasking::default();
        assert_eq!(action.id(), "pii_masking");
    }

    #[test]
    fn test_ssn_masked() {
        let ctx = RosterContext::new(test_df().lazy());
        let action = PIIMasking::default();
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let ssn_col = df.column("ssn").unwrap();
        let ssns: Vec<Option<&str>> = ssn_col.str().unwrap().into_iter().collect();

        assert_eq!(ssns[0], Some("***-**-6789"));
        assert_eq!(ssns[1], Some("***-**-4321"));
        assert_eq!(ssns[2], Some("***-**-3456"));
    }

    #[test]
    fn test_salary_zeroed() {
        let ctx = RosterContext::new(test_df().lazy());
        let action = PIIMasking::default();
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let salary_col = df.column("salary").unwrap();
        let salaries: Vec<Option<i64>> = salary_col.i64().unwrap().into_iter().collect();

        assert!(salaries.iter().all(|s| *s == Some(0)));
    }

    #[test]
    fn test_mask_ssn_only() {
        let config = PIIMaskingConfig {
            columns: vec![ColumnMask {
                name: "ssn".into(),
                strategy: MaskStrategy::Redact {
                    keep_last: 4,
                    mask_prefix: "***-**-".into(),
                },
            }],
        };
        let ctx = RosterContext::new(test_df().lazy());
        let action = PIIMasking::new(config);
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        // SSN should be masked
        let ssns: Vec<Option<&str>> = df.column("ssn").unwrap().str().unwrap().into_iter().collect();
        assert_eq!(ssns[0], Some("***-**-6789"));

        // Salary should be untouched
        let salaries: Vec<Option<i64>> = df.column("salary").unwrap().i64().unwrap().into_iter().collect();
        assert_eq!(salaries[0], Some(75_000));
    }

    #[test]
    fn test_mask_salary_only() {
        let config = PIIMaskingConfig {
            columns: vec![ColumnMask {
                name: "salary".into(),
                strategy: MaskStrategy::Zero,
            }],
        };
        let ctx = RosterContext::new(test_df().lazy());
        let action = PIIMasking::new(config);
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        // SSN untouched
        let ssns: Vec<Option<&str>> = df.column("ssn").unwrap().str().unwrap().into_iter().collect();
        assert_eq!(ssns[0], Some("123-45-6789"));

        // Salary masked
        let salaries: Vec<Option<i64>> = df.column("salary").unwrap().i64().unwrap().into_iter().collect();
        assert!(salaries.iter().all(|s| *s == Some(0)));
    }

    #[test]
    fn test_mask_no_ssn_column() {
        // DataFrame without ssn — should not error
        let df = df! {
            "employee_id" => &["001"],
            "salary"      => &[50_000i64],
        }
        .unwrap();
        let ctx = RosterContext::new(df.lazy());
        let action = PIIMasking::default();
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");
        assert_eq!(df.height(), 1);
    }

    #[test]
    fn test_field_metadata() {
        let ctx = RosterContext::new(test_df().lazy());
        let action = PIIMasking::default();
        let result = action.execute(ctx).expect("execute");

        let ssn_meta = result.field_metadata.get("ssn").expect("ssn metadata");
        assert_eq!(ssn_meta.source, "LOGIC_ACTION");
        assert_eq!(ssn_meta.modified_by.as_deref(), Some("pii_masking"));

        let sal_meta = result.field_metadata.get("salary").expect("salary metadata");
        assert_eq!(sal_meta.source, "LOGIC_ACTION");
        assert_eq!(sal_meta.modified_by.as_deref(), Some("pii_masking"));
    }

    #[test]
    fn test_from_action_config_legacy() {
        let json = serde_json::json!({ "mask_ssn": false, "mask_salary": true });
        let action = PIIMasking::from_action_config(&json);
        assert_eq!(action.config.columns.len(), 1);
        assert_eq!(action.config.columns[0].name, "salary");
    }

    #[test]
    fn test_from_action_config_new_format() {
        let json = serde_json::json!({
            "columns": [
                { "name": "phone", "strategy": "redact", "keep_last": 4, "mask_prefix": "***-" },
                { "name": "bonus", "strategy": "zero" }
            ]
        });
        let action = PIIMasking::from_action_config(&json);
        assert_eq!(action.config.columns.len(), 2);
        assert_eq!(action.config.columns[0].name, "phone");
        assert_eq!(action.config.columns[1].name, "bonus");
    }

    #[test]
    fn test_custom_redact_column() {
        let df = df! {
            "employee_id" => &["001"],
            "phone"       => &["555-123-4567"],
        }
        .unwrap();
        let config = PIIMaskingConfig {
            columns: vec![ColumnMask {
                name: "phone".into(),
                strategy: MaskStrategy::Redact {
                    keep_last: 4,
                    mask_prefix: "***-***-".into(),
                },
            }],
        };
        let ctx = RosterContext::new(df.lazy());
        let action = PIIMasking::new(config);
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let phones: Vec<Option<&str>> = df.column("phone").unwrap().str().unwrap().into_iter().collect();
        assert_eq!(phones[0], Some("***-***-4567"));
    }

    #[test]
    fn test_custom_zero_column() {
        let df = df! {
            "employee_id" => &["001"],
            "bonus"       => &[10_000i64],
        }
        .unwrap();
        let config = PIIMaskingConfig {
            columns: vec![ColumnMask {
                name: "bonus".into(),
                strategy: MaskStrategy::Zero,
            }],
        };
        let ctx = RosterContext::new(df.lazy());
        let action = PIIMasking::new(config);
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let bonuses: Vec<Option<i64>> = df.column("bonus").unwrap().i64().unwrap().into_iter().collect();
        assert_eq!(bonuses[0], Some(0));
    }
}
