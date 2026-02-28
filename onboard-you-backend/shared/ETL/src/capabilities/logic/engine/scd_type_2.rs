//! SCD Type 2: Effective dating logic for historical tracking
//!
//! Uses Polars window functions to calculate effective dates and set `is_current`
//! flags **without** row-based looping.
//!
//! ## Algorithm
//!
//! 1. Sort by configured `entity_column`, then `date_column` ascending.
//! 2. Within each entity partition, shift the date forward by 1 row
//!    to obtain `effective_to` (the next record's date).
//! 3. If `effective_to` is null the record is the latest → `is_current = true`.
//! 4. Rename the original date column to `effective_from`.
//!
//! Configurable via manifest JSON:
//! ```json
//! {
//!   "entity_column": "employee_id",
//!   "date_column": "start_date"
//! }
//! ```

use onboard_you_models::ScdType2Config;
use onboard_you_models::ColumnCalculator;
use onboard_you_models::{OnboardingAction, Result, RosterContext};
use polars::prelude::*;

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// SCD Type 2 implementation for historical tracking.
///
/// Adds `effective_from`, `effective_to`, and `is_current` columns using
/// Polars window functions.
///
/// # Output columns
///
/// | Column           | Type   | Description                                    |
/// |------------------|--------|------------------------------------------------|
/// | `effective_from` | string | Renamed from `date_column`                     |
/// | `effective_to`   | string | Next record's date within the entity partition  |
/// | `is_current`     | bool   | `true` for the latest record per entity         |
#[derive(Debug, Clone)]
pub struct SCDType2 {
    config: ScdType2Config,
}

impl SCDType2 {
    pub fn new(config: ScdType2Config) -> Self {
        Self { config }
    }

    /// Construct from a deserialised config.
    pub fn from_action_config(config: &ScdType2Config) -> Result<Self> {
        Ok(Self::new(config.clone()))
    }
}

impl Default for SCDType2 {
    fn default() -> Self {
        Self::new(ScdType2Config::default())
    }
}

impl ColumnCalculator for SCDType2 {
    fn calculate_columns(&self, mut context: RosterContext) -> Result<RosterContext> {
        let lf = std::mem::replace(&mut context.data, LazyFrame::default());

        // Rename date_column → effective_from
        let lf = lf.rename([self.config.date_column.as_str()], ["effective_from"], true);

        // Add effective_to and is_current columns
        let lf = lf
            .with_column(lit(NULL).cast(DataType::String).alias("effective_to"))
            .with_column(lit(NULL).cast(DataType::Boolean).alias("is_current"));

        context.data = lf;
        for col_name in ["effective_from", "effective_to", "is_current"] {
            context.set_field_source(col_name.to_string(), "LOGIC_ACTION".into());
        }
        Ok(context)
    }
}

impl OnboardingAction for SCDType2 {
    fn id(&self) -> &str {
        "scd_type_2"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            entity = %self.config.entity_column,
            date = %self.config.date_column,
            "SCDType2: computing effective dates"
        );

        let lf = std::mem::replace(&mut context.data, LazyFrame::default());

        // 1. Sort by entity_column then date_column
        let lf = lf.sort(
            [&self.config.entity_column, &self.config.date_column],
            SortMultipleOptions::default(),
        );

        // 2. Rename date_column → effective_from
        let lf = lf.rename([self.config.date_column.as_str()], ["effective_from"], true);

        // 3. Compute effective_to = next record's effective_from within the
        //    same entity partition (shift -1 brings the *next* row's value).
        let lf = lf.with_column(
            col("effective_from")
                .shift(lit(-1))
                .over([col(&self.config.entity_column)])
                .alias("effective_to"),
        );

        // 4. is_current = effective_to IS NULL (i.e. latest record)
        let lf = lf.with_column(col("effective_to").is_null().alias("is_current"));

        // 5. Update field metadata
        for col_name in ["effective_from", "effective_to", "is_current"] {
            context.set_field_source(col_name.to_string(), "LOGIC_ACTION".into());
            context.mark_field_modified(col_name.to_string(), "scd_type_2".into());
        }

        context.data = lf;
        Ok(context)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a small DataFrame with two employees, each having two records.
    fn test_df() -> DataFrame {
        df! {
            "employee_id" => &["001", "001", "002", "002"],
            "first_name"  => &["John", "John", "Jane", "Jane"],
            "salary"      => &[70_000i64, 75_000, 85_000, 90_000],
            "start_date"  => &["2024-01-01", "2024-06-01", "2024-02-15", "2024-08-01"],
        }
        .expect("test df")
    }

    #[test]
    fn test_scd_type_2_id() {
        let action = SCDType2::default();
        assert_eq!(action.id(), "scd_type_2");
    }

    #[test]
    fn test_scd_type_2_adds_columns() {
        let ctx = RosterContext::new(test_df().lazy());
        let action = SCDType2::default();
        let result = action.execute(ctx).expect("execute");

        let df = result.data.collect().expect("collect");
        assert!(
            df.column("effective_from").is_ok(),
            "should have effective_from"
        );
        assert!(
            df.column("effective_to").is_ok(),
            "should have effective_to"
        );
        assert!(df.column("is_current").is_ok(), "should have is_current");
        // start_date was renamed, should no longer exist
        assert!(
            df.column("start_date").is_err(),
            "start_date should be renamed"
        );
    }

    #[test]
    fn test_scd_type_2_is_current_flag() {
        let ctx = RosterContext::new(test_df().lazy());
        let action = SCDType2::default();
        let result = action.execute(ctx).expect("execute");

        let df = result.data.collect().expect("collect");

        let is_current = df.column("is_current").unwrap();
        let bools: Vec<Option<bool>> = is_current.bool().unwrap().into_iter().collect();

        assert_eq!(
            bools,
            vec![Some(false), Some(true), Some(false), Some(true)]
        );
    }

    #[test]
    fn test_scd_type_2_effective_to_values() {
        let ctx = RosterContext::new(test_df().lazy());
        let action = SCDType2::default();
        let result = action.execute(ctx).expect("execute");

        let df = result.data.collect().expect("collect");
        let eff_to = df.column("effective_to").unwrap();

        let vals: Vec<Option<&str>> = eff_to.str().unwrap().into_iter().collect();
        assert_eq!(vals[0], Some("2024-06-01"));
        assert_eq!(vals[1], None);
        assert_eq!(vals[2], Some("2024-08-01"));
        assert_eq!(vals[3], None);
    }

    #[test]
    fn test_scd_type_2_field_metadata() {
        let ctx = RosterContext::new(test_df().lazy());
        let action = SCDType2::default();
        let result = action.execute(ctx).expect("execute");

        for col_name in ["effective_from", "effective_to", "is_current"] {
            let meta = result
                .field_metadata
                .get(col_name)
                .unwrap_or_else(|| panic!("metadata for '{}'", col_name));
            assert_eq!(meta.source, "LOGIC_ACTION");
            assert_eq!(meta.modified_by.as_deref(), Some("scd_type_2"));
        }
    }

    #[test]
    fn test_scd_type_2_single_record_per_employee() {
        let df = df! {
            "employee_id" => &["001", "002"],
            "first_name"  => &["Solo", "Only"],
            "salary"      => &[50_000i64, 60_000],
            "start_date"  => &["2024-01-01", "2024-03-01"],
        }
        .unwrap();

        let ctx = RosterContext::new(df.lazy());
        let action = SCDType2::default();
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let is_current: Vec<Option<bool>> = df
            .column("is_current")
            .unwrap()
            .bool()
            .unwrap()
            .into_iter()
            .collect();
        assert!(is_current.iter().all(|v| *v == Some(true)));
    }

    #[test]
    fn test_config_from_json() {
        let json = serde_json::json!({
            "entity_column": "worker_id",
            "date_column": "hire_date"
        });
        let config: ScdType2Config = serde_json::from_value(json).unwrap();
        assert_eq!(config.entity_column, "worker_id");
        assert_eq!(config.date_column, "hire_date");
    }

    #[test]
    fn test_config_defaults() {
        let json = serde_json::json!({});
        let config: ScdType2Config = serde_json::from_value(json).unwrap();
        assert_eq!(config.entity_column, "employee_id");
        assert_eq!(config.date_column, "start_date");
    }

    #[test]
    fn test_custom_columns() {
        let df = df! {
            "worker_id"  => &["W1", "W1", "W2"],
            "hire_date"  => &["2024-01-01", "2024-06-01", "2024-03-01"],
            "salary"     => &[50_000i64, 55_000, 60_000],
        }
        .unwrap();

        let config = ScdType2Config {
            entity_column: "worker_id".into(),
            date_column: "hire_date".into(),
        };
        let ctx = RosterContext::new(df.lazy());
        let action = SCDType2::new(config);
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        // hire_date should be renamed to effective_from
        assert!(df.column("effective_from").is_ok());
        assert!(df.column("hire_date").is_err());

        let is_current: Vec<Option<bool>> = df
            .column("is_current")
            .unwrap()
            .bool()
            .unwrap()
            .into_iter()
            .collect();
        // W1 has 2 records: first not current, second current
        // W2 has 1 record: current
        assert_eq!(is_current, vec![Some(false), Some(true), Some(true)]);
    }
}
