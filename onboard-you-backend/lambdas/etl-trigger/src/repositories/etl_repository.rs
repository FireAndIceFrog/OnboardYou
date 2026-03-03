//! Config repository — reads PipelineConfig from DynamoDB using serde_dynamo.

use async_trait::async_trait;
use lambda_runtime::Error;
use std::sync::Arc;

use onboard_you_models::{ActionConfigPayload, Manifest};

use crate::dependancies::Dependancies;

/// Repository trait used by the pipeline engine to fetch pipeline configs.
#[async_trait]
pub trait IEtlRepo: Send + Sync {
    async fn resolve_default_auth(
        &self,
        deps: &Dependancies,
        manifest: &mut Manifest,
        organization_id: &str,
    ) -> Result<Manifest, Error>;

    fn resolve_csv_s3_keys(
        &self,
        manifest: &mut Manifest,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<Manifest, Error>;
}

/// Dynamo-backed implementation of `IEtlRepo`.
pub struct EtlRepository {}

impl EtlRepository {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }
}

#[async_trait]
impl IEtlRepo for EtlRepository {
    /// Scan the manifest for actions with `ApiDispatcher(Default)` and replace
    /// their config with the organisation's stored default auth settings.
    ///
    /// The settings lookup is lazy — only performed if at least one action
    /// actually uses `"default"`.
    async fn resolve_default_auth(
        &self,
        deps: &Dependancies,
        manifest: &mut Manifest,
        organization_id: &str,
    ) -> Result<Manifest, Error> {
        let mut manifest = manifest.clone();
        // Quick scan: does any action use "default"?
        let needs_resolution = manifest.actions.iter().any(|ac| {
            matches!(
                ac.config,
                ActionConfigPayload::ApiDispatcher(ref cfg) if cfg.is_default()
            )
        });

        if !needs_resolution {
            return Ok(manifest);
        }

        // Fetch org settings via injected repo
        let settings = deps
            .settings_repo
            .get(organization_id)
            .await?
            .ok_or_else(|| {
                Error::from(format!(
                    "auth_type 'default' used but no settings found for org: {organization_id}"
                ))
            })?;

        tracing::info!(%organization_id, "Resolved default auth from settings table");

        // Replace config for every action that uses "default"
        for action in &mut manifest.actions {
            let is_default = matches!(
                action.config,
                ActionConfigPayload::ApiDispatcher(ref cfg) if cfg.is_default()
            );

            if is_default {
                tracing::info!(
                    action_id = %action.id,
                    action_type = %action.action_type,
                    "Replacing auth_type 'default' with org settings"
                );
                action.config = ActionConfigPayload::ApiDispatcher(settings.default_auth.clone());
            }
        }

        Ok(manifest)
    }

    /// Inject the resolved S3 key (`{org_id}/{company_id}/{filename}`) into every
    /// `CsvHrisConnector` action in the manifest.
    ///
    /// This must run **before** the factory builds the actions so that
    /// `download_from_s3` can find the correct S3 object at runtime.
    fn resolve_csv_s3_keys(
        &self,
        manifest: &mut Manifest,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<Manifest, Error> {
        for action in &mut manifest.actions {
            if let ActionConfigPayload::CsvHrisConnector(ref mut cfg) = action.config {
                cfg.resolve_s3_key(organization_id, customer_company_id);
                tracing::info!(
                    action_id = %action.id,
                    filename = %cfg.filename,
                    s3_key = ?cfg.resolved_s3_key,
                    "Resolved CSV S3 key"
                );
            }
        }
        Ok(manifest.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependancies::{Dependancies, Env};
    use crate::repositories::settings_repository::ISettingsRepo;
    use async_trait::async_trait;
    use lambda_runtime::Error;
    use std::sync::Arc;

    struct FakeSettingsRepo;

    #[async_trait]
    impl ISettingsRepo for FakeSettingsRepo {
        async fn get(
            &self,
            _organization_id: &str,
        ) -> Result<Option<onboard_you_models::OrgSettings>, Error> {
            let bearer = onboard_you_models::ApiDispatcherConfig::Bearer(onboard_you_models::BearerRepoConfig {
                destination_url: "https://example.com".into(),
                token: Some("tkn".into()),
                placement: onboard_you_models::BearerPlacement::AuthorizationHeader,
                placement_key: None,
                extra_headers: std::collections::HashMap::new(),
                schema: std::collections::HashMap::new(),
                body_path: None,
            });

            Ok(Some(onboard_you_models::OrgSettings {
                organization_id: _organization_id.into(),
                default_auth: bearer
            }))
        }
    }

    #[tokio::test]
    async fn test_resolve_default_auth_replaces_default() {
        let etl_repo = EtlRepository::new();

        // Build a manifest with one ApiDispatcher(Default) action
        let manifest = onboard_you_models::Manifest {
            version: "1.0".into(),
            actions: vec![onboard_you_models::ActionConfig {
                id: "egress".into(),
                action_type: onboard_you_models::ActionType::ApiDispatcher,
                config: onboard_you_models::ActionConfigPayload::ApiDispatcher(
                    onboard_you_models::ApiDispatcherConfig::Default,
                ),
                disabled: false,
            }],
        };
        let mut manifest_mut = manifest.clone();
        let mut deps = Dependancies::new(Arc::new(Env::default())).await;

        deps.settings_repo = Arc::new(FakeSettingsRepo);

        let resolved = etl_repo
            .resolve_default_auth(&deps, &mut manifest_mut, "org-1")
            .await
            .expect("resolve");

        assert_eq!(resolved.actions.len(), 1);
        match &resolved.actions[0].config {
            onboard_you_models::ActionConfigPayload::ApiDispatcher(cfg) => {
                assert!(
                    !cfg.is_default(),
                    "expected default to be replaced with org settings"
                );
            }
            _ => panic!("unexpected payload variant"),
        }
    }

    #[tokio::test]
    async fn test_resolve_csv_s3_keys_replaces_path() {
        let etl_repo = EtlRepository::new();

        // Build a manifest with one CsvHrisConnector action
        let mut manifest = onboard_you_models::Manifest {
            version: "1.0".into(),
            actions: vec![onboard_you_models::ActionConfig {
                id: "csv".into(),
                action_type: onboard_you_models::ActionType::CsvHrisConnector,
                config: onboard_you_models::ActionConfigPayload::CsvHrisConnector(
                    onboard_you_models::CsvHrisConnectorConfig {
                        filename: "data.csv".into(),
                        resolved_s3_key: None,
                        columns: vec![],
                    },
                ),
                disabled: false,
            }],
        };

        let resolved = etl_repo
            .resolve_csv_s3_keys(&mut manifest, "org-1", "comp-1")
            .expect("resolve");

        assert_eq!(resolved.actions.len(), 1);
        match &resolved.actions[0].config {
            onboard_you_models::ActionConfigPayload::CsvHrisConnector(cfg) => {
                assert_eq!(
                    cfg.resolved_s3_key.as_deref(),
                    Some("org-1/comp-1/data.csv"),
                    "expected S3 key to be resolved with org and company ID"
                );
            }
            _ => panic!("unexpected payload variant"),
        }
    }
}
