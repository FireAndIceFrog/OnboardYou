//! The Resolver: Maps manifest ActionType variants to Capability instances
//!
//! Uses the `ActionConfig` from the manifest to instantiate the correct
//! `OnboardingAction` implementation, forwarding action-specific JSON config.
//!
//! The match is **exhaustive** — adding a new `ActionType` variant forces
//! a compiler error here until you wire it up.

use async_trait::async_trait;
use crate::RosterContext;
use crate::capabilities::egress::api_dispatcher::ApiDispatcher;
use crate::capabilities::ingestion::engine::{CsvHrisConnector, WorkdayHrisConnector};
use crate::capabilities::logic::engine::{
    CellphoneSanitizer, DropColumn, FilterByValue, HandleDiacritics,
    IdentityDeduplicator, IsoCountrySanitizer, PIIMasking, RegexReplace,
    RenameColumn, SCDType2,
};
use crate::domain::{ActionType, OnboardingAction, Result, Error};
use crate::domain::models::manifest::{ActionConfig, ActionConfigPayload};
use std::sync::Arc;

/// Factory for creating OnboardingAction instances from manifest action configs.
pub struct ActionFactory;

impl ActionFactory {
    pub fn new() -> Self {
        ActionFactory {}
    }
}

#[async_trait]
pub trait ActionFactoryTrait:  Send + Sync  {
    fn create(&self, action_config: &ActionConfig) -> Result<Arc<dyn OnboardingAction>>;
    fn run(
        &self,
        actions: Vec<Arc<dyn OnboardingAction>>,
        context: RosterContext,
    ) -> Result<RosterContext>;
}

impl ActionFactoryTrait for ActionFactory {
    /// Create an action instance from a full `ActionConfig`.
    ///
    /// The `action_type` enum selects the concrete implementation, while
    /// `config` is forwarded as-is to the implementation's constructor.
    fn create(&self, action_config: &ActionConfig) -> Result<Arc<dyn OnboardingAction>> {
        let action_type = action_config.action_type.clone();
        let payload = action_config.config.clone();

        match (action_type, payload) {
            (ActionType::CsvHrisConnector, ActionConfigPayload::CsvHrisConnector(cfg)) => {
                let connector = CsvHrisConnector::from_action_config(&cfg)?;
                Ok(Arc::new(connector))
            }
            (ActionType::WorkdayHrisConnector, ActionConfigPayload::WorkdayHrisConnector(cfg)) => {
                let connector = WorkdayHrisConnector::from_action_config(&cfg)?;
                Ok(Arc::new(connector))
            }
            (ActionType::ScdType2, ActionConfigPayload::ScdType2(cfg)) => {
                let scd = SCDType2::from_action_config(&cfg)?;
                Ok(Arc::new(scd))
            }
            (ActionType::PiiMasking, ActionConfigPayload::PiiMasking(cfg)) => {
                let masking = PIIMasking::from_action_config(&cfg)?;
                Ok(Arc::new(masking))
            }
            (ActionType::IdentityDeduplicator, ActionConfigPayload::IdentityDeduplicator(cfg)) => {
                let dedup = IdentityDeduplicator::from_action_config(&cfg)?;
                Ok(Arc::new(dedup))
            }
            (ActionType::RegexReplace, ActionConfigPayload::RegexReplace(cfg)) => {
                let action = RegexReplace::from_action_config(&cfg)?;
                Ok(Arc::new(action))
            }
            (ActionType::IsoCountrySanitizer, ActionConfigPayload::IsoCountrySanitizer(cfg)) => {
                let action = IsoCountrySanitizer::from_action_config(&cfg)?;
                Ok(Arc::new(action))
            }
            (ActionType::CellphoneSanitizer, ActionConfigPayload::CellphoneSanitizer(cfg)) => {
                let action = CellphoneSanitizer::from_action_config(&cfg)?;
                Ok(Arc::new(action))
            }
            (ActionType::HandleDiacritics, ActionConfigPayload::HandleDiacritics(cfg)) => {
                let action = HandleDiacritics::from_action_config(&cfg)?;
                Ok(Arc::new(action))
            }
            (ActionType::RenameColumn, ActionConfigPayload::RenameColumn(cfg)) => {
                let action = RenameColumn::from_action_config(&cfg)?;
                Ok(Arc::new(action))
            }
            (ActionType::DropColumn, ActionConfigPayload::DropColumn(cfg)) => {
                let action = DropColumn::from_action_config(&cfg)?;
                Ok(Arc::new(action))
            }
            (ActionType::FilterByValue, ActionConfigPayload::FilterByValue(cfg)) => {
                let action = FilterByValue::from_action_config(&cfg)?;
                Ok(Arc::new(action))
            }
            (ActionType::ApiDispatcher, ActionConfigPayload::ApiDispatcher(cfg)) => {
                if cfg.is_default() {
                    // Default is a meta-type resolved at runtime by the ETL trigger.
                    // For validation / column calculation we only need an unconfigured
                    // dispatcher — calculate_columns() is a pass-through for egress.
                    Ok(Arc::new(ApiDispatcher::new()))
                } else {
                    let action = ApiDispatcher::from_action_config(&cfg)?;
                    Ok(Arc::new(action))
                }
            }
            (t, _) => Err(Error::ConfigurationError(format!("Mismatched payload for action type {t:?}"))),
        }
    }

    /// Execute a pipeline defined by a manifest.
    ///
    /// Each action receives the `RosterContext` produced by the previous step
    /// (fold pattern). The final context is returned.
    fn run(
        &self,
        actions: Vec<Arc<dyn OnboardingAction>>,
        mut context: RosterContext,
    ) -> Result<RosterContext> {
        for action in &actions {
            tracing::info!(action_id = action.id(), "PipelineRunner: executing action");
            context = action.execute(context)?;
        }
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ActionType;
    use crate::capabilities::logic::traits::ColumnCalculator;
    use crate::domain::models::manifest::{ActionConfig, ActionConfigPayload};
    use polars::prelude::*;

    /// A trivial pass-through action for testing the runner.
    struct NoopAction;
    impl ColumnCalculator for NoopAction {
        fn calculate_columns(&self, ctx: RosterContext) -> Result<RosterContext> {
            Ok(ctx)
        }
    }
    impl OnboardingAction for NoopAction {
        fn id(&self) -> &str {
            "noop"
        }
        fn execute(&self, ctx: RosterContext) -> Result<RosterContext> {
            Ok(ctx)
        }
    }

    #[test]
    fn test_pipeline_runner_no_actions() {
        let ctx = RosterContext::new(LazyFrame::default());
        let result = ActionFactory::new().run(vec![], ctx).expect("run");
        assert!(result.field_metadata.is_empty());
    }

    #[test]
    fn test_pipeline_runner_single_action() {
        let ctx = RosterContext::new(LazyFrame::default());
        let actions: Vec<Arc<dyn OnboardingAction>> = vec![Arc::new(NoopAction)];
        let result = ActionFactory::new().run(actions, ctx).expect("run");
        assert!(result.field_metadata.is_empty());
    }

    #[test]
    fn test_factory_creates_csv_connector() {
        let config = ActionConfig {
            id: "ingest".into(),
            action_type: ActionType::CsvHrisConnector,
            config: ActionConfigPayload::CsvHrisConnector(serde_json::from_value(serde_json::json!({ "filename": "data.csv", "columns": ["a", "b"] })).unwrap()),
        };
        let action = ActionFactory::new().create(&config).expect("should create csv connector");
        assert_eq!(action.id(), "csv_hris_connector");
    }

    #[test]
    fn test_factory_creates_scd_type_2() {
        let config = ActionConfig {
            id: "scd".into(),
            action_type: ActionType::ScdType2,
            config: ActionConfigPayload::ScdType2(serde_json::from_value(serde_json::json!({
                "entity_column": "employee_id",
                "date_column": "start_date"
            })).unwrap()),
        };
        let action = ActionFactory::new().create(&config).expect("should create scd_type_2");
        assert_eq!(action.id(), "scd_type_2");
    }

    #[test]
    fn test_factory_creates_pii_masking() {
        let config = ActionConfig {
            id: "mask".into(),
            action_type: ActionType::PiiMasking,
            config: ActionConfigPayload::PiiMasking(serde_json::from_value(serde_json::json!({ "mask_ssn": true, "mask_salary": false })).unwrap()),
        };
        let action = ActionFactory::new().create(&config).expect("should create pii_masking");
        assert_eq!(action.id(), "pii_masking");
    }

    #[test]
    fn test_factory_creates_identity_deduplicator() {
        let config = ActionConfig {
            id: "dedup".into(),
            action_type: ActionType::IdentityDeduplicator,
            config: ActionConfigPayload::IdentityDeduplicator(serde_json::from_value(serde_json::json!({ "columns": ["email"] })).unwrap()),
        };
        let action = ActionFactory::new().create(&config).expect("should create identity_deduplicator");
        assert_eq!(action.id(), "identity_deduplicator");
    }

    #[test]
    fn test_factory_creates_regex_replace() {
        let config = ActionConfig {
            id: "clean_phone".into(),
            action_type: ActionType::RegexReplace,
            config: ActionConfigPayload::RegexReplace(serde_json::from_value(serde_json::json!({
                "column": "phone",
                "pattern": "\\+44\\s?",
                "replacement": "0"
            })).unwrap()),
        };
        let action = ActionFactory::new().create(&config).expect("should create regex_replace");
        assert_eq!(action.id(), "regex_replace");
    }

    #[test]
    fn test_factory_creates_cellphone_sanitizer() {
        let config = ActionConfig {
            id: "phone_intl".into(),
            action_type: ActionType::CellphoneSanitizer,
            config: ActionConfigPayload::CellphoneSanitizer(serde_json::from_value(serde_json::json!({
                "phone_column": "mobile_phone",
                "country_columns": ["work_country", "home_country"],
                "output_column": "phone_intl"
            })).unwrap()),
        };
        let action = ActionFactory::new().create(&config).expect("should create cellphone_sanitizer");
        assert_eq!(action.id(), "cellphone_sanitizer");
    }

    #[test]
    fn test_unknown_action_type_rejected_at_deserialization() {
        // Unknown action types are now caught by serde, not the factory
        let json = r#"{"id":"bad","action_type":"nope","config":{}}"#;
        let result: std::result::Result<ActionConfig, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_action_type_round_trip_serde() {
        let config = ActionConfig {
            id: "test".into(),
            action_type: ActionType::ApiDispatcher,
            config: ActionConfigPayload::ApiDispatcher(serde_json::from_value(serde_json::json!({
                "auth_type": "default"
            })).unwrap()),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"api_dispatcher\""));
        let back: ActionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.action_type, ActionType::ApiDispatcher);
    }

    #[test]
    fn test_factory_creates_api_dispatcher_default() {
        let config = ActionConfig {
            id: "egress".into(),
            action_type: ActionType::ApiDispatcher,
            config: ActionConfigPayload::ApiDispatcher(serde_json::from_value(serde_json::json!({
                "auth_type": "default"
            })).unwrap()),
        };
        let action = ActionFactory::new().create(&config).expect("Default should create an unconfigured dispatcher");
        assert_eq!(action.id(), "api_dispatcher");
    }
}
