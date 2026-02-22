use std::sync::Arc;

use crate::repositories::config_repository::{ConfigRepo, DynamoConfigRepo};
use crate::repositories::s3_repository::{S3Repo, S3Repository};
use crate::repositories::schedule_repository::{EventBridgeScheduleRepo, ScheduleRepo};
use crate::repositories::settings_repository::{DynamoSettingsRepo, SettingsRepo};

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
    pub dynamo: aws_sdk_dynamodb::Client,
    pub cognito: aws_sdk_cognitoidentityprovider::Client,
    pub cognito_client_id: String,
    pub csv_upload_bucket: String,
}

impl Dependancies {
    pub async fn new() -> Self {
        let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

        let dynamo = aws_sdk_dynamodb::Client::new(&aws_config);
        let table_name =
            std::env::var("CONFIG_TABLE_NAME").unwrap_or_else(|_| "PipelineConfigs".into());

        Self {
            config_repo: Arc::new(DynamoConfigRepo {
                dynamo: dynamo.clone(),
                table_name,
            }),
            settings_repo: Arc::new(DynamoSettingsRepo {
                dynamo: dynamo.clone(),
                table_name: std::env::var("SETTINGS_TABLE_NAME")
                    .unwrap_or_else(|_| "OrgSettings".into()),
            }),
            schedule_repo: Arc::new(EventBridgeScheduleRepo {
                scheduler: aws_sdk_scheduler::Client::new(&aws_config),
                etl_lambda_arn: std::env::var("ETL_LAMBDA_ARN")
                    .expect("ETL_LAMBDA_ARN must be set"),
                scheduler_role_arn: std::env::var("SCHEDULER_ROLE_ARN")
                    .expect("SCHEDULER_ROLE_ARN must be set"),
            }),
            s3_repo: Arc::new(S3Repository {
                s3: aws_sdk_s3::Client::new(&aws_config),
                bucket: std::env::var("CSV_UPLOAD_BUCKET")
                    .expect("CSV_UPLOAD_BUCKET must be set"),
            }),
            dynamo,
            cognito: aws_sdk_cognitoidentityprovider::Client::new(&aws_config),
            cognito_client_id: std::env::var("COGNITO_CLIENT_ID")
                .expect("COGNITO_CLIENT_ID must be set"),
            csv_upload_bucket: std::env::var("CSV_UPLOAD_BUCKET")
                .expect("CSV_UPLOAD_BUCKET must be set"),
        }
    }
}
