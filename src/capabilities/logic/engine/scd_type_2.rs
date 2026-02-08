//! SCD Type 2: Effective dating logic for historical tracking
//!
//! Uses Polars window functions to calculate effective dates and set `is_current`
//! flags **without** row-based looping.
//!
//! ## Algorithm
//!
//! 1. Sort by `employee_id`, then `start_date` ascending.
//! 2. Within each `employee_id` partition, shift `start_date` forward by 1 row
//!    to obtain `effective_to` (the next record's start date).
//! 3. If `effective_to` is null the record is the latest → `is_current = true`.
//! 4. Rename the original `start_date` column to `effective_from`.

use crate::domain::{OnboardingAction, Result, RosterContext};
use polars::prelude::*;

/// SCD Type 2 implementation for historical tracking
#[derive(Debug, Clone, Default)]
pub struct SCDType2;

impl SCDType2 {
    pub fn new() -> Self {
        Self
    }
}

impl OnboardingAction for SCDType2 {
    fn id(&self) -> &str {
        "scd_type_2"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!("SCDType2: computing effective dates");

        let lf = std::mem::replace(&mut context.data, LazyFrame::default());

        // 1. Sort by employee_id then start_date
        let lf = lf.sort(
            ["employee_id", "start_date"],
            SortMultipleOptions::default(),
        );

        // 2. Rename start_date → effective_from
        let lf = lf.rename(["start_date"], ["effective_from"], true);

        // 3. Compute effective_to = next record's effective_from within the
        //    same employee_id partition (shift -1 brings the *next* row's value).
        let lf = lf.with_column(
            col("effective_from")
                .shift(lit(-1))
                .over([col("employee_id")])
                .alias("effective_to"),
        );

        // 4. is_current = effective_to IS NULL (i.e. latest record)
        let lf = lf.with_column(
            col("effective_to")
                .is_null()
                .alias("is_current"),
        );

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
        let action = SCDType2::new();
        assert_eq!(action.id(), "scd_type_2");
    }

    #[test]
    fn test_scd_type_2_adds_columns() {
        let ctx = RosterContext::new(test_df().lazy());
        let action = SCDType2::new();
        let result = action.execute(ctx).expect("execute");

        let df = result.data.collect().expect("collect");
        assert!(df.column("effective_from").is_ok(), "should have effective_from");
        assert!(df.column("effective_to").is_ok(), "should have effective_to");
        assert!(df.column("is_current").is_ok(), "should have is_current");
        // start_date was renamed, should no longer exist
        assert!(df.column("start_date").is_err(), "start_date should be renamed");
    }

    #[test]
    fn test_scd_type_2_is_current_flag() {
        let ctx = RosterContext::new(test_df().lazy());
        let action = SCDType2::new();
        let result = action.execute(ctx).expect("execute");

        let df = result.data.collect().expect("collect");

        let is_current = df.column("is_current").unwrap();
        let bools: Vec<Option<bool>> = is_current
            .bool()
            .unwrap()
            .into_iter()
            .collect();

        // Within each employee_id group the last record should be current.
        // After sort by (employee_id, start_date), order is:
        //   001/2024-01-01  → not current
        //   001/2024-06-01  → current
        //   002/2024-02-15  → not current
        //   002/2024-08-01  → current
        assert_eq!(bools, vec![Some(false), Some(true), Some(false), Some(true)]);
    }

    #[test]
    fn test_scd_type_2_effective_to_values() {
        let ctx = RosterContext::new(test_df().lazy());
        let action = SCDType2::new();
        let result = action.execute(ctx).expect("execute");

        let df = result.data.collect().expect("collect");
        let eff_to = df.column("effective_to").unwrap();

        // First record of each group should have the second record's date.
        // Last record of each group should be null.
        let vals: Vec<Option<&str>> = eff_to.str().unwrap().into_iter().collect();
        assert_eq!(vals[0], Some("2024-06-01"));
        assert_eq!(vals[1], None); // latest for 001
        assert_eq!(vals[2], Some("2024-08-01"));
        assert_eq!(vals[3], None); // latest for 002
    }

    #[test]
    fn test_scd_type_2_field_metadata() {
        let ctx = RosterContext::new(test_df().lazy());
        let action = SCDType2::new();
        let result = action.execute(ctx).expect("execute");

        for col_name in ["effective_from", "effective_to", "is_current"] {
            let meta = result.field_metadata.get(col_name)
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
        let action = SCDType2::new();
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        // Every record is the only one → all should be current
        let is_current: Vec<Option<bool>> = df
            .column("is_current").unwrap()
            .bool().unwrap()
            .into_iter()
            .collect();
        assert!(is_current.iter().all(|v| *v == Some(true)));
    }
}
