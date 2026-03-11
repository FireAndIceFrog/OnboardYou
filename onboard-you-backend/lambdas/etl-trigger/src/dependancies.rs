use std::sync::Arc;
use crate::repositories::{
    config_repository::{self, PgConfigRepo},
    etl_repository::{EtlRepository, IEtlRepo},
    pipeline_repository::{IPipelineRepo, PipelineRepository},
    settings_repository::{self, PgSettingsRepo},
};
use config_repository::IConfigRepo;
use onboard_you::ActionFactoryTrait;
use settings_repository::ISettingsRepo;

/// Environment configuration read from process env.
#[derive(Clone)]
pub struct Env {
    pub database_url: String,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            database_url: "postgres://localhost/unused".into(),
        }
    }
}

impl Env {
    pub fn from_env() -> Arc<Self> {
        Arc::new(Self {
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
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
    pub action_factory: Arc<dyn ActionFactoryTrait>,
}

impl Dependancies {
    /// Create a new `Dependancies` from the provided `Env`.
    /// This loads the AWS config and constructs the clients/repositories, so it's async.
    pub async fn new(cfg: Arc<Env>) -> Self {
        let pool_opts: sqlx::postgres::PgConnectOptions = cfg.database_url
            .parse()
            .expect("Failed to parse DATABASE_URL");
        let pool = sqlx::pool::PoolOptions::new()
            .connect_lazy_with(pool_opts.statement_cache_capacity(0));

        Self {
            config_repo: PgConfigRepo::new(pool.clone()),
            settings_repo: PgSettingsRepo::new(pool),
            etl_repo: EtlRepository::new(),
            pipeline_repo: PipelineRepository::new(),
            action_factory: Arc::new(onboard_you::ActionFactory::new()),
        }
    }
}
