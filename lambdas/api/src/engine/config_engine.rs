//! Config engine — business logic for pipeline configuration.
//!
//! Validates inputs, stamps server-controlled fields,
//! then delegates to repository trait objects for persistence and scheduling.

use crate::{dependancies::Dependancies, models::ApiError};

use onboard_you::PipelineConfig;

/// Fetch a pipeline config by organization ID and customer company ID.
pub async fn get(
    state: &Dependancies,
    organization_id: &str,
    customer_company_id: &str,
) -> Result<PipelineConfig, ApiError> {
    state
        .config_repo
        .get(organization_id, customer_company_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("{organization_id}/{customer_company_id}")))
}

/// List all pipeline configs owned by an organization.
pub async fn list(
    state: &Dependancies,
    organization_id: &str,
) -> Result<Vec<PipelineConfig>, ApiError> {
    state.config_repo.list(organization_id).await
}

/// Validate, persist, and schedule a pipeline config.
pub async fn upsert(
    state: &Dependancies,
    organization_id: &str,
    customer_company_id: &str,
    mut config: PipelineConfig,
) -> Result<PipelineConfig, ApiError> {
    // Server-controlled fields
    config.organization_id = organization_id.to_string();
    config.customer_company_id = customer_company_id.to_string();
    config.last_edited = chrono::Utc::now().to_rfc3339();

    validate(&config)?;

    state.config_repo.put(&config).await?;
    state.schedule_repo.upsert(&config).await?;

    tracing::info!(
        organization_id = %config.organization_id,
        customer_company_id = %config.customer_company_id,
        "Config saved and schedule updated"
    );

    Ok(config)
}

/// Delete a pipeline config and its associated schedule.
pub async fn delete(
    state: &Dependancies,
    organization_id: &str,
    customer_company_id: &str,
) -> Result<(), ApiError> {
    state
        .config_repo
        .delete(organization_id, customer_company_id)
        .await?;

    // Best-effort schedule cleanup — don't fail the delete if it's missing
    if let Err(e) = state
        .schedule_repo
        .delete(organization_id, customer_company_id)
        .await
    {
        tracing::warn!(
            organization_id = %organization_id,
            customer_company_id = %customer_company_id,
            error = ?e,
            "Failed to delete schedule (may not exist)"
        );
    }

    tracing::info!(
        organization_id = %organization_id,
        customer_company_id = %customer_company_id,
        "Config and schedule deleted"
    );

    Ok(())
}

fn validate(config: &PipelineConfig) -> Result<(), ApiError> {
    if config.cron.is_empty() {
        return Err(ApiError::Validation("cron field is required".into()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependancies::Env;
    use crate::repositories::config_repository::ConfigRepo;
    use crate::repositories::schedule_repository::ScheduleRepo;
    use async_trait::async_trait;
    use onboard_you::Manifest;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // ---- In-memory fakes ----

    #[derive(Default)]
    struct InMemoryConfigRepo {
        store: RwLock<HashMap<(String, String), PipelineConfig>>,
    }

    #[async_trait]
    impl ConfigRepo for InMemoryConfigRepo {
        async fn put(&self, config: &PipelineConfig) -> Result<(), ApiError> {
            let key = (
                config.organization_id.clone(),
                config.customer_company_id.clone(),
            );
            self.store.write().await.insert(key, config.clone());
            Ok(())
        }

        async fn get(&self, org: &str, company: &str) -> Result<Option<PipelineConfig>, ApiError> {
            let store = self.store.read().await;
            Ok(store.get(&(org.to_string(), company.to_string())).cloned())
        }

        async fn list(&self, org: &str) -> Result<Vec<PipelineConfig>, ApiError> {
            let store = self.store.read().await;
            Ok(store
                .values()
                .filter(|c| c.organization_id == org)
                .cloned()
                .collect())
        }

        async fn delete(&self, org: &str, company: &str) -> Result<(), ApiError> {
            self.store
                .write()
                .await
                .remove(&(org.to_string(), company.to_string()));
            Ok(())
        }
    }

    struct NoOpScheduleRepo;

    #[async_trait]
    impl ScheduleRepo for NoOpScheduleRepo {
        async fn upsert(&self, _config: &PipelineConfig) -> Result<(), ApiError> {
            Ok(())
        }
        async fn delete(&self, _org: &str, _company: &str) -> Result<(), ApiError> {
            Ok(())
        }
    }

    // ---- Helpers ----

    async fn test_state() -> Dependancies {
        let mut deps = Dependancies::new(Env::default()).await;
        deps.config_repo = Arc::new(InMemoryConfigRepo::default());
        deps.schedule_repo = Arc::new(NoOpScheduleRepo);

        deps
    }

    fn sample_config() -> PipelineConfig {
        PipelineConfig {
            name: "Test Pipeline".into(),
            image: None,
            cron: "rate(1 hour)".into(),
            organization_id: String::new(),
            customer_company_id: String::new(),
            last_edited: String::new(),
            pipeline: Manifest {
                version: "1.0".into(),
                actions: vec![],
            },
        }
    }

    // ---- Tests ----

    #[tokio::test]
    async fn upsert_stamps_server_fields() {
        let state = test_state().await;
        let result = upsert(&state, "org-1", "company-1", sample_config())
            .await
            .unwrap();

        assert_eq!(result.organization_id, "org-1");
        assert_eq!(result.customer_company_id, "company-1");
        assert!(!result.last_edited.is_empty());
    }

    #[tokio::test]
    async fn upsert_rejects_empty_cron() {
        let state = test_state().await;
        let mut cfg = sample_config();
        cfg.cron = String::new();

        let err = upsert(&state, "org-1", "company-1", cfg).await.unwrap_err();
        assert!(matches!(err, ApiError::Validation(_)));
    }

    #[tokio::test]
    async fn get_returns_upserted_config() {
        let state = test_state().await;
        upsert(&state, "org-1", "company-1", sample_config())
            .await
            .unwrap();

        let fetched = get(&state, "org-1", "company-1").await.unwrap();
        assert_eq!(fetched.name, "Test Pipeline");
        assert_eq!(fetched.organization_id, "org-1");
    }

    #[tokio::test]
    async fn get_not_found() {
        let state = test_state().await;
        let err = get(&state, "org-1", "missing").await.unwrap_err();
        assert!(matches!(err, ApiError::NotFound(_)));
    }

    #[tokio::test]
    async fn list_returns_matching_configs() {
        let state = test_state().await;

        upsert(&state, "org-1", "c1", sample_config())
            .await
            .unwrap();
        upsert(&state, "org-1", "c2", sample_config())
            .await
            .unwrap();
        upsert(&state, "org-2", "c3", sample_config())
            .await
            .unwrap();

        let org1 = list(&state, "org-1").await.unwrap();
        assert_eq!(org1.len(), 2);

        let org2 = list(&state, "org-2").await.unwrap();
        assert_eq!(org2.len(), 1);
    }

    #[tokio::test]
    async fn delete_removes_config() {
        let state = test_state().await;
        upsert(&state, "org-1", "company-1", sample_config())
            .await
            .unwrap();

        delete(&state, "org-1", "company-1").await.unwrap();

        let err = get(&state, "org-1", "company-1").await.unwrap_err();
        assert!(matches!(err, ApiError::NotFound(_)));
    }
}
