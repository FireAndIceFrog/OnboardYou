//! PII protection: SSN/Salary masking based on residency rules
//!
//! ## Default behaviour
//!
//! | Field    | Masking rule                                        |
//! |----------|-----------------------------------------------------|
//! | `ssn`    | Keep last 4 characters, replace prefix with `***-**` |
//! | `salary` | Replace with `0`                                    |
//!
//! Columns to mask are configurable via manifest JSON:
//!
//! ```json
//! {
//!   "mask_ssn": true,
//!   "mask_salary": true
//! }
//! ```

use crate::capabilities::logic::traits::Masker;
use crate::domain::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;

/// Configuration for PII masking, extracted from manifest JSON.
#[derive(Debug, Clone)]
pub struct PIIMaskingConfig {
    /// Whether to mask the `ssn` column (default: true)
    pub mask_ssn: bool,
    /// Whether to mask the `salary` column (default: true)
    pub mask_salary: bool,
}

impl Default for PIIMaskingConfig {
    fn default() -> Self {
        Self {
            mask_ssn: true,
            mask_salary: true,
        }
    }
}

impl PIIMaskingConfig {
    /// Build from manifest `ActionConfig.config` JSON.
    pub fn from_json(value: &serde_json::Value) -> Self {
        Self {
            mask_ssn: value
                .get("mask_ssn")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            mask_salary: value
                .get("mask_salary")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
        }
    }
}

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
        tracing::info!("PIIMasking: applying PII masks");

        // Collect eagerly for reliable column operations
        let lf = std::mem::replace(&mut context.data, LazyFrame::default());
        let mut df = lf.collect().map_err(|e| {
            Error::LogicError(format!("Failed to collect for masking: {}", e))
        })?;

        let has_ssn = df.schema().contains("ssn");
        let has_salary = df.schema().contains("salary");

        // --- SSN masking: ***-**-XXXX (keep last 4) -------------------------
        if self.config.mask_ssn && has_ssn {
            let ssn_col = df.column("ssn").unwrap().str().unwrap();
            let masked: StringChunked = ssn_col
                .into_iter()
                .map(|opt_val| {
                    opt_val.map(|val| {
                        let len = val.len();
                        if len >= 4 {
                            format!("***-**-{}", &val[len - 4..])
                        } else {
                            "***-**-????".to_string()
                        }
                    })
                })
                .collect();
            let masked = masked.with_name("ssn".into());
            let _ = df.replace("ssn", masked);
            context.set_field_source("ssn".into(), "LOGIC_ACTION".into());
            context.mark_field_modified("ssn".into(), "pii_masking".into());
        }

        // --- Salary masking: replace with 0 ---------------------------------
        if self.config.mask_salary && has_salary {
            let salary_col = df.column("salary").unwrap();
            let n = salary_col.len();
            let dtype = salary_col.dtype().clone();

            // Create a zero column matching the original dtype
            let zeros = match dtype {
                DataType::Int32 => Series::new("salary".into(), vec![0i32; n]),
                DataType::Int64 => Series::new("salary".into(), vec![0i64; n]),
                DataType::Float32 => Series::new("salary".into(), vec![0.0f32; n]),
                DataType::Float64 => Series::new("salary".into(), vec![0.0f64; n]),
                _ => Series::new("salary".into(), vec![0i64; n]),
            };
            let _ = df.replace("salary", zeros);
            context.set_field_source("salary".into(), "LOGIC_ACTION".into());
            context.mark_field_modified("salary".into(), "pii_masking".into());
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
            mask_ssn: true,
            mask_salary: false,
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
            mask_ssn: false,
            mask_salary: true,
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
    fn test_from_action_config() {
        let json = serde_json::json!({ "mask_ssn": false, "mask_salary": true });
        let action = PIIMasking::from_action_config(&json);
        assert!(!action.config.mask_ssn);
        assert!(action.config.mask_salary);
    }
}
