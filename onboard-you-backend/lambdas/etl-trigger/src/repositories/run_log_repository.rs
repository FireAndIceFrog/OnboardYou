//! Run log repository — persists pipeline run history to PostgreSQL.

use std::sync::Arc;

use async_trait::async_trait;
use lambda_runtime::Error;
use sqlx::PgPool;

use onboard_you_models::{PipelineRun, PipelineWarning, ValidationResult};

/// Repository trait for pipeline run persistence.
#[async_trait]
pub trait IRunLogRepo: Send + Sync {
    /// Insert a new run record (status = "running").
    async fn create_run(&self, run: &PipelineRun) -> Result<(), Error>;

    /// Mark a run as succeeded.
    async fn complete_run(
        &self,
        run_id: &str,
        rows_processed: Option<i32>,
        warnings: &[PipelineWarning],
    ) -> Result<(), Error>;

    /// Mark a run as failed, recording the error context.
    async fn fail_run(
        &self,
        run_id: &str,
        error_message: &str,
        error_action_id: Option<&str>,
        error_row: Option<i32>,
        warnings: &[PipelineWarning],
    ) -> Result<(), Error>;

    /// Mark a run as validation_failed (pre-execution schema check).
    async fn fail_validation(
        &self,
        run_id: &str,
        error_message: &str,
    ) -> Result<(), Error>;

    /// Store the validation result snapshot.
    async fn store_validation_result(
        &self,
        run_id: &str,
        validation_result: &ValidationResult,
    ) -> Result<(), Error>;
}

/// PostgreSQL-backed implementation.
pub struct PgRunLogRepo {
    pool: PgPool,
}

impl PgRunLogRepo {
    pub fn new(pool: PgPool) -> Arc<Self> {
        Arc::new(Self { pool })
    }
}

#[async_trait]
impl IRunLogRepo for PgRunLogRepo {
    async fn create_run(&self, run: &PipelineRun) -> Result<(), Error> {
        let manifest_json = run.manifest_snapshot.as_ref()
            .and_then(|m| serde_json::to_value(m).ok());
        sqlx::query(
            r#"INSERT INTO pipeline_runs (id, organization_id, customer_company_id, status, started_at, warnings, manifest_snapshot)
               VALUES ($1, $2, $3, $4, NOW(), '[]'::jsonb, $5)"#,
        )
        .persistent(false)
        .bind(&run.id)
        .bind(&run.organization_id)
        .bind(&run.customer_company_id)
        .bind(&run.status)
        .bind(manifest_json)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::from(format!("create_run failed: {e}")))?;
        Ok(())
    }

    async fn complete_run(
        &self,
        run_id: &str,
        rows_processed: Option<i32>,
        warnings: &[PipelineWarning],
    ) -> Result<(), Error> {
        let warnings_json = serde_json::to_value(warnings)
            .unwrap_or_else(|_| serde_json::json!([]));
        sqlx::query(
            r#"UPDATE pipeline_runs
               SET status = 'success', finished_at = NOW(),
                   rows_processed = $2, warnings = $3, current_action = NULL
               WHERE id = $1"#,
        )
        .persistent(false)
        .bind(run_id)
        .bind(rows_processed)
        .bind(warnings_json)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::from(format!("complete_run failed: {e}")))?;
        Ok(())
    }

    async fn fail_run(
        &self,
        run_id: &str,
        error_message: &str,
        error_action_id: Option<&str>,
        error_row: Option<i32>,
        warnings: &[PipelineWarning],
    ) -> Result<(), Error> {
        let warnings_json = serde_json::to_value(warnings)
            .unwrap_or_else(|_| serde_json::json!([]));
        sqlx::query(
            r#"UPDATE pipeline_runs
               SET status = 'failed', finished_at = NOW(),
                   error_message = $2, error_action_id = $3,
                   error_row = $4, warnings = $5, current_action = NULL
               WHERE id = $1"#,
        )
        .persistent(false)
        .bind(run_id)
        .bind(error_message)
        .bind(error_action_id)
        .bind(error_row)
        .bind(warnings_json)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::from(format!("fail_run failed: {e}")))?;
        Ok(())
    }

    async fn fail_validation(
        &self,
        run_id: &str,
        error_message: &str,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"UPDATE pipeline_runs
               SET status = 'validation_failed', finished_at = NOW(),
                   error_message = $2, current_action = NULL
               WHERE id = $1"#,
        )
        .persistent(false)
        .bind(run_id)
        .bind(error_message)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::from(format!("fail_validation failed: {e}")))?;
        Ok(())
    }

    async fn store_validation_result(
        &self,
        run_id: &str,
        validation_result: &ValidationResult,
    ) -> Result<(), Error> {
        let json = serde_json::to_value(validation_result)
            .map_err(|e| Error::from(format!("serialize validation_result: {e}")))?;
        sqlx::query(
            r#"UPDATE pipeline_runs SET validation_result = $2 WHERE id = $1"#,
        )
        .persistent(false)
        .bind(run_id)
        .bind(json)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::from(format!("store_validation_result failed: {e}")))?;
        Ok(())
    }
}
