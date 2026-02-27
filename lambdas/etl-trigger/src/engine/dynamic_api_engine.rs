//! Engine for the "DynamicApi" scheduled event.
//!
//! This module is intentionally small – the business logic lives in the
//! `run` function which is exercised by unit tests below using in‑memory
//! fake repositories.

use lambda_runtime::Error;
use onboard_you::ApiDispatcherConfig;
use std::sync::Arc;

use crate::dependancies::Dependancies;

/// Execute the dynamic‑api workflow for a single organisation/company pair.
///
/// The caller already knows which company triggered the event but we don’t use
/// that value here; it is logged purely for symmetry with other engines.
pub async fn run(
    deps: Arc<Dependancies>,
    organization_id: &str,
    customer_company_id: &str,

) -> Result<(), Error> {
    tracing::info!(%organization_id, %customer_company_id, "DynamicApi event received");
    //get settings model

    let settings = match deps.settings_repo.get(organization_id).await? {
        Some(s) => s,
        None => {
            tracing::warn!(%organization_id, "No settings found for org, using defaults");
            return Err(Error::from("No settings found for org"));
        }
    };
    let url = match settings.default_auth.clone() {
        ApiDispatcherConfig::Bearer(cfg) => cfg.output_schema_openapi_url,
        ApiDispatcherConfig::OAuth(cfg) => cfg.output_schema_openapi_url,
        ApiDispatcherConfig::OAuth2(cfg) => cfg.output_schema_openapi_url,
        ApiDispatcherConfig::Default => {
            tracing::warn!(%organization_id, "Default auth type found in settings, cannot proceed with Dynamic API workflow");
            return Err(Error::from("Default auth type found in settings"));
        },
    }.unwrap_or_default();

    tracing::info!(%organization_id, openapi_url = %url, "Fetching OpenAPI schema");
    //get openapi schema
    let openapi_json = deps.openapi_repo.fetch(&url.clone()).await?;

    tracing::info!(%organization_id, openapi_url = %url, "Fetched OpenAPI schema");

    // parse openapi schema and generate manifest

    
    let dynamic_api = deps.gh_models_repo.generate_dynamic_body(&openapi_json).await?;

    let modified_schema: ApiDispatcherConfig = match settings.default_auth.clone() {
        ApiDispatcherConfig::Bearer(cfg) => {
            let mut new_cfg = cfg.clone();
            new_cfg.output_schema = Some(dynamic_api.output_schema);
            new_cfg.output_schema_body_path = Some(dynamic_api.output_schema_body_path);
            onboard_you::ApiDispatcherConfig::Bearer(new_cfg)
        },
        ApiDispatcherConfig::OAuth(cfg) => {
            let mut new_cfg = cfg.clone();
            new_cfg.output_schema = Some(dynamic_api.output_schema);
            new_cfg.output_schema_body_path = Some(dynamic_api.output_schema_body_path);
            onboard_you::ApiDispatcherConfig::OAuth(new_cfg)
        },
        ApiDispatcherConfig::OAuth2(cfg) => {
            let mut new_cfg = cfg.clone();
            new_cfg.output_schema = Some(dynamic_api.output_schema);
            new_cfg.output_schema_body_path = Some(dynamic_api.output_schema_body_path);
            onboard_you::ApiDispatcherConfig::OAuth2(new_cfg)
        },
        _ => {
            tracing::warn!(%organization_id, "Default auth type found in settings, cannot proceed with Dynamic API workflow");
            return Err(Error::from("Default auth type found in settings"));
        },
    };

    let mut settings = settings.clone();
    settings.default_auth = modified_schema.clone();

    deps.settings_repo.save(&settings).await?;
    
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependancies::Dependancies;
    use crate::repositories::{
        settings_repository::ISettingsRepo,
        openapi_repository::OpenApiRepo,
        gh_models_repository::GhModelsRepo,
    };

    // keep async_trait import for fake repos below
    use async_trait::async_trait;
    use lambda_runtime::Error;
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::{AtomicBool, Ordering};

    /// A trivial settings repo that lets us control the return value and
    /// observe what gets saved.
    struct FakeSettingsRepo {
        pub get_called: Arc<AtomicBool>,
        pub save_called: Arc<AtomicBool>,
        pub last_saved: Arc<Mutex<Option<onboard_you::OrgSettings>>>,
        pub return_settings: Option<onboard_you::OrgSettings>,
    }

    #[async_trait]
    impl ISettingsRepo for FakeSettingsRepo {
        async fn get(
            &self,
            _organization_id: &str,
        ) -> Result<Option<onboard_you::OrgSettings>, Error> {
            self.get_called.store(true, Ordering::SeqCst);
            Ok(self.return_settings.clone())
        }

        async fn save(&self, settings: &onboard_you::OrgSettings) -> Result<(), Error> {
            self.save_called.store(true, Ordering::SeqCst);
            let mut lock = self.last_saved.lock().unwrap();
            *lock = Some(settings.clone());
            Ok(())
        }
    }

    /// Fake OpenAPI repo that records the requested URL.
    struct FakeOpenApiRepo {
        pub called: Arc<AtomicBool>,
        pub last_url: Arc<Mutex<Option<String>>>,
    }

    #[async_trait]
    impl OpenApiRepo for FakeOpenApiRepo {
        async fn fetch(&self, url: &str) -> Result<String, Error> {
            self.called.store(true, Ordering::SeqCst);
            let mut lock = self.last_url.lock().unwrap();
            *lock = Some(url.to_string());
            Ok(r#"{"paths":{}}"#.to_string())
        }
    }

    /// Dummy GH repo that returns a canned dynamic-api response.
    struct DummyGhRepo;
    #[async_trait]
    impl GhModelsRepo for DummyGhRepo {
        async fn generate_dynamic_body(&self, _input: &str) -> Result<crate::models::OpenapiDynamicApiResponse, Error> {
            Ok(crate::models::OpenapiDynamicApiResponse {
                output_schema: serde_json::json!({"foo": "bar"}),
                output_schema_body_path: "data.items".to_string(),
            })
        }
    }

    // helper to make a basic Bearer settings object
    fn bearer_settings_with_url(url: &str) -> onboard_you::OrgSettings {
        onboard_you::OrgSettings {
            organization_id: "org-1".into(),
            default_auth: onboard_you::ApiDispatcherConfig::Bearer(
                onboard_you::BearerRepoConfig {
                    destination_url: "https://example.com".into(),
                    token: Some("tok".into()),
                    placement: onboard_you::BearerPlacement::AuthorizationHeader,
                    placement_key: None,
                    extra_headers: Default::default(),
                    schema_generation_status: None,
                    output_schema: None,
                    output_schema_body_path: None,
                    output_schema_openapi_url: Some(url.into()),
                },
            ),
        }
    }

    #[tokio::test]
    async fn test_run_success_updates_settings() {
        let get_called = Arc::new(AtomicBool::new(false));
        let save_called = Arc::new(AtomicBool::new(false));
        let last_saved: Arc<Mutex<Option<onboard_you::OrgSettings>>> = Arc::new(Mutex::new(None));
        let openapi_called = Arc::new(AtomicBool::new(false));
        let openapi_url: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let settings = bearer_settings_with_url("http://schema.url");
        let fake_settings = FakeSettingsRepo {
            get_called: get_called.clone(),
            save_called: save_called.clone(),
            last_saved: last_saved.clone(),
            return_settings: Some(settings.clone()),
        };

        let fake_openapi = FakeOpenApiRepo {
            called: openapi_called.clone(),
            last_url: openapi_url.clone(),
        };

        // start from a fully-populated Dependancies instance and override the
        // few fields we care about for the test.  this mirrors the behaviour
        // described by the user request.
        let mut deps = Dependancies::new(Arc::new(crate::dependancies::Env::default())).await;
        deps.settings_repo = Arc::new(fake_settings);
        deps.openapi_repo = Arc::new(fake_openapi);
        deps.gh_models_repo = Arc::new(DummyGhRepo);
        let deps = Arc::new(deps);

        // run should succeed without error
        run(deps.clone(), "org-1", "cust-1").await.unwrap();

        assert!(get_called.load(Ordering::SeqCst));
        assert!(openapi_called.load(Ordering::SeqCst));
        assert_eq!(openapi_url.lock().unwrap().as_deref(), Some("http://schema.url"));
        assert!(save_called.load(Ordering::SeqCst));

        // ensure the saved settings contain the generated schema
        let saved = last_saved.lock().unwrap();
        let saved = saved.as_ref().expect("settings were saved");
        if let onboard_you::ApiDispatcherConfig::Bearer(cfg) = &saved.default_auth {
            assert_eq!(cfg.output_schema, Some(serde_json::json!({"foo":"bar"})));
            assert_eq!(cfg.output_schema_body_path.as_deref(), Some("data.items"));
        } else {
            panic!("unexpected auth type");
        }
    }

    #[tokio::test]
    async fn test_run_missing_settings_returns_error() {
        let get_called = Arc::new(AtomicBool::new(false));
        let fake_settings = FakeSettingsRepo {
            get_called: get_called.clone(),
            save_called: Arc::new(AtomicBool::new(false)),
            last_saved: Arc::new(Mutex::new(None)),
            return_settings: None,
        };

        let mut deps = Dependancies::new(Arc::new(crate::dependancies::Env::default())).await;
        deps.settings_repo = Arc::new(fake_settings);
        deps.openapi_repo = Arc::new(FakeOpenApiRepo {
            called: Arc::new(AtomicBool::new(false)),
            last_url: Arc::new(Mutex::new(None)),
        });
        deps.gh_models_repo = Arc::new(DummyGhRepo);
        let deps = Arc::new(deps);

        let err = run(deps.clone(), "org-1", "cust-1").await.err().unwrap();
        assert!(err.to_string().contains("No settings found"));
        assert!(get_called.load(Ordering::SeqCst));
    }

}