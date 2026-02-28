//! Pipeline engine — loads config, builds actions, runs the ETL pipeline.
//!
//! When a manifest action specifies `auth_type: "default"`, the engine
//! fetches the organisation's stored auth settings from the settings table
//! and injects them into the action config before factory construction.

use lambda_runtime::Error;
use std::sync::Arc;

use crate::dependancies::Dependancies;

use crate::models::PipelineResult;

/// Load config from DynamoDB, build the pipeline, and execute it.
pub async fn run(
    deps: Arc<Dependancies>,
    organization_id: &str,
    customer_company_id: &str,
) -> Result<PipelineResult, Error> {
    tracing::info!(%organization_id, %customer_company_id, "ETL trigger fired");

    // 1. Fetch config via injected repository
    let config = deps
        .config_repo
        .get(organization_id, customer_company_id)
        .await?;

    // 2. Deserialize the Manifest
    let mut manifest = config.pipeline;

    // 3. Resolve any "default" auth types from the settings table via injected repo
    manifest = deps
        .etl_repo
        .resolve_default_auth(&deps, &mut manifest, organization_id)
        .await?;

    // 3b. Resolve CSV S3 keys from org_id / company_id / filename
    manifest =
        deps.etl_repo
            .resolve_csv_s3_keys(&mut manifest, organization_id, customer_company_id)?;

    deps.pipeline_repo
        .run_pipeline(&deps, manifest, organization_id, customer_company_id)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependancies::Dependancies;
    use crate::repositories::{
        config_repository::IConfigRepo, etl_repository::IEtlRepo,
        pipeline_repository::IPipelineRepo, settings_repository::ISettingsRepo,
    };
    use async_trait::async_trait;
    use lambda_runtime::Error;
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
        ) -> Result<onboard_you::PipelineConfig, Error> {
            self.called.store(true, Ordering::SeqCst);
            let manifest = onboard_you::Manifest {
                version: "1.0".into(),
                actions: vec![onboard_you::ActionConfig {
                    id: "egress".into(),
                    action_type: onboard_you::ActionType::ApiDispatcher,
                    config: onboard_you::ActionConfigPayload::ApiDispatcher(
                        onboard_you::ApiDispatcherConfig::Default,
                    ),
                }],
            };
            Ok(onboard_you::PipelineConfig {
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
            manifest: &mut onboard_you::Manifest,
            _organization_id: &str,
        ) -> Result<onboard_you::Manifest, Error> {
            self.resolved_default.store(true, Ordering::SeqCst);
            Ok(manifest.clone())
        }

        fn resolve_csv_s3_keys(
            &self,
            manifest: &mut onboard_you::Manifest,
            _organization_id: &str,
            _customer_company_id: &str,
        ) -> Result<onboard_you::Manifest, Error> {
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
            _manifest: onboard_you::Manifest,
            _organization_id: &str,
            _customer_company_id: &str,
        ) -> Result<crate::models::PipelineResult, Error> {
            self.called.store(true, Ordering::SeqCst);
            Ok(crate::models::PipelineResult::success(
                _organization_id,
                _customer_company_id,
                Some(0),
            ))
        }
    }

    struct FakeSettingsRepo;
    #[async_trait]
    impl ISettingsRepo for FakeSettingsRepo {
        async fn get(
            &self,
            _organization_id: &str,
        ) -> Result<Option<onboard_you::OrgSettings>, Error> {
            Ok(None)
        }
    }

    #[tokio::test]
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
