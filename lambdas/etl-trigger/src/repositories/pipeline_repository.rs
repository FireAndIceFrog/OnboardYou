//! Config repository — reads PipelineConfig from DynamoDB using serde_dynamo.

use async_trait::async_trait;
use lambda_runtime::Error;
use polars::prelude::LazyFrame;
use std::sync::Arc;

use onboard_you::{
    Manifest,
    RosterContext,
};

use crate::{dependancies::Dependancies, models::PipelineResult};

/// Repository trait used by the pipeline engine to fetch pipeline configs.
#[async_trait]
pub trait IPipelineRepo: Send + Sync {
    async fn run_pipeline(
        &self,
        deps: &Dependancies, 
        manifest: Manifest,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<PipelineResult, Error>;
}

/// Dynamo-backed implementation of `IPipelineRepo`.
pub struct PipelineRepository {}

impl PipelineRepository {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }
}

#[async_trait]
impl IPipelineRepo for PipelineRepository {
    async fn run_pipeline(
        &self,
        deps: &Dependancies, 
        manifest: Manifest,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<PipelineResult, Error> {
        let action_factory = deps.action_factory.clone();
        // 4. Build actions from manifest via Factory
        let actions: Vec<_> = manifest
            .actions
            .iter()
            .map(|ac| action_factory.create(ac))
            .collect::<onboard_you::Result<_>>()
            .map_err(|e| Error::from(format!("Failed to build actions: {e}")))?;

        // 5. Execute the pipeline
        let context = RosterContext::new(LazyFrame::default());

        match action_factory.run(actions, context) {
            Ok(result) => {
                let rows = result
                    .data
                    .clone()
                    .collect()
                    .map(|df: polars::prelude::DataFrame| df.height())
                    .ok();
                tracing::info!(%organization_id, %customer_company_id, rows_processed = ?rows, "Pipeline completed");
                Ok(PipelineResult::success(
                    organization_id,
                    customer_company_id,
                    rows,
                ))
            }
            Err(e) => {
                tracing::error!(%organization_id, %customer_company_id, error = %e, "Pipeline failed");
                Ok(PipelineResult::failure(
                    organization_id,
                    customer_company_id,
                    e,
                ))
            }
        }
    }
}
