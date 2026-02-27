use std::sync::Arc;

use crate::repositories::cognito_repository::{AuthRepo, CognitoAuthRepo};
use crate::repositories::config_repository::{ConfigRepo, DynamoConfigRepo};
use crate::repositories::etl_repository::{EtlRepo, EtlRepository};
use crate::repositories::s3_repository::{S3Repo, S3Repository};
use crate::repositories::schedule_repository::{EventBridgeScheduleRepo, ScheduleRepo};
use aws_sdk_sqs::Client as SqsClient;
use crate::repositories::settings_repository::{DynamoSettingsRepo, SettingsRepo};

#[derive(Debug, Clone, Default)]
pub struct Env {
    pub config_table_name: String,
    pub settings_table_name: String,
    pub etl_lambda_arn: String,
    pub scheduler_role_arn: String,
    pub csv_upload_bucket: String,
    pub cognito_client_id: String,
    pub sqs_queue_url: String,
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
}

impl Dependancies {
    pub fn create_env() -> Env {
        Env {
            config_table_name: std::env::var("CONFIG_TABLE_NAME")
                .unwrap_or_else(|_| "PipelineConfigs".into()),
            settings_table_name: std::env::var("SETTINGS_TABLE_NAME")
                .unwrap_or_else(|_| "OrgSettings".into()),
            etl_lambda_arn: std::env::var("ETL_LAMBDA_ARN").expect("ETL_LAMBDA_ARN must be set"),
            scheduler_role_arn: std::env::var("SCHEDULER_ROLE_ARN")
                .expect("SCHEDULER_ROLE_ARN must be set"),
            csv_upload_bucket: std::env::var("CSV_UPLOAD_BUCKET")
                .expect("CSV_UPLOAD_BUCKET must be set"),
            cognito_client_id: std::env::var("COGNITO_CLIENT_ID")
                .expect("COGNITO_CLIENT_ID must be set"),
            sqs_queue_url: std::env::var("SQS_QUEUE_URL")
                .expect("SQS_QUEUE_URL must be set"),
        }
    }

    pub async fn new(env: Env) -> Self {
        let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

        let dynamo = aws_sdk_dynamodb::Client::new(&aws_config);

        Self {
            config_repo: Arc::new(DynamoConfigRepo {
                dynamo: dynamo.clone(),
                table_name: env.config_table_name.clone(),
            }),
            settings_repo: Arc::new(DynamoSettingsRepo {
                dynamo: dynamo.clone(),
                table_name: env.settings_table_name.clone(),
            }),
            schedule_repo: Arc::new(EventBridgeScheduleRepo {
                scheduler: aws_sdk_scheduler::Client::new(&aws_config),
                sqs: SqsClient::new(&aws_config),
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
