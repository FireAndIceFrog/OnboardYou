//! ETL Trigger Lambda
//!
//! Invoked by EventBridge Scheduler on a per-organization cron.
//! 1. Receives { "organizationId": "..." } from the schedule
//! 2. Reads the full pipeline config from DynamoDB
//! 3. Deserializes the Manifest and runs the ETL pipeline

use aws_sdk_dynamodb::types::AttributeValue;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{fmt, EnvFilter};

use onboard_you::{ActionFactory, Manifest, PipelineRunner, RosterContext};

/// Event payload from EventBridge Scheduler
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScheduleEvent {
    organization_id: String,
}

/// Response payload
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PipelineResult {
    organization_id: String,
    status: String,
    rows_processed: Option<usize>,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let dynamo_client = aws_sdk_dynamodb::Client::new(&aws_config);

    let table_name =
        std::env::var("CONFIG_TABLE_NAME").unwrap_or_else(|_| "PipelineConfigs".to_string());

    run_lambda(dynamo_client, table_name).await
}

async fn run_lambda(dynamo: aws_sdk_dynamodb::Client, table_name: String) -> Result<(), Error> {
    lambda_runtime::run(service_fn(|event: LambdaEvent<ScheduleEvent>| {
        let dynamo = dynamo.clone();
        let table_name = table_name.clone();
        async move { handle_event(event, &dynamo, &table_name).await }
    }))
    .await
}

async fn handle_event(
    event: LambdaEvent<ScheduleEvent>,
    dynamo: &aws_sdk_dynamodb::Client,
    table_name: &str,
) -> Result<PipelineResult, Error> {
    let organization_id = &event.payload.organization_id;

    tracing::info!(%organization_id, "ETL trigger fired");

    // 1. Fetch config from DynamoDB
    let config = fetch_config(dynamo, table_name, organization_id).await?;

    let pipeline_json = config
        .get("pipeline")
        .and_then(|v| v.as_s().ok())
        .cloned()
        .ok_or_else(|| Error::from("Missing pipeline field in config"))?;

    tracing::info!(%organization_id, "Pipeline config loaded from DynamoDB");

    // 2. Deserialize the Manifest
    let manifest: Manifest = serde_json::from_str(&pipeline_json)
        .map_err(|e| Error::from(format!("Failed to parse manifest: {e}")))?;

    // 3. Build actions from manifest via Factory
    let factory = ActionFactory::new();
    let actions = factory.build_actions(&manifest);

    // 4. Execute the pipeline
    let runner = PipelineRunner::new(actions);

    // The ingestion step populates the LazyFrame — start with an empty context
    let context = RosterContext::new(polars::prelude::LazyFrame::default());

    match runner.run(context) {
        Ok(result) => {
            // Attempt to get row count from the result
            let rows = result
                .data
                .clone()
                .collect()
                .map(|df| df.height())
                .ok();

            tracing::info!(
                %organization_id,
                rows_processed = ?rows,
                "Pipeline completed successfully"
            );

            Ok(PipelineResult {
                organization_id: organization_id.clone(),
                status: "success".to_string(),
                rows_processed: rows,
                error: None,
            })
        }
        Err(e) => {
            tracing::error!(%organization_id, error = %e, "Pipeline failed");

            Ok(PipelineResult {
                organization_id: organization_id.clone(),
                status: "error".to_string(),
                rows_processed: None,
                error: Some(e.to_string()),
            })
        }
    }
}

async fn fetch_config(
    dynamo: &aws_sdk_dynamodb::Client,
    table_name: &str,
    organization_id: &str,
) -> Result<std::collections::HashMap<String, AttributeValue>, Error> {
    let result = dynamo
        .get_item()
        .table_name(table_name)
        .key(
            "organizationId",
            AttributeValue::S(organization_id.to_string()),
        )
        .send()
        .await
        .map_err(|e| Error::from(format!("DynamoDB get_item failed: {e}")))?;

    result
        .item
        .ok_or_else(|| Error::from(format!("No config found for org: {organization_id}")))
}
