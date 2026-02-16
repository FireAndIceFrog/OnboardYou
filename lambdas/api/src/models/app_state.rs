/// Shared AWS client state, injected via axum's State extractor.
#[derive(Clone)]
pub struct AppState {
    pub dynamo: aws_sdk_dynamodb::Client,
    pub scheduler: aws_sdk_scheduler::Client,
    pub cognito: aws_sdk_cognitoidentityprovider::Client,
    pub table_name: String,
    pub settings_table_name: String,
    pub etl_lambda_arn: String,
    pub scheduler_role_arn: String,
    pub cognito_client_id: String,
}

impl AppState {
    pub async fn from_env() -> Self {
        let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;

        Self {
            dynamo: aws_sdk_dynamodb::Client::new(&aws_config),
            scheduler: aws_sdk_scheduler::Client::new(&aws_config),
            cognito: aws_sdk_cognitoidentityprovider::Client::new(&aws_config),
            table_name: std::env::var("CONFIG_TABLE_NAME")
                .unwrap_or_else(|_| "PipelineConfigs".into()),
            settings_table_name: std::env::var("SETTINGS_TABLE_NAME")
                .unwrap_or_else(|_| "OrgSettings".into()),
            etl_lambda_arn: std::env::var("ETL_LAMBDA_ARN").expect("ETL_LAMBDA_ARN must be set"),
            scheduler_role_arn: std::env::var("SCHEDULER_ROLE_ARN")
                .expect("SCHEDULER_ROLE_ARN must be set"),
            cognito_client_id: std::env::var("COGNITO_CLIENT_ID")
                .expect("COGNITO_CLIENT_ID must be set"),
        }
    }
}
