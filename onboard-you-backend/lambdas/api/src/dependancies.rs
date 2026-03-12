use std::sync::Arc;

use crate::repositories::cognito_repository::{AuthRepo, CognitoAuthRepo};
use crate::repositories::config_repository::{ConfigRepo, PgConfigRepo};
use crate::repositories::etl_repository::{EtlRepo, EtlRepository};
use crate::repositories::run_history_repository::{PgRunHistoryRepo, RunHistoryRepo};
use crate::repositories::s3_repository::{S3Repo, S3Repository};
use crate::repositories::schedule_repository::{EventBridgeScheduleRepo, ScheduleRepo};
use crate::repositories::settings_repository::{PgSettingsRepo, SettingsRepo};

#[derive(Debug, Clone)]
pub struct Env {
    pub database_url: String,
    pub etl_lambda_arn: String,
    pub scheduler_role_arn: String,
    pub csv_upload_bucket: String,
    pub cognito_client_id: String,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            database_url: "postgres://localhost/unused".into(),
            etl_lambda_arn: String::new(),
            scheduler_role_arn: String::new(),
            csv_upload_bucket: String::new(),
            cognito_client_id: String::new(),
        }
    }
}

/// Shared application state, injected via axum's State extractor.
///
/// Repository traits are behind `Arc<dyn Trait>` so the engine layer
/// can be tested with in-memory fakes — no AWS calls needed.
#[derive(Clone)]
pub struct Dependancies {
    pub config_repo: Arc<dyn ConfigRepo>,
    pub schedule_repo: Arc<dyn ScheduleRepo>,
    pub settings_repo: Arc<dyn SettingsRepo>,
    pub s3_repo: Arc<dyn S3Repo>,
    pub auth_repo: Arc<dyn AuthRepo>,
    pub etl_repo: Arc<dyn EtlRepo>,
    pub run_history_repo: Arc<dyn RunHistoryRepo>,
}

impl Dependancies {
    pub fn create_env() -> Env {
        Env {
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            etl_lambda_arn: std::env::var("ETL_LAMBDA_ARN").expect("ETL_LAMBDA_ARN must be set"),
            scheduler_role_arn: std::env::var("SCHEDULER_ROLE_ARN")
                .expect("SCHEDULER_ROLE_ARN must be set"),
            csv_upload_bucket: std::env::var("CSV_UPLOAD_BUCKET")
                .expect("CSV_UPLOAD_BUCKET must be set"),
            cognito_client_id: std::env::var("COGNITO_CLIENT_ID")
                .expect("COGNITO_CLIENT_ID must be set"),
        }
    }

    pub async fn new(env: Env) -> Self {
        let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

        let pool_opts: sqlx::postgres::PgConnectOptions = env.database_url
            .parse()
            .expect("Failed to parse DATABASE_URL");
        let pool = sqlx::pool::PoolOptions::new()
            .connect_lazy_with(pool_opts.statement_cache_capacity(0));


        Self {
            config_repo: Arc::new(PgConfigRepo {
                pool: pool.clone(),
            }),
            settings_repo: Arc::new(PgSettingsRepo {
                pool: pool.clone(),
            }),
            run_history_repo: Arc::new(PgRunHistoryRepo {
                pool,
            }),
            schedule_repo: Arc::new(EventBridgeScheduleRepo {
                scheduler: aws_sdk_scheduler::Client::new(&aws_config),
                env: env.clone(),
            }),
            s3_repo: Arc::new(S3Repository {
                s3: aws_sdk_s3::Client::new(&aws_config),
                bucket: env.csv_upload_bucket.clone(),
            }),
            auth_repo: Arc::new(CognitoAuthRepo {
                cognito: aws_sdk_cognitoidentityprovider::Client::new(&aws_config),
                client_id: env.cognito_client_id.clone(),
            }),
            etl_repo: Arc::new(EtlRepository {}),
        }
    }
}
