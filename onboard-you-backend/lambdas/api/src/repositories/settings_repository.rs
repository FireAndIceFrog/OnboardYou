//! Settings repository — PostgreSQL persistence for OrgSettings.
//!
//! Organisation settings (default_auth) are stored as a JSONB column on the
//! `organisation` table.

use async_trait::async_trait;
use sqlx::PgPool;

use crate::models::ApiError;
use onboard_you_models::{OrgSettings, OrgSettingsRow};

#[async_trait]
pub trait SettingsRepo: Send + Sync {
    async fn put(&self, settings: &OrgSettings) -> Result<(), ApiError>;
    async fn get(&self, organization_id: &str) -> Result<Option<OrgSettings>, ApiError>;
}

/// PostgreSQL-backed implementation.
pub struct PgSettingsRepo {
    pub pool: PgPool,
}

#[async_trait]
impl SettingsRepo for PgSettingsRepo {
    async fn put(&self, settings: &OrgSettings) -> Result<(), ApiError> {
        sqlx::query("UPDATE organisation SET default_auth = $1 WHERE id = $2")
            .persistent(false)
            .bind(sqlx::types::Json(&settings.default_auth))
            .bind(&settings.organization_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::Repository(format!("update failed: {e}")))?;

        tracing::info!(
            organization_id = %settings.organization_id,
            "Settings persisted"
        );
        Ok(())
    }

    async fn get(&self, organization_id: &str) -> Result<Option<OrgSettings>, ApiError> {
        let row = sqlx::query_as::<_, OrgSettingsRow>(
            "SELECT id, default_auth FROM organisation WHERE id = $1",
        )
        .persistent(false)
        .bind(organization_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::Repository(format!("query failed: {e}")))?;

        Ok(row.map(OrgSettings::from))
    }
}
