//! ETL Trigger Lambda
//!
//! Bootstrap only. Read engine/pipeline_engine.rs for what the pipeline does.

mod dependancies;
mod engine;
mod models;
mod repositories;

use std::sync::Arc;

use lambda_runtime::{service_fn, Error, LambdaEvent};
use onboard_you::ScheduledEvent;
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

    lambda_runtime::run(service_fn(|event: LambdaEvent<ScheduledEvent>| {
        let deps = deps.clone();
        async move {

            match event.payload {
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
                            Ok(())
                        }
                        Err(e) => {
                            tracing::error!("Pipeline execution failed: {e}");
                            Ok(())
                        }
                    }
                },
                ScheduledEvent::DynamicApi(payload) => {
                    tracing::info!("Received Dynamic API event: {:?}", payload);
                    Ok::<(), Error>(())
                },
            }
        }
    }))
    .await
}
