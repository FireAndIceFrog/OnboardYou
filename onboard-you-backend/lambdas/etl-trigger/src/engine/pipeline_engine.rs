//! Pipeline engine — loads config, validates columns, builds actions, runs the ETL pipeline.
//!
//! Before executing the pipeline, the engine performs a dry-run validation
//! (column propagation via `calculate_columns`) to catch schema mismatches
//! early. Both the validation result and the run outcome are persisted to
//! the `pipeline_runs` table.
//!
//! When a manifest action specifies `auth_type: "default"`, the engine
//! fetches the organisation's stored auth settings from the settings table
//! and injects them into the action config before factory construction.

use lambda_runtime::Error;
use std::sync::Arc;

use onboard_you_models::PipelineRun;

use crate::dependancies::Dependancies;
use crate::models::PipelineResult;

/// Generate a short random run ID (UUID v4 hex, no dashes).
fn new_run_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    // Combine timestamp with a random component for uniqueness
    let random: u64 = {
        // Simple xorshift pseudo-random — good enough for an ID, not crypto
        let mut seed = ts as u64 ^ 0x517cc1b727220a95;
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    format!("{ts:x}-{random:x}")
}

/// Load config, validate schema, build the pipeline, and execute it.
pub async fn run(
    deps: Arc<Dependancies>,
    organization_id: &str,
    customer_company_id: &str,
) -> Result<PipelineResult, Error> {
    tracing::info!(%organization_id, %customer_company_id, "ETL trigger fired");

    let run_id = new_run_id();

    // 1. Fetch config via injected repository
    let config = deps
        .config_repo
        .get(organization_id, customer_company_id)
        .await?;

    // 2. Deserialize the Manifest
    let mut manifest = config.pipeline;

    // 3. Resolve any "default" auth types from the settings table
    manifest = deps
        .etl_repo
        .resolve_default_auth(&deps, &mut manifest, organization_id)
        .await?;

    // 3b. Resolve CSV S3 keys from org_id / company_id / filename
    manifest =
        deps.etl_repo
            .resolve_csv_s3_keys(&mut manifest, organization_id, customer_company_id)?;

    // Create the run record (status = "running") with resolved manifest snapshot
    let run_record = PipelineRun {
        id: run_id.clone(),
        organization_id: organization_id.to_string(),
        customer_company_id: customer_company_id.to_string(),
        status: "running".to_string(),
        started_at: String::new(), // DB defaults to NOW()
        finished_at: None,
        rows_processed: None,
        current_action: None,
        error_message: None,
        error_action_id: None,
        error_row: None,
        warnings: vec![],
        validation_result: None,
        manifest_snapshot: Some(manifest.clone()),
    };
    let _ = deps.run_log_repo.create_run(&run_record).await;

    // 4. Pre-flight validation: dry-run column propagation
    let action_factory = deps.action_factory.clone();
    let validation_result = match action_factory.validate_manifest(&manifest) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!(
                "Pre-flight validation failed at step '{}' ({}): {}",
                e.action_id, e.action_type, e.error
            );
            tracing::error!(%organization_id, %customer_company_id, %msg);

            let _ = deps.run_log_repo.fail_validation(&run_id, &msg).await;

            return Ok(PipelineResult::failure(
                &run_id,
                organization_id,
                customer_company_id,
                &msg,
                vec![],
            ));
        }
    };

    // Store validation result
    let _ = deps
        .run_log_repo
        .store_validation_result(&run_id, &validation_result)
        .await;

    tracing::info!(
        %organization_id, %customer_company_id,
        steps = validation_result.steps.len(),
        "Pre-flight validation passed"
    );

    // 5. Execute the pipeline
    deps.pipeline_repo
        .run_pipeline(&deps, manifest, organization_id, customer_company_id, &run_id)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependancies::Dependancies;
    use crate::repositories::{
        config_repository::IConfigRepo, etl_repository::IEtlRepo,
        pipeline_repository::IPipelineRepo, run_log_repository::IRunLogRepo,
        settings_repository::ISettingsRepo,
    };
    use async_trait::async_trait;
    use lambda_runtime::Error;
    use onboard_you_models::PipelineWarning;
    use onboard_you_models::ValidationResult;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    struct FakeConfigRepo {
        called: Arc<AtomicBool>,
    }

    #[async_trait]
    impl IConfigRepo for FakeConfigRepo {
        async fn get(
            &self,
            _organization_id: &str,
            _customer_company_id: &str,
        ) -> Result<onboard_you_models::PipelineConfig, Error> {
            self.called.store(true, Ordering::SeqCst);
            let manifest = onboard_you_models::Manifest {
                version: "1.0".into(),
                actions: vec![onboard_you_models::ActionConfig {
                    id: "egress".into(),
                    action_type: onboard_you_models::ActionType::ApiDispatcher,
                    config: onboard_you_models::ActionConfigPayload::ApiDispatcher(
                        onboard_you_models::ApiDispatcherConfig::Default,
                    ),
                }],
            };
            Ok(onboard_you_models::PipelineConfig {
                name: "test".into(),
                image: None,
                cron: "rate(1 hour)".into(),
                organization_id: _organization_id.into(),
                customer_company_id: _customer_company_id.into(),
                last_edited: "".into(),
                pipeline: manifest,
            })
        }
    }

    struct FakeEtlRepo {
        resolved_default: Arc<AtomicBool>,
        resolved_csv: Arc<AtomicBool>,
    }

    #[async_trait]
    impl IEtlRepo for FakeEtlRepo {
        async fn resolve_default_auth(
            &self,
            _deps: &Dependancies,
            manifest: &mut onboard_you_models::Manifest,
            _organization_id: &str,
        ) -> Result<onboard_you_models::Manifest, Error> {
            self.resolved_default.store(true, Ordering::SeqCst);
            Ok(manifest.clone())
        }

        fn resolve_csv_s3_keys(
            &self,
            manifest: &mut onboard_you_models::Manifest,
            _organization_id: &str,
            _customer_company_id: &str,
        ) -> Result<onboard_you_models::Manifest, Error> {
            self.resolved_csv.store(true, Ordering::SeqCst);
            Ok(manifest.clone())
        }
    }

    struct FakePipelineRepo {
        called: Arc<AtomicBool>,
    }

    #[async_trait]
    impl IPipelineRepo for FakePipelineRepo {
        async fn run_pipeline(
            &self,
            _deps: &Dependancies,
            _manifest: onboard_you_models::Manifest,
            _organization_id: &str,
            _customer_company_id: &str,
            _run_id: &str,
        ) -> Result<crate::models::PipelineResult, Error> {
            self.called.store(true, Ordering::SeqCst);
            Ok(crate::models::PipelineResult::success(
                _run_id,
                _organization_id,
                _customer_company_id,
                Some(0),
                vec![],
            ))
        }
    }

    struct FakeSettingsRepo;
    #[async_trait]
    impl ISettingsRepo for FakeSettingsRepo {
        async fn get(
            &self,
            _organization_id: &str,
        ) -> Result<Option<onboard_you_models::OrgSettings>, Error> {
            Ok(None)
        }
    }

    struct NoopRunLogRepo;

    #[async_trait]
    impl IRunLogRepo for NoopRunLogRepo {
        async fn create_run(&self, _: &PipelineRun) -> Result<(), Error> { Ok(()) }
        async fn complete_run(&self, _: &str, _: Option<i32>, _: &[PipelineWarning]) -> Result<(), Error> { Ok(()) }
        async fn fail_run(&self, _: &str, _: &str, _: Option<&str>, _: Option<i32>, _: &[PipelineWarning]) -> Result<(), Error> { Ok(()) }
        async fn fail_validation(&self, _: &str, _: &str) -> Result<(), Error> { Ok(()) }
        async fn store_validation_result(&self, _: &str, _: &ValidationResult) -> Result<(), Error> { Ok(()) }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_run_calls_repos_and_pipeline() {
        let cfg_called = Arc::new(AtomicBool::new(false));
        let etl_default = Arc::new(AtomicBool::new(false));
        let etl_csv = Arc::new(AtomicBool::new(false));
        let pipeline_called = Arc::new(AtomicBool::new(false));


        let deps = Arc::new(Dependancies {
            config_repo: Arc::new(FakeConfigRepo {
                called: cfg_called.clone(),
            }),
            settings_repo: Arc::new(FakeSettingsRepo),
            etl_repo: Arc::new(FakeEtlRepo {
                resolved_default: etl_default.clone(),
                resolved_csv: etl_csv.clone(),
            }),
            pipeline_repo: Arc::new(FakePipelineRepo {
                called: pipeline_called.clone(),
            }),
            run_log_repo: Arc::new(NoopRunLogRepo),
            action_factory: Arc::new(onboard_you::ActionFactory::new()),
        });

        let result = run(deps.clone(), "org-1", "cust-1").await.expect("run ok");
        assert_eq!(result.status, "success");
        assert!(cfg_called.load(Ordering::SeqCst));
        assert!(etl_default.load(Ordering::SeqCst));
        assert!(etl_csv.load(Ordering::SeqCst));
        assert!(pipeline_called.load(Ordering::SeqCst));
    }
}
