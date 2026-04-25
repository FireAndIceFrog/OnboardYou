use std::sync::Arc;

use crate::repositories::{
    email_route_repository::{EmailRouteRepository, IEmailRouteRepo},
    s3_repository::{IS3Repo, S3Repository},
    sqs_repository::{ISqsRepo, SqsRepository},
    textract_repository::{ITextractRepo, TextractRepository},
};

/// Environment variables required by this Lambda.
#[derive(Clone, Debug)]
pub struct Env {
    pub ses_inbox_bucket: String,
    pub csv_upload_bucket: String,
    pub email_routes_table: String,
    pub etl_sqs_queue_url: String,
}

impl Env {
    pub fn from_env() -> Result<Arc<Self>, String> {
        Ok(Arc::new(Self {
            ses_inbox_bucket: std::env::var("SES_INBOX_BUCKET")
                .map_err(|_| "SES_INBOX_BUCKET not set")?,
            csv_upload_bucket: std::env::var("CSV_UPLOAD_BUCKET")
                .map_err(|_| "CSV_UPLOAD_BUCKET not set")?,
            email_routes_table: std::env::var("EMAIL_ROUTES_TABLE")
                .map_err(|_| "EMAIL_ROUTES_TABLE not set")?,
            etl_sqs_queue_url: std::env::var("ETL_SQS_QUEUE_URL")
                .map_err(|_| "ETL_SQS_QUEUE_URL not set")?,
        }))
    }
}

/// Repositories injected into every engine function.
/// Hold `Arc<dyn ITrait>` fields so tests can substitute fakes.
pub struct Dependancies {
    pub env: Arc<Env>,
    pub s3_repo: Arc<dyn IS3Repo>,
    pub email_route_repo: Arc<dyn IEmailRouteRepo>,
    pub sqs_repo: Arc<dyn ISqsRepo>,
    pub textract_repo: Arc<dyn ITextractRepo>,
}

impl Dependancies {
    /// Construct the production set of dependencies from the resolved environment.
    pub async fn new(env: Arc<Env>) -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        Self {
            s3_repo: S3Repository::new(aws_sdk_s3::Client::new(&config)),
            email_route_repo: EmailRouteRepository::new(aws_sdk_dynamodb::Client::new(&config)),
            sqs_repo: SqsRepository::new(aws_sdk_sqs::Client::new(&config)),
            textract_repo: TextractRepository::new(aws_sdk_textract::Client::new(&config)),
            env,
        }
    }
}
