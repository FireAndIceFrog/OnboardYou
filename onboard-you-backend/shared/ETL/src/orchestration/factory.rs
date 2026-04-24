//! The Resolver: Maps manifest ActionType variants to Capability instances
//!
//! Uses the `ActionConfig` from the manifest to instantiate the correct
//! `OnboardingAction` implementation, forwarding action-specific JSON config.
//!
//! The match is **exhaustive** — adding a new `ActionType` variant forces
//! a compiler error here until you wire it up.

use crate::capabilities::egress::api_dispatcher::ApiDispatcher;
use crate::capabilities::egress::show_data::ShowData;
use crate::capabilities::ingestion::engine::{GenericIngestionConnector, WorkdayHrisConnector, SageHrConnector};
use crate::capabilities::logic::engine::{
    CellphoneSanitizer, DropColumn, FilterByValue, HandleDiacritics, IdentityDeduplicator,
    IsoCountrySanitizer, PIIMasking, RegexReplace, RenameColumn, SCDType2,
};
use onboard_you_models::{ActionConfig, ActionConfigPayload};
use onboard_you_models::{ActionType, Error, Manifest, OnboardingAction, Result};
use onboard_you_models::{
    ETLDependancies, RosterContext, StepValidation, ValidationResult, ValidationStepError,
};
use async_trait::async_trait;
use polars::prelude::LazyFrame;
use std::sync::Arc;

/// Factory for creating OnboardingAction instances from manifest action configs.
pub struct ActionFactory;

/// Captures which action failed and the error details.
#[derive(Debug)]
pub struct StepError {
    pub action_id: String,
    pub error: onboard_you_models::Error,
    /// Warnings accumulated by earlier (successful) actions before this step failed.
    pub warnings: Vec<onboard_you_models::PipelineWarning>,
}

impl std::fmt::Display for StepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Action '{}' failed: {}", self.action_id, self.error)
    }
}

impl ActionFactory {
    pub fn new() -> Self {
        ActionFactory {}
    }
}

#[async_trait]
pub trait ActionFactoryTrait: Send + Sync {
    fn create(&self, action_config: &ActionConfig) -> Result<Arc<dyn OnboardingAction>>;
    fn validate_manifest(
        &self,
        manifest: &Manifest,
    ) -> std::result::Result<ValidationResult, ValidationStepError>;
    fn run(
        &self,
        actions: Vec<Arc<dyn OnboardingAction>>,
        context: RosterContext,
    ) -> std::result::Result<RosterContext, StepError>;
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
            (ActionType::WorkdayHrisConnector, ActionConfigPayload::WorkdayHrisConnector(cfg)) => {
                let connector = WorkdayHrisConnector::from_action_config(&cfg)?;
                Ok(Arc::new(connector))
            }
            (ActionType::SageHrConnector, ActionConfigPayload::SageHrConnector(cfg)) => {
                let connector = SageHrConnector::from_action_config(&cfg)?;
                Ok(Arc::new(connector))
            }
            (ActionType::GenericIngestionConnector, ActionConfigPayload::GenericIngestionConnector(cfg)) => {
                let connector = GenericIngestionConnector::from_action_config(&cfg)?;
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
            (ActionType::ShowData, ActionConfigPayload::ShowData(cfg)) => {
                // During validation (s3_key is None) create an unresolved stub —
                // calculate_columns() is a pass-through so no S3 key is needed.
                if cfg.s3_key.is_some() {
                    let action = ShowData::from_action_config(&cfg)?;
                    Ok(Arc::new(action))
                } else {
                    // Validation path: wrap a dummy that passes columns through.
                    Ok(Arc::new(ShowDataStub))
                }
            }
            (t, _) => Err(Error::ConfigurationError(format!(
                "Mismatched payload for action type {t:?}"
            ))),
        }
    }

    /// Validate a manifest by chaining `calculate_columns` for each action.
    fn validate_manifest(
        &self,
        manifest: &Manifest,
    ) -> std::result::Result<ValidationResult, ValidationStepError> {
        if manifest.actions.is_empty() {
            return Ok(ValidationResult {
                steps: vec![],
                final_columns: vec![],
            });
        }

        let actions: Vec<_> = manifest
            .actions
            .iter()
            .map(|ac| {
                self.create(ac).map_err(|e| ValidationStepError {
                    action_id: ac.id.clone(),
                    action_type: ac.action_type.to_string(),
                    error: e,
                })
            })
            .collect::<std::result::Result<_, _>>()?;

        let mut context = RosterContext::with_deps(LazyFrame::default(), ETLDependancies::default());
        let mut steps = Vec::with_capacity(actions.len());

        for (action, ac) in actions.iter().zip(manifest.actions.iter()) {
            context = action.calculate_columns(context).map_err(|e| ValidationStepError {
                action_id: ac.id.clone(),
                action_type: ac.action_type.to_string(),
                error: e,
            })?;

            let schema = context
                .get_data()
                .collect_schema()
                .map_err(|e| ValidationStepError {
                    action_id: ac.id.clone(),
                    action_type: ac.action_type.to_string(),
                    error: e.into(),
                })?;

            let columns_after: Vec<String> = schema.iter_names().map(|n| n.to_string()).collect();

            steps.push(StepValidation {
                action_id: ac.id.clone(),
                action_type: ac.action_type.to_string(),
                columns_after,
            });
        }

        let final_columns = steps
            .last()
            .map(|s| s.columns_after.clone())
            .unwrap_or_default();

        Ok(ValidationResult {
            steps,
            final_columns,
        })
    }

    /// Execute a pipeline defined by a manifest.
    ///
    /// Each action receives the `RosterContext` produced by the previous step
    /// (fold pattern). The final context is returned. On failure the
    /// `StepError` identifies which action broke.
    fn run(
        &self,
        actions: Vec<Arc<dyn OnboardingAction>>,
        mut context: RosterContext,
    ) -> std::result::Result<RosterContext, StepError> {
        for action in &actions {
            tracing::info!(action_id = action.id(), "PipelineRunner: executing action");
            let logger = context.deps.logger.clone();

            context = action.execute(context).map_err(|e| {
                StepError {
                    action_id: action.id().to_string(),
                    error: e,
                    warnings: logger.drain_deferred_warnings(),
                }
            })?;
        }
        Ok(context)
    }
}

/// Validation-only stub for `ShowData`.
///
/// When the manifest is being validated (no S3 key yet) we still need to
/// handle the `ShowData` arm in the factory.  This struct implements the
/// pass-through `ColumnCalculator` and a no-op `OnboardingAction::execute`
/// so column propagation works without an S3 key.
struct ShowDataStub;
impl onboard_you_models::ColumnCalculator for ShowDataStub {
    fn calculate_columns(&self, ctx: RosterContext) -> Result<RosterContext> {
        Ok(ctx)
    }
}
impl OnboardingAction for ShowDataStub {
    fn id(&self) -> &str {
        "show_data"
    }
    fn execute(&self, ctx: RosterContext) -> Result<RosterContext> {
        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use onboard_you_models::ETLDependancies;
    use super::*;
    use onboard_you_models::ColumnCalculator;
    use onboard_you_models::manifest::{ActionConfig, ActionConfigPayload};
    use ActionType;

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
        let ctx = RosterContext::with_deps(LazyFrame::default(), ETLDependancies::default());
        let result = ActionFactory::new().run(vec![], ctx).expect("run");
        assert!(result.field_metadata().is_empty());
    }

    #[test]
    fn test_pipeline_runner_single_action() {
        let ctx = RosterContext::with_deps(LazyFrame::default(), ETLDependancies::default());
        let actions: Vec<Arc<dyn OnboardingAction>> = vec![Arc::new(NoopAction)];
        let result = ActionFactory::new().run(actions, ctx).expect("run");
        assert!(result.field_metadata().is_empty());
    }

    #[test]
    fn test_factory_creates_csv_connector() {
        let config = ActionConfig {
            id: "ingest".into(),
            action_type: ActionType::GenericIngestionConnector,
            config: ActionConfigPayload::GenericIngestionConnector(
                serde_json::from_value(
                    serde_json::json!({ "filename": "data.csv", "columns": ["a", "b"] }),
                )
                .unwrap(),
            ),
        };
        let action = ActionFactory::new()
            .create(&config)
            .expect("should create generic ingestion connector");
        assert_eq!(action.id(), "generic_ingestion_connector");
    }

    #[test]
    fn test_factory_creates_scd_type_2() {
        let config = ActionConfig {
            id: "scd".into(),
            action_type: ActionType::ScdType2,
            config: ActionConfigPayload::ScdType2(
                serde_json::from_value(serde_json::json!({
                    "entity_column": "employee_id",
                    "date_column": "start_date"
                }))
                .unwrap(),
            ),
        };
        let action = ActionFactory::new()
            .create(&config)
            .expect("should create scd_type_2");
        assert_eq!(action.id(), "scd_type_2");
    }

    #[test]
    fn test_factory_creates_pii_masking() {
        let config = ActionConfig {
            id: "mask".into(),
            action_type: ActionType::PiiMasking,
            config: ActionConfigPayload::PiiMasking(
                serde_json::from_value(
                    serde_json::json!({ "mask_ssn": true, "mask_salary": false }),
                )
                .unwrap(),
            ),
        };
        let action = ActionFactory::new()
            .create(&config)
            .expect("should create pii_masking");
        assert_eq!(action.id(), "pii_masking");
    }

    #[test]
    fn test_factory_creates_identity_deduplicator() {
        let config = ActionConfig {
            id: "dedup".into(),
            action_type: ActionType::IdentityDeduplicator,
            config: ActionConfigPayload::IdentityDeduplicator(
                serde_json::from_value(serde_json::json!({ "columns": ["email"] })).unwrap(),
            ),
        };
        let action = ActionFactory::new()
            .create(&config)
            .expect("should create identity_deduplicator");
        assert_eq!(action.id(), "identity_deduplicator");
    }

    #[test]
    fn test_factory_creates_regex_replace() {
        let config = ActionConfig {
            id: "clean_phone".into(),
            action_type: ActionType::RegexReplace,
            config: ActionConfigPayload::RegexReplace(
                serde_json::from_value(serde_json::json!({
                    "column": "phone",
                    "pattern": "\\+44\\s?",
                    "replacement": "0"
                }))
                .unwrap(),
            ),
        };
        let action = ActionFactory::new()
            .create(&config)
            .expect("should create regex_replace");
        assert_eq!(action.id(), "regex_replace");
    }

    #[test]
    fn test_factory_creates_cellphone_sanitizer() {
        let config = ActionConfig {
            id: "phone_intl".into(),
            action_type: ActionType::CellphoneSanitizer,
            config: ActionConfigPayload::CellphoneSanitizer(
                serde_json::from_value(serde_json::json!({
                    "phone_column": "mobile_phone",
                    "country_columns": ["work_country", "home_country"],
                    "output_column": "phone_intl"
                }))
                .unwrap(),
            ),
        };
        let action = ActionFactory::new()
            .create(&config)
            .expect("should create cellphone_sanitizer");
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
            config: ActionConfigPayload::ApiDispatcher(
                serde_json::from_value(serde_json::json!({
                    "auth_type": "default"
                }))
                .unwrap(),
            ),
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
            config: ActionConfigPayload::ApiDispatcher(
                serde_json::from_value(serde_json::json!({
                    "auth_type": "default"
                }))
                .unwrap(),
            ),
        };
        let action = ActionFactory::new()
            .create(&config)
            .expect("Default should create an unconfigured dispatcher");
        assert_eq!(action.id(), "api_dispatcher");
    }

    // -----------------------------------------------------------------------
    // validate_manifest tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_manifest_empty_actions() {
        let manifest = onboard_you_models::Manifest {
            version: "1.0".into(),
            actions: vec![],
        };

        let result = ActionFactory::new()
            .validate_manifest(&manifest)
            .expect("empty manifest should validate");

        assert!(result.steps.is_empty());
        assert!(result.final_columns.is_empty());
    }

    #[test]
    fn test_validate_manifest_csv_then_dispatcher_propagates_columns() {
        let manifest = onboard_you_models::Manifest {
            version: "1.0".into(),
            actions: vec![
                ActionConfig {
                    id: "ingest".into(),
                    action_type: ActionType::GenericIngestionConnector,
                    config: ActionConfigPayload::GenericIngestionConnector(
                        serde_json::from_value(
                            serde_json::json!({
                                "filename": "data.csv",
                                "columns": ["employee_id", "email"]
                            }),
                        )
                        .unwrap(),
                    ),
                },
                ActionConfig {
                    id: "dispatch".into(),
                    action_type: ActionType::ApiDispatcher,
                    config: ActionConfigPayload::ApiDispatcher(
                        serde_json::from_value(serde_json::json!({ "auth_type": "default" }))
                            .unwrap(),
                    ),
                },
            ],
        };

        let result = ActionFactory::new()
            .validate_manifest(&manifest)
            .expect("manifest should validate");

        assert_eq!(result.steps.len(), 2);
        assert_eq!(result.steps[0].action_id, "ingest");
        assert_eq!(result.steps[1].action_id, "dispatch");
        assert_eq!(result.steps[0].columns_after, vec!["employee_id", "email"]);
        assert_eq!(result.steps[1].columns_after, vec!["employee_id", "email"]);
        assert_eq!(result.final_columns, vec!["employee_id", "email"]);
    }

    #[test]
    fn test_validate_manifest_reports_action_for_payload_mismatch() {
        let manifest = onboard_you_models::Manifest {
            version: "1.0".into(),
            actions: vec![ActionConfig {
                id: "bad_step".into(),
                action_type: ActionType::GenericIngestionConnector,
                // Intentional mismatch: generic ingestion type with api dispatcher payload
                config: ActionConfigPayload::ApiDispatcher(
                    onboard_you_models::ApiDispatcherConfig::Default,
                ),
            }],
        };

        let err = ActionFactory::new()
            .validate_manifest(&manifest)
            .expect_err("mismatched payload should fail");

        assert_eq!(err.action_id, "bad_step");
        assert_eq!(err.action_type, "generic_ingestion_connector");
        assert!(matches!(
            err.error,
            onboard_you_models::Error::ConfigurationError(_)
        ));
    }

    // -----------------------------------------------------------------------
    // StepError warning preservation tests
    // -----------------------------------------------------------------------

    /// An action that adds a warning, then succeeds.
    struct WarnThenSucceed;
    impl onboard_you_models::ColumnCalculator for WarnThenSucceed {
        fn calculate_columns(&self, ctx: RosterContext) -> Result<RosterContext> {
            Ok(ctx)
        }
    }
    impl OnboardingAction for WarnThenSucceed {
        fn id(&self) -> &str {
            "warn_action"
        }
        fn execute(&self, ctx: RosterContext) -> Result<RosterContext> {
            ctx.deps.logger.warn(onboard_you_models::PipelineWarning {
                action_id: self.id().to_string(),
                message: "test warning".into(),
                count: 1,
                detail: None,
            });
            Ok(ctx)
        }
    }

    /// An action that always fails.
    struct AlwaysFail;
    impl onboard_you_models::ColumnCalculator for AlwaysFail {
        fn calculate_columns(&self, ctx: RosterContext) -> Result<RosterContext> {
            Ok(ctx)
        }
    }
    impl OnboardingAction for AlwaysFail {
        fn id(&self) -> &str {
            "failing_action"
        }
        fn execute(&self, _ctx: RosterContext) -> Result<RosterContext> {
            Err(onboard_you_models::Error::LogicError("boom".into()))
        }
    }

    #[test]
    fn test_step_error_preserves_warnings_from_earlier_steps() {
        let ctx = RosterContext::with_deps(LazyFrame::default(), ETLDependancies::default());
        let actions: Vec<Arc<dyn OnboardingAction>> = vec![
            Arc::new(WarnThenSucceed),
            Arc::new(AlwaysFail),
        ];
        let err = ActionFactory::new()
            .run(actions, ctx)
            .expect_err("should fail");

        assert_eq!(err.action_id, "failing_action");
        assert_eq!(err.warnings.len(), 1);
        assert_eq!(err.warnings[0].action_id, "warn_action");
        assert_eq!(err.warnings[0].message, "test warning");
    }

    #[test]
    fn test_step_error_empty_warnings_when_first_action_fails() {
        let ctx = RosterContext::with_deps(LazyFrame::default(), ETLDependancies::default());
        let actions: Vec<Arc<dyn OnboardingAction>> = vec![Arc::new(AlwaysFail)];
        let err = ActionFactory::new()
            .run(actions, ctx)
            .expect_err("should fail");

        assert_eq!(err.action_id, "failing_action");
        assert!(err.warnings.is_empty());
    }

    #[test]
    fn test_successful_pipeline_preserves_warnings() {
        let ctx = RosterContext::with_deps(LazyFrame::default(), ETLDependancies::default());
        let actions: Vec<Arc<dyn OnboardingAction>> = vec![
            Arc::new(WarnThenSucceed),
            Arc::new(NoopAction),
        ];
        let result = ActionFactory::new()
            .run(actions, ctx)
            .expect("should succeed");

        let warnings = result.deps.logger.drain_deferred_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].message, "test warning");
    }
}
