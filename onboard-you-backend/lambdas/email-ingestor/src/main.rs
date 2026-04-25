//! Email Ingestor Lambda — bootstrap only.
//!
//! Triggered by an S3 `ObjectCreated` event when SES stores an inbound email
//! in the `ses-inbox` bucket.  See `engine/email_engine.rs` for the processing
//! logic and the module tree below for individual responsibilities.
//!
//! ## Module layout
//!
//! ```text
//! dependancies   — Env (env-vars) + Dependancies (AWS SDK clients)
//! models/
//!   s3_event     — S3Event deserialization types
//!   email_route  — EmailRoute value object
//! repositories/
//!   s3_repository           — download / upload
//!   email_route_repository  — DynamoDB EmailRoutes table lookup
//!   sqs_repository          — SQS enqueue
//! engine/
//!   filename        — timestamped(), stem_of(), is_csv()
//!   mime_parser     — MIME → ParsedEmail (sender/subject/attachment)
//!   textract_engine — Textract job → CSV bytes
//!   email_engine    — orchestrates the full ingest flow
//! ```

mod dependancies;
mod engine;
mod models;
mod repositories;

use std::sync::Arc;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use tracing_subscriber::{fmt, EnvFilter};

use models::s3_event::S3Event;

#[tokio::main]
async fn main() -> Result<(), Error> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let env = dependancies::Env::from_env().map_err(|msg| {
        tracing::error!("Missing required environment variable: {msg}");
        Error::from(msg)
    })?;

    let deps = Arc::new(dependancies::Dependancies::new(env).await);

    lambda_runtime::run(service_fn(|event: LambdaEvent<serde_json::Value>| {
        let deps = deps.clone();
        async move {
            let s3_event: S3Event = match serde_json::from_value(event.payload) {
                Ok(e) => e,
                Err(err) => {
                    tracing::error!(?err, "Failed to parse S3 event — ignoring");
                    return Ok::<(), Error>(());
                }
            };

            for record in s3_event.records {
                let inbox_bucket = record.s3.bucket.name.clone();
                let email_key = record.s3.object.key.clone();

                tracing::info!(%inbox_bucket, %email_key, "Processing inbound email");

                let raw_bytes = match deps.s3_repo.download(&inbox_bucket, &email_key).await {
                    Ok(b) => b,
                    Err(e) => {
                        tracing::error!(%e, "Failed to download email — skipping");
                        continue;
                    }
                };

                if let Err(e) = engine::email_engine::run(&deps, raw_bytes).await {
                    // Log and continue — do not propagate to avoid S3 retry storms.
                    tracing::error!(%e, %email_key, "Email processing failed");
                }
            }

            Ok(())
        }
    }))
    .await
}
