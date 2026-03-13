//! ETL Trigger Lambda
//!
//! Bootstrap only. Read engine/pipeline_engine.rs for what the pipeline does.
//!
//! Handles two invocation sources:
//! - **EventBridge Scheduler** — receives `ScheduledEvent` directly.
//! - **SQS** — receives `{ "Records": [{ "body": "<ScheduledEvent JSON>" }] }`.

mod dependancies;
mod engine;
mod models;
mod repositories;
use std::sync::Arc;

use lambda_runtime::{service_fn, Error, LambdaEvent};
use onboard_you_models::ScheduledEvent;
use tracing_subscriber::{fmt, EnvFilter};

/// Extract a [`ScheduledEvent`] from either a direct payload or an SQS wrapper.
fn parse_event(value: serde_json::Value) -> Result<ScheduledEvent, Error> {
    // SQS events have a top-level "Records" array
    if let Some(records) = value.get("Records").and_then(|r| r.as_array()) {
        let body = records
            .first()
            .and_then(|r| r.get("body"))
            .and_then(|b| b.as_str())
            .ok_or("SQS record missing body")?;
        Ok(serde_json::from_str::<ScheduledEvent>(body)?)
    } else {
        Ok(serde_json::from_value::<ScheduledEvent>(value)?)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    // Build environment + dependancies (repositories/clients)
    let env = dependancies::Env::from_env();
    let deps = Arc::new(dependancies::Dependancies::new(env.clone()).await);

    lambda_runtime::run(service_fn(|event: LambdaEvent<serde_json::Value>| {
        let deps = deps.clone();
        async move {
            let scheduled_event = parse_event(event.payload)?;

            match scheduled_event {
                ScheduledEvent::Etl(payload) => {
                    tracing::info!("Received ETL event: {:?}", payload);
                    match engine::pipeline_engine::run(
                        deps,
                        &payload.organization_id,
                        &payload.customer_company_id,
                    )
                    .await {
                        Ok(result) => {
                            tracing::info!("Pipeline executed successfully: {:?}", result);
                            Ok::<(), Error>(())
                        }
                        Err(e) => {
                            tracing::error!("Pipeline execution failed: {e}");
                            Ok::<(), Error>(())
                        }
                    }
                },
            }
        }
    }))
    .await
}
