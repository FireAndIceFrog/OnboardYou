//! ETL Trigger Lambda
//!
//! Bootstrap only. Read engine/pipeline_engine.rs for what the pipeline does.

mod dependancies;
mod engine;
mod models;
mod repositories;

use std::sync::Arc;

use lambda_runtime::{service_fn, Error, LambdaEvent};
use models::ScheduleEvent;
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

    lambda_runtime::run(service_fn(|event: LambdaEvent<ScheduleEvent>| {
        let deps = deps.clone();
        async move {
            engine::pipeline_engine::run(
                deps,
                &event.payload.organization_id,
                &event.payload.customer_company_id,
            )
            .await
        }
    }))
    .await
}
