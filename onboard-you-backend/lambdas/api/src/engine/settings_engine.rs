//! Settings engine — business logic for organization settings.
//!
//! Validates inputs, stamps server-controlled fields,
//! then delegates to the settings repository for persistence.
//!
//! The `default_auth` field is validated by attempting to construct
//! an `ApiEngine` from it — the same code path used at pipeline
//! execution time. If the config is invalid, the save is rejected.

use crate::dependancies::Dependancies;
use crate::models::ApiError;
use onboard_you::capabilities::egress::engine::api_engine::ApiEngine;
use onboard_you_models::OrgSettings;

/// Fetch settings for an organization. Returns `NotFound` if no settings exist.
pub async fn get(deps: &Dependancies, organization_id: &str) -> Result<OrgSettings, ApiError> {
    deps
        .settings_repo
        .get(organization_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(organization_id.to_string()))
}

/// Validate and persist organization settings.
///
/// Validates that `default_auth` can be parsed by `ApiEngine::from_action_config`
/// before persisting — prevents storing broken configs.
pub async fn upsert(
    deps: &Dependancies,
    organization_id: &str,
    mut settings: OrgSettings,
) -> Result<OrgSettings, ApiError> {
    // Server-controlled field — always use the JWT-derived org id
    settings.organization_id = organization_id.to_string();

    validate(&settings)?;

    deps.settings_repo.put(&settings).await?;

    tracing::info!(
        organization_id = %settings.organization_id,
        "Settings saved"
    );

    Ok(settings)
}

/// Validate that `default_auth` contains a well-formed auth config
/// by running it through the same factory the pipeline uses.
///
/// Also rejects `auth_type: "default"` — the org's own settings must
/// use a concrete auth type (bearer, oauth, oauth2).
fn validate(settings: &OrgSettings) -> Result<(), ApiError> {
    // Reject self-referential "default"
    if settings.default_auth.is_default() {
        return Err(ApiError::Validation(
            "default_auth cannot use auth_type 'default' — must be a concrete type \
             (bearer, oauth, oauth2)"
                .into(),
        ));
    }

    ApiEngine::from_action_config(&settings.default_auth)
        .map_err(|e| ApiError::Validation(format!("Invalid default_auth config: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependancies::{Dependancies, Env};
    use crate::repositories::schedule_repository::ScheduleRepo;
    use crate::repositories::settings_repository::SettingsRepo;
    use async_trait::async_trait;
    use onboard_you_models::{ApiDispatcherConfig, PipelineConfig};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    
    #[derive(Default)]
    struct NoOpScheduleRepo ;

    #[async_trait]
    impl ScheduleRepo for NoOpScheduleRepo {
        async fn upsert_schedule(&self, _config: &PipelineConfig) -> Result<(), ApiError> {
            Ok(())
        }
        async fn delete_schedule(&self, _org: &str, _company: &str) -> Result<(), ApiError> {
            Ok(())
        }
    }

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

    async fn test_state() -> (Dependancies, Arc<NoOpScheduleRepo>) {
        let schedule_repository = Arc::new(NoOpScheduleRepo);
        let mut deps = Dependancies::new(Env::default()).await;
        deps.settings_repo = Arc::new(InMemorySettingsRepo::default());
        deps.schedule_repo = schedule_repository.clone();
        (deps, schedule_repository)
    }

    fn bearer_config() -> ApiDispatcherConfig {
        let json = serde_json::json!({
            "auth_type": "bearer",
            "destination_url": "https://api.example.com/employees",
            "token": "sk-live-abc123",
            "schema": { "id": "string" },
            "body_path": "id"
        });
        serde_json::from_value(json).unwrap()
    }

    #[tokio::test]
    async fn upsert_persists_and_stamps_org_id() {
        let (state, _) = test_state().await;

        let settings = OrgSettings {
            organization_id: String::new(),
            default_auth: bearer_config(),
        };

        let out = super::upsert(&state, "org-1", settings).await.unwrap();
        assert_eq!(out.organization_id, "org-1");

        let fetched = super::get(&state, "org-1").await.unwrap();
        assert_eq!(fetched.organization_id, "org-1");
        // the helper config injects schema/body_path so they should round-trip
        if let onboard_you_models::ApiDispatcherConfig::Bearer(b) = &fetched.default_auth {
            assert_eq!(b.schema.get("id"), Some(&"string".to_string()));
            assert_eq!(b.body_path.as_deref(), Some("id"));
        } else {
            panic!("expected bearer config");
        }
    }

    #[tokio::test]
    async fn upsert_rejects_default_variant() {
        let (state, _) = test_state().await;

        let cfg = serde_json::json!({ "auth_type": "default" });
        let settings = OrgSettings {
            organization_id: String::new(),
            default_auth: serde_json::from_value(cfg).unwrap()
        };

        let err = super::upsert(&state, "org-1", settings).await.unwrap_err();
        assert!(matches!(err, ApiError::Validation(_)));
    }

    #[tokio::test]
    async fn get_not_found_returns_notfound() {
        let (state, _) = test_state().await;
        let err = super::get(&state, "missing").await.unwrap_err();
        assert!(matches!(err, ApiError::NotFound(_)));
    }
}
