use std::sync::Arc;
use reqwest::Client;

use gh_models::GHModels;
use crate::repositories::{
    config_repository::{self, DynamoConfigRepo},
    etl_repository::{EtlRepository, IEtlRepo},
    pipeline_repository::{IPipelineRepo, PipelineRepository},
    settings_repository::{self, DynamoSettingsRepo},
    gh_models_repository::{GhModelsRepo, GhModelsRepoImpl},
    openapi_repository::{OpenApiRepo, SimpleOpenApiRepo},
};
use config_repository::IConfigRepo;
use onboard_you::ActionFactoryTrait;
use settings_repository::ISettingsRepo;

/// Environment configuration read from process env.
#[derive(Clone, Default)]
pub struct Env {
    pub table_name: String,
    pub settings_table_name: String,
    pub gh_token: String,
}

impl Env {
    pub fn from_env() -> Arc<Self> {
        Arc::new(Self {
            table_name: std::env::var("CONFIG_TABLE_NAME")
                .unwrap_or_else(|_| "PipelineConfigs".to_string()),
            settings_table_name: std::env::var("SETTINGS_TABLE_NAME")
                .unwrap_or_else(|_| "OrgSettings".to_string()),
            gh_token: std::env::var("GITHUB_TOKEN").expect("Missing GITHUB_TOKEN"),
        })
    }
}

// Traits and concrete implementations live in the repository modules.

/// Runtime dependancies (repositories/engines) constructed from `Env`.
pub struct Dependancies {
    pub config_repo: Arc<dyn IConfigRepo>,
    pub settings_repo: Arc<dyn ISettingsRepo>,
    pub etl_repo: Arc<dyn IEtlRepo>,
    pub pipeline_repo: Arc<dyn IPipelineRepo>,
    pub gh_models_repo: Arc<dyn GhModelsRepo>,
    pub openapi_repo: Arc<dyn OpenApiRepo>,
    pub action_factory: Arc<dyn ActionFactoryTrait>,
}

impl Dependancies {
    /// Create a new `Dependancies` from the provided `Env`.
    /// This loads the AWS config and constructs the clients/repositories, so it's async.
    pub async fn new(cfg: Arc<Env>) -> Self {
        let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let dynamo = aws_sdk_dynamodb::Client::new(&aws_config);

        // Construct concrete Dynamo-backed repo implementations from their modules
        let gh_client = Arc::new(GHModels::new(cfg.gh_token.clone()));
        let http_client = Client::new();
        Self {
            config_repo: DynamoConfigRepo::new(dynamo.clone(), cfg.table_name.clone()),
            settings_repo: DynamoSettingsRepo::new(dynamo.clone(), cfg.settings_table_name.clone()),
            etl_repo: EtlRepository::new(),
            pipeline_repo: PipelineRepository::new(),
            gh_models_repo: GhModelsRepoImpl::new(gh_client),
            openapi_repo: SimpleOpenApiRepo::new(http_client.clone()),
            action_factory: Arc::new(onboard_you::ActionFactory::new()),
        }
    }
}
