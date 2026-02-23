//! Config repository — reads PipelineConfig from DynamoDB using serde_dynamo.

use async_trait::async_trait;
use lambda_runtime::Error;
use std::sync::Arc;

use onboard_you::{ActionConfigPayload, Manifest};

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
