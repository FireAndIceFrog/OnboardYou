//! Config repository — reads PipelineConfig from PostgreSQL.

use async_trait::async_trait;
use lambda_runtime::Error;
use std::sync::Arc;

use sqlx::PgPool;
use onboard_you_models::{PipelineConfig, PipelineConfigRow};

/// Repository trait used by the pipeline engine to fetch pipeline configs.
#[async_trait]
pub trait IConfigRepo: Send + Sync {
    async fn get(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<PipelineConfig, Error>;
}

/// PostgreSQL-backed implementation of `IConfigRepo`.
pub struct PgConfigRepo {
    pub pool: PgPool,
}

impl PgConfigRepo {
    pub fn new(pool: PgPool) -> Arc<Self> {
        Arc::new(Self { pool })
    }
}

#[async_trait]
impl IConfigRepo for PgConfigRepo {
    async fn get(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<PipelineConfig, Error> {
        let row = sqlx::query_as::<_, PipelineConfigRow>(
            "SELECT * FROM pipeline_configs WHERE organization_id = $1 AND customer_company_id = $2",
        )
        .bind(organization_id)
        .bind(customer_company_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::from(format!("query failed: {e}")))?;

        let row = row.ok_or_else(|| {
            Error::from(format!(
                "No config found for org: {organization_id}, customer: {customer_company_id}"
            ))
        })?;

        Ok(PipelineConfig::from(row))
    }
}
