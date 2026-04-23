//! Pipeline repository — builds and executes the ETL pipeline, persisting
//! run history to the `pipeline_runs` table.

use async_trait::async_trait;
use lambda_runtime::Error;
use polars::prelude::LazyFrame;
use std::sync::Arc;

use onboard_you_models::{ActionConfigPayload, ETLDependancies, Manifest, RosterContext};

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
        run_id: &str,
    ) -> Result<PipelineResult, Error>;
}

/// PostgreSQL-backed implementation of `IPipelineRepo`.
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
        run_id: &str,
    ) -> Result<PipelineResult, Error> {
        let action_factory = deps.action_factory.clone();

        // Resolve the S3 key for any GenericIngestionConnector before building actions.
        // The key is derived from the filename and org/company IDs.
        let mut manifest = manifest;
        for ac in &mut manifest.actions {
            if let ActionConfigPayload::GenericIngestionConnector(ref mut cfg) = ac.config {
                cfg.resolve_s3_key(organization_id, customer_company_id);
            }
        }

        // Build actions from manifest via Factory
        let actions: Vec<_> = manifest
            .actions
            .iter()
            .map(|ac| action_factory.create(ac))
            .collect::<onboard_you_models::Result<_>>()
            .map_err(|e| Error::from(format!("Failed to build actions: {e}")))?;

        // Execute the pipeline with step tracking
        let context = RosterContext::with_deps(LazyFrame::default(), ETLDependancies::default());

        match action_factory.run(actions, context) {
            Ok(result) => {
                // Collect the LazyFrame to count rows and trigger deferred closures
                let rows = result
                    .get_data()
                    .collect()
                    .map(|df: polars::prelude::DataFrame| df.height())
                    .ok();

                // Drain any deferred warnings emitted by Polars .map() closures
                let warnings = result.deps.logger.drain_deferred_warnings();

                tracing::info!(
                    %organization_id, %customer_company_id,
                    rows_processed = ?rows, "Pipeline completed"
                );

                // Persist success
                let _ = deps
                    .run_log_repo
                    .complete_run(
                        run_id,
                        rows.map(|r| r as i32),
                        &warnings,
                    )
                    .await;

                Ok(PipelineResult::success(
                    run_id,
                    organization_id,
                    customer_company_id,
                    rows,
                    warnings,
                ))
            }
            Err(step_err) => {
                tracing::error!(
                    %organization_id, %customer_company_id,
                    action_id = %step_err.action_id,
                    error = %step_err.error,
                    "Pipeline failed"
                );

                let warnings = step_err.warnings.clone();

                // Persist failure
                let _ = deps
                    .run_log_repo
                    .fail_run(
                        run_id,
                        &step_err.error.to_string(),
                        Some(&step_err.action_id),
                        None,
                        &warnings,
                    )
                    .await;

                Ok(PipelineResult::failure(
                    run_id,
                    organization_id,
                    customer_company_id,
                    step_err,
                    warnings,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dependancies::{Dependancies, Env};
    use crate::repositories::run_log_repository::IRunLogRepo;

    use super::*;
    use onboard_you_models::{
        ActionConfig, ActionConfigPayload, ActionType, OnboardingAction, PipelineRun, PipelineWarning,
        RosterContext, ValidationResult,
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

        fn validate_manifest(
            &self,
            _manifest: &onboard_you_models::Manifest,
        ) -> std::result::Result<onboard_you_models::ValidationResult, onboard_you_models::ValidationStepError> {
            Ok(onboard_you_models::ValidationResult {
                steps: vec![],
                final_columns: vec![],
            })
        }

        fn run(
            &self,
            _actions: Vec<std::sync::Arc<dyn OnboardingAction>>,
            context: RosterContext,
        ) -> std::result::Result<RosterContext, onboard_you::StepError> {
            self.run_count.fetch_add(1, Ordering::SeqCst);
            Ok(context)
        }
    }

    struct NoopRunLogRepo;

    #[async_trait::async_trait]
    impl IRunLogRepo for NoopRunLogRepo {
        async fn create_run(&self, _: &PipelineRun) -> Result<(), lambda_runtime::Error> { Ok(()) }
        async fn complete_run(&self, _: &str, _: Option<i32>, _: &[PipelineWarning]) -> Result<(), lambda_runtime::Error> { Ok(()) }
        async fn fail_run(&self, _: &str, _: &str, _: Option<&str>, _: Option<i32>, _: &[PipelineWarning]) -> Result<(), lambda_runtime::Error> { Ok(()) }
        async fn fail_validation(&self, _: &str, _: &str) -> Result<(), lambda_runtime::Error> { Ok(()) }
        async fn store_validation_result(&self, _: &str, _: &ValidationResult) -> Result<(), lambda_runtime::Error> { Ok(()) }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_run_pipeline_uses_action_factory() {
        let factory = Arc::new(FakeFactory::new());
        let mut deps = Dependancies::new(Arc::new(Env::default())).await;
        deps.action_factory = factory.clone();
        deps.run_log_repo = Arc::new(NoopRunLogRepo);

        let manifest = onboard_you_models::Manifest {
            version: "1.0".into(),
            actions: vec![ActionConfig {
                id: "a".into(),
                action_type: ActionType::ApiDispatcher,
                config: ActionConfigPayload::ApiDispatcher(
                    onboard_you_models::ApiDispatcherConfig::Default,
                ),
            }],
        };

        let repo = PipelineRepository::new();
        let res = repo
            .run_pipeline(&deps, manifest, "org", "cust", "run-test-1")
            .await
            .expect("run pipeline");
        assert_eq!(res.status, "success");
        assert_eq!(factory.create_count.load(Ordering::SeqCst), 1);
        assert_eq!(factory.run_count.load(Ordering::SeqCst), 1);
    }
}
