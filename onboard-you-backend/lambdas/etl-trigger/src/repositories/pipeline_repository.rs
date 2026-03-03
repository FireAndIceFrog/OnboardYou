//! Config repository — reads PipelineConfig from DynamoDB using serde_dynamo.

use async_trait::async_trait;
use lambda_runtime::Error;
use polars::prelude::LazyFrame;
use std::sync::Arc;

use onboard_you_models::{Manifest, RosterContext};

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
            .collect::<onboard_you_models::Result<_>>()
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

#[cfg(test)]
mod tests {
    use crate::dependancies::{Dependancies, Env};

    use super::*;
    use onboard_you_models::{
        ActionConfig, ActionConfigPayload, ActionType, OnboardingAction, RosterContext,
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    struct NoopAction;
    impl onboard_you_models::ColumnCalculator for NoopAction {
        fn calculate_columns(&self, ctx: RosterContext) -> onboard_you_models::Result<RosterContext> {
            Ok(ctx)
        }
    }
    impl OnboardingAction for NoopAction {
        fn id(&self) -> &str {
            "noop"
        }
        fn execute(&self, ctx: RosterContext) -> onboard_you_models::Result<RosterContext> {
            Ok(ctx)
        }
    }

    struct FakeFactory {
        create_count: Arc<AtomicUsize>,
        run_count: Arc<AtomicUsize>,
    }

    impl FakeFactory {
        fn new() -> Self {
            Self {
                create_count: Arc::new(AtomicUsize::new(0)),
                run_count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl onboard_you::ActionFactoryTrait for FakeFactory {
        fn create(
            &self,
            _action_config: &ActionConfig,
        ) -> onboard_you_models::Result<std::sync::Arc<dyn OnboardingAction>> {
            self.create_count.fetch_add(1, Ordering::SeqCst);
            Ok(std::sync::Arc::new(NoopAction))
        }

        fn run(
            &self,
            _actions: Vec<std::sync::Arc<dyn OnboardingAction>>,
            context: RosterContext,
        ) -> onboard_you_models::Result<RosterContext> {
            self.run_count.fetch_add(1, Ordering::SeqCst);
            Ok(context)
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_run_pipeline_uses_action_factory() {
        let factory = Arc::new(FakeFactory::new());
        let mut deps = Dependancies::new(Arc::new(Env::default())).await;
        deps.action_factory = factory.clone();

        let manifest = onboard_you_models::Manifest {
            version: "1.0".into(),
            actions: vec![ActionConfig {
                id: "a".into(),
                action_type: ActionType::ApiDispatcher,
                config: ActionConfigPayload::ApiDispatcher(
                    onboard_you_models::ApiDispatcherConfig::Default,
                ),
                disabled: false,
            }],
        };

        let repo = PipelineRepository::new();
        let res = repo
            .run_pipeline(&deps, manifest, "org", "cust")
            .await
            .expect("run pipeline");
        assert_eq!(res.status, "success");
        assert_eq!(factory.create_count.load(Ordering::SeqCst), 1);
        assert_eq!(factory.run_count.load(Ordering::SeqCst), 1);
    }
}
