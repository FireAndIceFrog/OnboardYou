//! ETL Trigger Lambda
//!
//! Handles two invocation paths:
//! - **EventBridge Scheduler** → direct invocation with `ScheduledEvent` JSON
//! - **SQS** → `{ "Records": [{ "body": "<ScheduledEvent JSON>" }] }`
//!
//! Read engine/pipeline_engine.rs for what the pipeline does.

mod dependancies;
mod engine;
mod models;
mod repositories;
use std::sync::Arc;

use lambda_runtime::{service_fn, Error, LambdaEvent};
use onboard_you_models::ScheduledEvent;
use tracing_subscriber::{fmt, EnvFilter};

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
            tracing::info!(payload = %event.payload, "Lambda invoked");

            let events = models::parse_events(event.payload)?;

            for scheduled in events {
                match scheduled {
                    ScheduledEvent::Etl(payload) => {
                        tracing::info!(?payload, "Received ETL event");
                        match engine::pipeline_engine::run(
                            deps.clone(),
                            &payload.organization_id,
                            &payload.customer_company_id,
                        )
                        .await {
                            Ok(result) => {
                                tracing::info!("Pipeline executed successfully: {:?}", result);
                            }
                            Err(e) => {
                                tracing::error!("Pipeline execution failed: {e}");
                            }
                        }
                    },
                    ScheduledEvent::GeneratePlan(payload) => {
                        tracing::info!(
                            organization_id = %payload.organization_id,
                            customer_company_id = %payload.customer_company_id,
                            "Starting plan generation"
                        );
                        match engine::plan_generation_engine::run(
                            deps.clone(),
                            &payload.organization_id,
                            &payload.customer_company_id,
                        )
                        .await {
                            Ok(()) => {
                                tracing::info!("Plan generation completed successfully");
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "Plan generation failed");
                            }
                        }
                    },
                }
            }

            Ok::<(), Error>(())
        }
    }))
    .await
}
