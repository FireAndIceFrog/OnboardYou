//! Validation engine — dry-run column propagation using `CalculateColumns`.
//!
//! Parses the manifest, builds every action via the factory, and folds
//! `calculate_columns` through the pipeline without executing any real
//! transformations or touching external data sources.

use crate::{
    dependancies::Dependancies,
    models::{ApiError, ValidationResult},
};
use onboard_you::{ActionFactoryTrait};
use onboard_you_models::{ActionConfigPayload, ApiDispatcherConfig, DynamicEgressModel, Manifest};

/// Validate a pipeline manifest by propagating columns through every step.
///
/// Returns the column set at each step, or an `ApiError` on the first failure.
pub async fn validate_pipeline(
    deps: &Dependancies,
    pipeline_json: &Manifest,
    organization_id: Option<String>
)  -> Result<ValidationResult, ApiError> {
    let manifest: Manifest = pipeline_json.clone();

    let action_factory = deps.etl_repo.create_action_factory();
    let validation_result = action_factory
        .validate_manifest(&manifest)
        .map_err(|e| {
            let msg = e.error.to_string();
            let short = msg.split(';').next().unwrap_or(&msg);
            ApiError::Validation(format!(
                "Step '{}' ({}): {short}",
                e.action_id, e.action_type
            ))
        })?;
    let final_columns = validation_result.final_columns.clone();

    if let Some(org_id) = organization_id {
        let last_action = match manifest.actions.last() {
            Some(action) => action,
            None => return Err(ApiError::Validation("No actions in manifest".into())),
        };
        let egress_config: Box<dyn DynamicEgressModel> = match last_action.config.clone() {
            ActionConfigPayload::ApiDispatcher(cfg) => {
                let pre_settings = match cfg {
                    ApiDispatcherConfig::Default => {
                        let settings = deps.settings_repo.get(&org_id).await?;

                        if settings.is_none(){
                            return Err(ApiError::Validation(
                                "No API dispatcher settings configured — go to Settings to set up a default auth method".into()
                            ))
                        }

                        let settings = settings.unwrap();
                        match settings.default_auth {

                            ApiDispatcherConfig::Default => return Err(ApiError::Validation(
                                "Default auth in Settings is not configured — choose Bearer, OAuth, or OAuth2 in Settings".into()
                            )),
                            cfg => cfg 
                        }
                        
                    },
                    cfg => cfg
                };

                match pre_settings {
                    ApiDispatcherConfig::Default => 
                        return Err(ApiError::Validation(
                            "No API dispatcher settings configured — go to Settings to set up a default auth method".into()
                        )),
                    ApiDispatcherConfig::Bearer(curr) => Ok(Box::new(curr.clone()) as Box<dyn DynamicEgressModel>),
                    ApiDispatcherConfig::OAuth(curr) => Ok(Box::new(curr.clone()) as Box<dyn DynamicEgressModel>),
                    ApiDispatcherConfig::OAuth2(curr) => Ok(Box::new(curr.clone()) as Box<dyn DynamicEgressModel>)
                }
            },
            _ => return Err(ApiError::Validation(format!(
                "Last step must be an API Dispatcher, found '{}'",
                last_action.action_type
            ))),
        }?;

        let egress_schema = egress_config.get_schema();
        let missing: Vec<String> = egress_schema.keys()
            .filter(|k| !final_columns.contains(k))
            .cloned()
            .collect();

        if !missing.is_empty() {
            return Err(ApiError::Validation(format!(
                "API Dispatcher requires columns not produced by the pipeline: {}",
                missing.join(", ")
            )))
        }

    }

    Ok(ValidationResult {
        steps: validation_result.steps,
        final_columns,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependancies::{Dependancies, Env};
    use crate::repositories::settings_repository::SettingsRepo;
    use async_trait::async_trait;
    use onboard_you_models::{ApiDispatcherConfig, OrgSettings};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[derive(Default)]
    struct InMemorySettingsRepo {
        store: RwLock<Option<OrgSettings>>,
    }

    #[async_trait]
    impl SettingsRepo for InMemorySettingsRepo {
        async fn put(&self, settings: &OrgSettings) -> Result<(), ApiError> {
            self.store.write().await.replace(settings.clone());
            Ok(())
        }

        async fn get(&self, _organization_id: &str) -> Result<Option<OrgSettings>, ApiError> {
            let guard = self.store.read().await;
            Ok(guard.clone())
        }
    }

    async fn test_state_with_settings(settings: Option<OrgSettings>) -> Dependancies {
        let repo = InMemorySettingsRepo::default();
        if let Some(s) = settings {
            repo.store.write().await.replace(s);
        }
        let mut deps = Dependancies::new(Env::default()).await;
        deps.settings_repo = Arc::new(repo);
        deps
    }

    fn bearer_config_with_schema(schema: std::collections::HashMap<String, String>) -> ApiDispatcherConfig {
        let schema_json: serde_json::Value = schema.iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into();

        let json = serde_json::json!({
            "auth_type": "bearer",
            "destination_url": "https://api.example.com/employees",
            "token": "sk-live-abc123",
            "schema": schema_json,
            "body_path": "id"
        });
        serde_json::from_value(json).unwrap()
    }

    /// Helper: generic ingest + api_dispatcher manifest JSON
    fn pipeline_with_dispatcher(columns: &[&str], dispatcher_config: serde_json::Value) -> String {
        let cols: Vec<String> = columns.iter().map(|c| format!("\"{}\"", c)).collect();
        format!(
            r#"{{
                "version": "1.0",
                "actions": [
                    {{ "id": "ingest", "action_type": "generic_ingestion_connector", "config": {{ "filename": "data.csv", "columns": [{}] }} }},
                    {{ "id": "dispatch", "action_type": "api_dispatcher", "config": {} }}
                ]
            }}"#,
            cols.join(","),
            dispatcher_config
        )
    }

    /// Describes one validation scenario.
    struct Case {
        name: &'static str,
        /// Raw manifest JSON. If `None`, built from `columns` + `dispatcher`.
        manifest_json: Option<&'static str>,
        /// Pipeline input columns (fed to generic_ingestion_connector). Ignored when `manifest_json` is set.
        columns: &'static [&'static str],
        /// Dispatcher config JSON — `None` means CSV-only pipeline.
        dispatcher: Option<serde_json::Value>,
        /// Organization id passed to `validate_pipeline`.
        org_id: Option<&'static str>,
        /// Pre-seeded OrgSettings for the in-memory repo.
        settings: Option<OrgSettings>,
    }

    impl Default for Case {
        fn default() -> Self {
            Self {
                name: "",
                manifest_json: None,
                columns: &[],
                dispatcher: None,
                org_id: None,
                settings: None,
            }
        }
    }

    fn all_cases() -> Vec<Case> {
        let bearer = |keys: &[&str]| serde_json::json!({
            "auth_type": "bearer",
            "destination_url": "https://api.example.com/employees",
            "token": "tok",
            "schema": keys.iter().map(|k| (k.to_string(), serde_json::Value::String("string".into()))).collect::<serde_json::Map<_,_>>(),
            "body_path": keys.first().unwrap_or(&"id")
        });

        let bearer_settings = |keys: &[&str]| -> OrgSettings {
            let schema: std::collections::HashMap<String, String> =
                keys.iter().map(|k| (k.to_string(), "string".into())).collect();
            OrgSettings {
                organization_id: "org-1".into(),
                default_auth: bearer_config_with_schema(schema),
            }
        };

        vec![
            // ── Basic pipeline validation ──
            Case {
                name: "empty_manifest",
                manifest_json: Some(r#"{ "version": "1.0", "actions": [] }"#),
                ..Default::default()
            },
            Case {
                name: "csv_column_propagation",
                columns: &["a", "b"],
                ..Default::default()
            },
            Case {
                name: "bad_action_config_empty_columns",
                manifest_json: Some(r#"{
                    "version": "1.0",
                    "actions": [
                        { "id": "ingest", "action_type": "generic_ingestion_connector", "config": { "filename": "data.csv", "columns": [] } }
                    ]
                }"#),
                ..Default::default()
            },
            // ── Egress validation ──
            Case {
                name: "last_action_not_dispatcher",
                columns: &["a"],
                org_id: Some("org-1"),
                ..Default::default()
            },
            Case {
                name: "default_config_no_settings",
                columns: &["a", "b"],
                dispatcher: Some(serde_json::json!({ "auth_type": "default" })),
                org_id: Some("org-1"),
                ..Default::default()
            },
            Case {
                name: "default_config_settings_also_default",
                columns: &["a", "b"],
                dispatcher: Some(serde_json::json!({ "auth_type": "default" })),
                org_id: Some("org-1"),
                settings: Some(OrgSettings {
                    organization_id: "org-1".into(),
                    default_auth: ApiDispatcherConfig::Default,
                }),
                ..Default::default()
            },
            Case {
                name: "default_resolved_from_settings_ok",
                columns: &["a", "b"],
                dispatcher: Some(serde_json::json!({ "auth_type": "default" })),
                org_id: Some("org-1"),
                settings: Some(bearer_settings(&["a", "b"])),
                ..Default::default()
            },
            Case {
                name: "bearer_schema_columns_match",
                columns: &["a", "b"],
                dispatcher: Some(bearer(&["a", "b"])),
                org_id: Some("org-1"),
                ..Default::default()
            },
            Case {
                name: "bearer_schema_column_missing",
                columns: &["a", "b"],
                dispatcher: Some(bearer(&["a", "missing_col"])),
                org_id: Some("org-1"),
                ..Default::default()
            },
            Case {
                name: "egress_skipped_when_no_org_id",
                columns: &["x"],
                ..Default::default()
            },
            Case {
                name: "default_resolved_to_bearer_missing_column",
                columns: &["a", "b"],
                dispatcher: Some(serde_json::json!({ "auth_type": "default" })),
                org_id: Some("org-1"),
                settings: Some(bearer_settings(&["a", "not_here"])),
                ..Default::default()
            },
        ]
    }

    fn build_manifest_json(case: &Case) -> String {
        if let Some(raw) = case.manifest_json {
            return raw.to_string();
        }
        match &case.dispatcher {
            Some(d) => pipeline_with_dispatcher(case.columns, d.clone()),
            None => {
                let cols: Vec<String> = case.columns.iter().map(|c| format!("\"{}\"", c)).collect();
                format!(
                    r#"{{ "version": "1.0", "actions": [
                        {{ "id": "ingest", "action_type": "generic_ingestion_connector", "config": {{ "filename": "data.csv", "columns": [{}] }} }}
                    ] }}"#,
                    cols.join(",")
                )
            }
        }
    }

    fn format_result(res: &Result<ValidationResult, ApiError>) -> String {
        match res {
            Ok(v) => serde_json::to_string_pretty(v).unwrap(),
            Err(ApiError::Validation(msg)) => format!("ERR Validation: {msg}"),
            Err(other) => format!("ERR: {other:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn validate_pipeline_cases() {
        for case in all_cases() {
            let manifest_json = build_manifest_json(&case);
            let state = test_state_with_settings(case.settings).await;
            let manifest = Manifest::from_json(&manifest_json).expect("parse manifest");
            let org_id = case.org_id.map(String::from);

            let result = validate_pipeline(&state, &manifest, org_id).await;
            let output = format_result(&result);

            insta::assert_snapshot!(case.name, output);
        }
    }
}
