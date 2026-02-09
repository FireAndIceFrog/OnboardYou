//! Config API Lambda
//!
//! Handles POST/PUT /{organizationId}/config
//! - Validates the pipeline config payload
//! - Persists to DynamoDB
//! - Creates/updates an EventBridge Scheduler schedule for the cron trigger

mod config;
mod dynamo;
mod scheduler;

use lambda_http::{run, service_fn, Body, Error, Request, Response};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Error> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let dynamo_client = aws_sdk_dynamodb::Client::new(&aws_config);
    let scheduler_client = aws_sdk_scheduler::Client::new(&aws_config);

    let table_name =
        std::env::var("CONFIG_TABLE_NAME").unwrap_or_else(|_| "PipelineConfigs".to_string());
    let etl_lambda_arn = std::env::var("ETL_LAMBDA_ARN").expect("ETL_LAMBDA_ARN must be set");
    let scheduler_role_arn =
        std::env::var("SCHEDULER_ROLE_ARN").expect("SCHEDULER_ROLE_ARN must be set");

    let state = config::AppState {
        dynamo: dynamo_client,
        scheduler: scheduler_client,
        table_name,
        etl_lambda_arn,
        scheduler_role_arn,
    };

    run(service_fn(|req: Request| {
        let state = state.clone();
        async move { handler(req, &state).await }
    }))
    .await
}

async fn handler(req: Request, state: &config::AppState) -> Result<Response<Body>, Error> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    tracing::info!(%method, %path, "Incoming request");

    // Extract organizationId from path: /{organizationId}/config
    let segments: Vec<&str> = path.trim_matches('/').split('/').collect();

    if segments.len() != 2 || segments[1] != "config" {
        return Ok(Response::builder()
            .status(404)
            .body(Body::from(r#"{"error": "Not found. Use /{organizationId}/config"}"#))
            .unwrap());
    }

    let organization_id = segments[0].to_string();

    match method.as_str() {
        "POST" | "PUT" => {
            let body = std::str::from_utf8(req.body().as_ref())
                .map_err(|e| Error::from(format!("Invalid UTF-8 body: {e}")))?;

            let mut pipeline_config: config::PipelineConfig = serde_json::from_str(body)
                .map_err(|e| Error::from(format!("Invalid JSON: {e}")))?;

            // Enforce the path parameter as the canonical org ID
            pipeline_config.organization_id = organization_id.clone();
            pipeline_config.last_edited = chrono::Utc::now().to_rfc3339();

            // Validate cron expression (basic check)
            if pipeline_config.cron.is_empty() {
                return Ok(Response::builder()
                    .status(400)
                    .body(Body::from(r#"{"error": "cron field is required"}"#))
                    .unwrap());
            }

            // 1. Persist to DynamoDB
            dynamo::put_config(state, &pipeline_config).await?;

            // 2. Create/update EventBridge schedule
            scheduler::upsert_schedule(state, &pipeline_config).await?;

            let response_body = serde_json::to_string(&pipeline_config)?;

            tracing::info!(
                organization_id = %pipeline_config.organization_id,
                "Config saved and schedule updated"
            );

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::from(response_body))
                .unwrap())
        }
        _ => Ok(Response::builder()
            .status(405)
            .body(Body::from(r#"{"error": "Method not allowed"}"#))
            .unwrap()),
    }
}
