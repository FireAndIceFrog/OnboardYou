//! SQS envelope model — parses the Lambda invocation payload into
//! `ScheduledEvent`s regardless of whether the invocation came from
//! SQS (batch of records) or EventBridge Scheduler (direct).

use lambda_runtime::Error;
use onboard_you_models::ScheduledEvent;
use serde::Deserialize;

/// A single SQS record containing a JSON body.
#[derive(Debug, Deserialize)]
struct SqsRecord {
    body: String,
}

/// Minimal SQS batch envelope — only the fields we need.
#[derive(Debug, Deserialize)]
struct SqsBatch {
    #[serde(rename = "Records")]
    records: Vec<SqsRecord>,
}

/// Parse a raw Lambda payload into one or more `ScheduledEvent`s.
///
/// Handles two invocation shapes:
/// - **SQS** → `{ "Records": [{ "body": "…" }, …] }`
/// - **EventBridge Scheduler** → direct `ScheduledEvent` JSON
pub fn parse_events(raw: serde_json::Value) -> Result<Vec<ScheduledEvent>, Error> {
    if raw.get("Records").is_some() {
        parse_sqs_batch(raw)
    } else {
        parse_direct(raw)
    }
}

/// Extract `ScheduledEvent`s from an SQS batch envelope.
fn parse_sqs_batch(raw: serde_json::Value) -> Result<Vec<ScheduledEvent>, Error> {
    let batch: SqsBatch = serde_json::from_value(raw).map_err(|e| {
        tracing::error!(error = %e, "Failed to deserialize SQS envelope");
        Error::from(format!("Failed to deserialize SQS envelope: {e}"))
    })?;

    tracing::info!(record_count = batch.records.len(), "Processing SQS batch");

    batch
        .records
        .into_iter()
        .enumerate()
        .map(|(i, record)| {
            tracing::info!(index = i, body = %record.body, "Extracted SQS message body");

            serde_json::from_str::<ScheduledEvent>(&record.body).map_err(|e| {
                tracing::error!(index = i, body = %record.body, error = %e, "Failed to deserialize SQS body");
                Error::from(format!("Failed to deserialize SQS record[{i}]: {e}"))
            })
        })
        .collect()
}

/// Parse a direct EventBridge Scheduler invocation payload.
fn parse_direct(raw: serde_json::Value) -> Result<Vec<ScheduledEvent>, Error> {
    tracing::info!("Direct invocation detected (EventBridge Scheduler)");

    let event: ScheduledEvent = serde_json::from_value(raw.clone()).map_err(|e| {
        tracing::error!(payload = %raw, error = %e, "Failed to deserialize direct payload");
        Error::from(format!("Failed to deserialize direct payload: {e}"))
    })?;

    Ok(vec![event])
}
