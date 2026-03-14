//! Run history repository — reads pipeline run logs from PostgreSQL.

use async_trait::async_trait;
use sqlx::PgPool;

use crate::models::ApiError;
use onboard_you_models::{PipelineRun, PipelineRunRow};

/// Repository trait for querying pipeline run history.
#[async_trait]
pub trait RunHistoryRepo: Send + Sync + std::fmt::Debug {
    /// Count total runs for a given org + customer company.
    async fn count_runs(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<i64, ApiError>;

    /// List runs for a given org + customer company, most recent first.
    async fn list_runs(
        &self,
        organization_id: &str,
        customer_company_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PipelineRun>, ApiError>;

    /// Get a single run by ID.
    async fn get_run(
        &self,
        organization_id: &str,
        run_id: &str,
    ) -> Result<Option<PipelineRun>, ApiError>;

    /// Return true if there is a run with status 'running' started within the
    /// last 12 hours for the given org + customer company. Used to enforce
    /// idempotency on the trigger endpoint.
    async fn has_active_run(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<bool, ApiError>;
}

#[derive(Debug)]
pub struct PgRunHistoryRepo {
    pub pool: PgPool,
}

#[async_trait]
impl RunHistoryRepo for PgRunHistoryRepo {
    async fn count_runs(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<i64, ApiError> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM pipeline_runs WHERE organization_id = $1 AND customer_company_id = $2",
        )
        .persistent(false)
        .bind(organization_id)
        .bind(customer_company_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::Repository(format!("count_runs query failed: {e}")))?;
        Ok(count)
    }

    async fn list_runs(
        &self,
        organization_id: &str,
        customer_company_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PipelineRun>, ApiError> {
        let rows = sqlx::query_as::<_, PipelineRunRow>(
            r#"SELECT id, organization_id, customer_company_id, status,
                      started_at::text, finished_at::text,
                      rows_processed, current_action,
                      error_message, error_action_id, error_row,
                      warnings, validation_result, manifest_snapshot
               FROM pipeline_runs
               WHERE organization_id = $1 AND customer_company_id = $2
               ORDER BY started_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .persistent(false)
        .bind(organization_id)
        .bind(customer_company_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError::Repository(format!("list_runs query failed: {e}")))?;

        Ok(rows.into_iter().map(PipelineRun::from).collect())
    }

    async fn get_run(
        &self,
        organization_id: &str,
        run_id: &str,
    ) -> Result<Option<PipelineRun>, ApiError> {
        let row = sqlx::query_as::<_, PipelineRunRow>(
            r#"SELECT id, organization_id, customer_company_id, status,
                      started_at::text, finished_at::text,
                      rows_processed, current_action,
                      error_message, error_action_id, error_row,
                      warnings, validation_result, manifest_snapshot
               FROM pipeline_runs
               WHERE organization_id = $1 AND id = $2"#,
        )
        .persistent(false)
        .bind(organization_id)
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::Repository(format!("get_run query failed: {e}")))?;

        Ok(row.map(PipelineRun::from))
    }

    async fn has_active_run(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<bool, ApiError> {
        let (count,): (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*)
               FROM pipeline_runs
               WHERE organization_id = $1
                 AND customer_company_id = $2
                 AND status = 'running'
                 AND started_at > NOW() - INTERVAL '12 hours'"#,
        )
        .persistent(false)
        .bind(organization_id)
        .bind(customer_company_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::Repository(format!("has_active_run query failed: {e}")))?;
        Ok(count > 0)
    }
}
