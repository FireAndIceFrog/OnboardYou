//! Settings repository — reads OrgSettings from PostgreSQL for the ETL trigger.
//!
//! Used to resolve `auth_type: "default"` before pipeline construction.

use std::sync::Arc;

use async_trait::async_trait;
use lambda_runtime::Error;
use sqlx::PgPool;

use onboard_you_models::{OrgSettings, OrgSettingsRow};

/// Repository trait used by the pipeline engine to fetch org settings.
#[async_trait]
pub trait ISettingsRepo: Send + Sync {
    async fn get(&self, organization_id: &str) -> Result<Option<OrgSettings>, Error>;
}

/// PostgreSQL-backed implementation of `ISettingsRepo`.
pub struct PgSettingsRepo {
    pub pool: PgPool,
}

impl PgSettingsRepo {
    pub fn new(pool: PgPool) -> Arc<Self> {
        Arc::new(Self { pool })
    }
}

#[async_trait]
impl ISettingsRepo for PgSettingsRepo {
    async fn get(&self, organization_id: &str) -> Result<Option<OrgSettings>, Error> {
        let row = sqlx::query_as::<_, OrgSettingsRow>(
            "SELECT id, default_auth FROM organisation WHERE id = $1",
        )
        .persistent(false)
        .bind(organization_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::from(format!("query (settings) failed: {e}")))?;

        Ok(row.map(OrgSettings::from))
    }
}
