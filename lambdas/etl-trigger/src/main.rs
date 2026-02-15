//! ETL Trigger Lambda
//!
//! Bootstrap only. Read engine/pipeline_engine.rs for what the pipeline does.

mod engine;
mod models;
mod repositories;

use lambda_runtime::{service_fn, Error, LambdaEvent};
use models::ScheduleEvent;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Error> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let dynamo = aws_sdk_dynamodb::Client::new(&aws_config);
    let table_name =
        std::env::var("CONFIG_TABLE_NAME").unwrap_or_else(|_| "PipelineConfigs".to_string());
    let settings_table_name =
        std::env::var("SETTINGS_TABLE_NAME").unwrap_or_else(|_| "OrgSettings".to_string());

    lambda_runtime::run(service_fn(|event: LambdaEvent<ScheduleEvent>| {
        let dynamo = dynamo.clone();
        let table_name = table_name.clone();
        let settings_table_name = settings_table_name.clone();
        async move {
            engine::pipeline_engine::run(
                &dynamo,
                &table_name,
                &settings_table_name,
                &event.payload.organization_id,
                &event.payload.customer_company_id,
            )
            .await
        }
    }))
    .await
}
