//! Drop columns according to a list
//!
//! Configurable via manifest JSON:
//! ```json
//! {
//!   "columns": ["col1", "col2"]
//! }
//! ```

use crate::capabilities::logic::models::DropConfig;
use crate::capabilities::logic::traits::ColumnCalculator;
use crate::domain::{OnboardingAction, Result, RosterContext};
use polars::prelude::*;

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Drop columns according to a list.
///
/// Validates that all column names are unique, then applies a lazy drop
/// so the operation is folded into the Polars query plan without
/// materialising the frame.
#[derive(Debug, Clone)]
pub struct DropColumn {
    config: DropConfig,
}

impl DropColumn {
    pub fn new(config: DropConfig) -> Self {
        Self { config }
    }

    /// Construct from a deserialised config.
    pub fn from_action_config(config: &DropConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self::new(config.clone()))
    }
}

impl ColumnCalculator for DropColumn {
    fn calculate_columns(&self, mut context: RosterContext) -> Result<RosterContext> {
        let lf = std::mem::replace(&mut context.data, LazyFrame::default());
        let selector = Selector::ByName {
            names: self.config.columns.iter().map(|s| PlSmallStr::from(s.as_str())).collect::<Vec<_>>().into(),
            strict: true,
        };
        context.data = lf.drop(selector);
        Ok(context)
    }
}

impl OnboardingAction for DropColumn {
    fn id(&self) -> &str {
        "drop_column"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!(columns = ?self.config.columns, "DropColumn: dropping columns");

        let lf = std::mem::replace(&mut context.data, LazyFrame::default());
        let selector = Selector::ByName {
            names: self.config.columns.iter().map(|s| PlSmallStr::from(s.as_str())).collect::<Vec<_>>().into(),
            strict: true,
        };
        let lf = lf.drop(selector);

        // Optionally, mark columns as dropped in metadata if needed
        for col in &self.config.columns {
            context.mark_field_modified(col.clone(), "drop_column".into());
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

    fn sample_df() -> DataFrame {
        df! {
            "employee_id" => &["001", "002"],
            "first_name"  => &["John", "Jane"],
            "last_name"   => &["Doe", "Roe"],
        }
        .unwrap()
    }

    #[test]
    fn test_id() {
        let config = DropConfig { columns: vec![] };
        let act = DropColumn::new(config);
        assert_eq!(act.id(), "drop_column");
    }

    #[test]
    fn test_drop_columns() {
        let json = serde_json::json!({
            "columns": ["first_name", "last_name"]
        });
        let cfg: DropConfig = serde_json::from_value(json.clone()).expect("deserialise");
        let action = DropColumn::from_action_config(&cfg).expect("valid config");
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        assert!(df.column("first_name").is_err());
        assert!(df.column("last_name").is_err());
        assert!(df.column("employee_id").is_ok());
    }

    #[test]
    fn test_duplicate_columns_rejected_at_construction() {
        let cfg: DropConfig = serde_json::from_value(serde_json::json!({
            "columns": ["first_name", "first_name"]
        })).unwrap();
        assert!(DropColumn::from_action_config(&cfg).is_err());
    }

    #[test]
    fn test_missing_columns_key_rejected() {
        let json = serde_json::json!({ "not_columns": ["first_name"] });
        assert!(serde_json::from_value::<DropConfig>(json.clone()).is_err());
    }

    #[test]
    fn test_config_deserialise() {
        let json = serde_json::json!({
            "columns": ["a", "b"]
        });
        let config: DropConfig = serde_json::from_value(json).expect("deserialise");
        assert_eq!(config.columns.len(), 2);
        assert_eq!(config.columns[0], "a");
    }

    #[test]
    fn test_validate_ok() {
        let config = DropConfig {
            columns: vec!["a".into(), "b".into()],
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_duplicate_columns() {
        let config = DropConfig {
            columns: vec!["z".into(), "z".into()],
        };
        assert!(config.validate().is_err());
    }
}
