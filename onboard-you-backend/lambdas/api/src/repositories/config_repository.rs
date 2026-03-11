//! Config repository — trait + PostgreSQL implementation for PipelineConfig persistence.

use async_trait::async_trait;
use sqlx::PgPool;

use crate::models::ApiError;

use onboard_you_models::{PipelineConfig, PipelineConfigRow};
/// Abstract persistence for pipeline configurations.
#[async_trait]
pub trait ConfigRepo: Send + Sync + 'static {
    async fn put(&self, config: &PipelineConfig) -> Result<(), ApiError>;
    async fn get(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<Option<PipelineConfig>, ApiError>;
    async fn list(&self, organization_id: &str) -> Result<Vec<PipelineConfig>, ApiError>;
    async fn delete(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<(), ApiError>;
}

/// PostgreSQL-backed implementation.
pub struct PgConfigRepo {
    pub pool: PgPool,
}

#[async_trait]
impl ConfigRepo for PgConfigRepo {
    async fn put(&self, config: &PipelineConfig) -> Result<(), ApiError> {
        sqlx::query(
            r#"INSERT INTO pipeline_configs
               (organization_id, customer_company_id, name, image, cron, last_edited, pipeline)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (organization_id, customer_company_id)
               DO UPDATE SET name = $3, image = $4, cron = $5, last_edited = $6, pipeline = $7"#,
        )
        .persistent(false)
        .bind(&config.organization_id)
        .bind(&config.customer_company_id)
        .bind(&config.name)
        .bind(&config.image)
        .bind(&config.cron)
        .bind(&config.last_edited)
        .bind(sqlx::types::Json(&config.pipeline))
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::Repository(format!("insert/update failed: {e}")))?;

        tracing::info!(
            organization_id = %config.organization_id,
            customer_company_id = %config.customer_company_id,
            "Config persisted"
        );
        Ok(())
    }

    async fn get(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<Option<PipelineConfig>, ApiError> {
        let row = sqlx::query_as::<_, PipelineConfigRow>(
            "SELECT * FROM pipeline_configs WHERE organization_id = $1 AND customer_company_id = $2",
        )
        .persistent(false)
        .bind(organization_id)
        .bind(customer_company_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::Repository(format!("query failed: {e}")))?;

        Ok(row.map(PipelineConfig::from))
    }

    async fn delete(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<(), ApiError> {
        sqlx::query(
            "DELETE FROM pipeline_configs WHERE organization_id = $1 AND customer_company_id = $2",
        )
        .persistent(false)
        .bind(organization_id)
        .bind(customer_company_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::Repository(format!("delete failed: {e}")))?;

        tracing::info!(
            organization_id = %organization_id,
            customer_company_id = %customer_company_id,
            "Config deleted"
        );
        Ok(())
    }

    async fn list(&self, organization_id: &str) -> Result<Vec<PipelineConfig>, ApiError> {
        let rows = sqlx::query_as::<_, PipelineConfigRow>(
            "SELECT * FROM pipeline_configs WHERE organization_id = $1",
        )
        .persistent(false)
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError::Repository(format!("query failed: {e}")))?;

        Ok(rows.into_iter().map(PipelineConfig::from).collect())
    }
}
