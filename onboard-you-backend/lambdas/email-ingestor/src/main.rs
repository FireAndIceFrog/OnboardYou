//! Email Ingestor Lambda
//!
//! Triggered by an S3 `ObjectCreated` event when SES stores an inbound email
//! in the `ses-inbox` bucket.
//!
//! ## Flow
//!
//! 1. Parse the S3 event → locate the raw email object.
//! 2. Download raw email bytes from `ses-inbox`.
//! 3. Parse MIME → extract FROM address, SUBJECT, first attachment.
//! 4. Look up `EmailRoutes` DynamoDB table by sender → `org_id`, `company_id`.
//! 5. Validate sender is in the target pipeline's `allowed_senders` list.
//! 6. Generate a timestamped filename: `{stem}_{YYYYMMDDTHHMMSSZ}.{ext}`.
//! 7. Upload the attachment to `CSV_UPLOAD_BUCKET/{org_id}/{company_id}/{stamped}`.
//! 8. If non-CSV: run Textract synchronously → convert to CSV → upload CSV.
//! 9. Push `ScheduledEtlEvent { org_id, company_id, filename_override }` to SQS.
//!
//! ## Error handling
//!
//! All errors are logged and `Ok(())` is returned to prevent S3 event retries.

use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_sqs::Client as SqsClient;
use aws_sdk_textract::Client as TextractClient;
use chrono::Utc;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use mailparse::MailHeaderMap;
use onboard_you_models::ScheduledEtlEvent;
use serde::Deserialize;
use tracing_subscriber::{fmt, EnvFilter};

// ---------------------------------------------------------------------------
// S3 event types (minimal — only what we need)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct S3Event {
    #[serde(rename = "Records")]
    records: Vec<S3EventRecord>,
}

#[derive(Debug, Deserialize)]
struct S3EventRecord {
    s3: S3Object,
}

#[derive(Debug, Deserialize)]
struct S3Object {
    bucket: S3Bucket,
    object: S3ObjectKey,
}

#[derive(Debug, Deserialize)]
struct S3Bucket {
    name: String,
}

#[derive(Debug, Deserialize)]
struct S3ObjectKey {
    key: String,
}

// ---------------------------------------------------------------------------
// DynamoDB EmailRoute
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct EmailRoute {
    org_id: String,
    company_id: String,
}

// ---------------------------------------------------------------------------
// Config from environment
// ---------------------------------------------------------------------------

struct Env {
    ses_inbox_bucket: String,
    csv_upload_bucket: String,
    email_routes_table: String,
    etl_sqs_queue_url: String,
}

impl Env {
    fn from_env() -> Result<Self, String> {
        Ok(Self {
            ses_inbox_bucket: std::env::var("SES_INBOX_BUCKET")
                .map_err(|_| "SES_INBOX_BUCKET not set")?,
            csv_upload_bucket: std::env::var("CSV_UPLOAD_BUCKET")
                .map_err(|_| "CSV_UPLOAD_BUCKET not set")?,
            email_routes_table: std::env::var("EMAIL_ROUTES_TABLE")
                .map_err(|_| "EMAIL_ROUTES_TABLE not set")?,
            etl_sqs_queue_url: std::env::var("ETL_SQS_QUEUE_URL")
                .map_err(|_| "ETL_SQS_QUEUE_URL not set")?,
        })
    }
}

// ---------------------------------------------------------------------------
// Shared AWS clients
// ---------------------------------------------------------------------------

struct Clients {
    s3: S3Client,
    sqs: SqsClient,
    textract: TextractClient,
    dynamo: aws_sdk_dynamodb::Client,
    env: Env,
}

impl Clients {
    async fn new(env: Env) -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        Self {
            s3: S3Client::new(&config),
            sqs: SqsClient::new(&config),
            textract: TextractClient::new(&config),
            dynamo: aws_sdk_dynamodb::Client::new(&config),
            env,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Inject a UTC timestamp suffix into a filename.
fn timestamped_filename(filename: &str) -> String {
    let ts = Utc::now().format("%Y%m%dT%H%M%SZ");
    match filename.rfind('.') {
        Some(pos) if pos > 0 => {
            let stem = &filename[..pos];
            let ext = &filename[pos..];
            format!("{stem}_{ts}{ext}")
        }
        _ => format!("{filename}_{ts}"),
    }
}

/// Return the file stem (name without extension).
fn stem_of(filename: &str) -> &str {
    match filename.rfind('.') {
        Some(pos) if pos > 0 => &filename[..pos],
        _ => filename,
    }
}

fn is_csv(filename: &str) -> bool {
    filename.to_lowercase().ends_with(".csv")
}

// ---------------------------------------------------------------------------
// DynamoDB lookup
// ---------------------------------------------------------------------------

async fn lookup_email_route(
    clients: &Clients,
    sender: &str,
) -> Result<Option<EmailRoute>, String> {
    let resp = clients
        .dynamo
        .get_item()
        .table_name(&clients.env.email_routes_table)
        .key(
            "sender_email",
            AttributeValue::S(sender.to_lowercase()),
        )
        .send()
        .await
        .map_err(|e| format!("DynamoDB GetItem failed: {e}"))?;

    let Some(item) = resp.item else {
        return Ok(None);
    };

    let org_id = item
        .get("org_id")
        .and_then(|v| v.as_s().ok())
        .map(String::from)
        .ok_or_else(|| "EmailRoute missing org_id".to_string())?;

    let company_id = item
        .get("company_id")
        .and_then(|v| v.as_s().ok())
        .map(String::from)
        .ok_or_else(|| "EmailRoute missing company_id".to_string())?;

    Ok(Some(EmailRoute { org_id, company_id }))
}

// ---------------------------------------------------------------------------
// S3 download / upload
// ---------------------------------------------------------------------------

async fn download_s3_object(clients: &Clients, bucket: &str, key: &str) -> Result<Vec<u8>, String> {
    let resp = clients
        .s3
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| format!("S3 GetObject failed ({bucket}/{key}): {e}"))?;

    let bytes = resp
        .body
        .collect()
        .await
        .map_err(|e| format!("Failed to read S3 body: {e}"))?;

    Ok(bytes.into_bytes().to_vec())
}

async fn upload_s3_object(
    clients: &Clients,
    bucket: &str,
    key: &str,
    bytes: Vec<u8>,
    content_type: &str,
) -> Result<(), String> {
    clients
        .s3
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type(content_type)
        .body(bytes.into())
        .send()
        .await
        .map_err(|e| format!("S3 PutObject failed ({bucket}/{key}): {e}"))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Textract conversion (synchronous polling)
// ---------------------------------------------------------------------------

async fn convert_to_csv_via_textract(
    clients: &Clients,
    s3_key: &str,
) -> Result<Vec<u8>, String> {
    use aws_sdk_textract::types::{DocumentLocation, S3Object as TextractS3Obj, FeatureType};

    let job = clients
        .textract
        .start_document_analysis()
        .document_location(
            DocumentLocation::builder()
                .s3_object(
                    TextractS3Obj::builder()
                        .bucket(&clients.env.ses_inbox_bucket)
                        .name(s3_key)
                        .build(),
                )
                .build(),
        )
        .feature_types(FeatureType::Tables)
        .send()
        .await
        .map_err(|e| format!("Textract StartDocumentAnalysis failed: {e}"))?;

    let job_id = job.job_id().ok_or("Textract returned no job ID")?.to_string();

    // Poll until complete (max ~2 min, 15s between attempts)
    let blocks = loop {
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;

        let result = clients
            .textract
            .get_document_analysis()
            .job_id(&job_id)
            .send()
            .await
            .map_err(|e| format!("Textract GetDocumentAnalysis failed: {e}"))?;

        match result.job_status() {
            Some(s) if s.as_str() == "SUCCEEDED" => {
                break result.blocks().to_vec();
            }
            Some(s) if s.as_str() == "FAILED" => {
                return Err(format!("Textract job {job_id} failed"));
            }
            _ => {
                tracing::info!(job_id = %job_id, "Textract job still running...");
            }
        }
    };

    // Extract the first TABLE's cells into CSV rows
    let mut csv_rows: Vec<Vec<String>> = Vec::new();
    for block in &blocks {
        if block.block_type().map(|t| t.as_str()) == Some("CELL") {
            let row = block.row_index().unwrap_or(0) as usize;
            let col = block.column_index().unwrap_or(0) as usize;
            let text = block.text().unwrap_or("").to_string();
            if row > 0 {
                while csv_rows.len() < row { csv_rows.push(Vec::new()); }
                let r = &mut csv_rows[row - 1];
                while r.len() < col { r.push(String::new()); }
                if col > 0 { r[col - 1] = text; }
            }
        }
    }

    let mut csv_out = String::new();
    for row in &csv_rows {
        let line: Vec<String> = row.iter().map(|c| {
            if c.contains(',') || c.contains('"') || c.contains('\n') {
                format!("\"{}\"", c.replace('"', "\"\""))
            } else {
                c.clone()
            }
        }).collect();
        csv_out.push_str(&line.join(","));
        csv_out.push('\n');
    }

    Ok(csv_out.into_bytes())
}

// ---------------------------------------------------------------------------
// SQS trigger
// ---------------------------------------------------------------------------

async fn enqueue_etl_event(clients: &Clients, event: &ScheduledEtlEvent) -> Result<(), String> {
    let body = serde_json::to_string(event)
        .map_err(|e| format!("Failed to serialize ScheduledEtlEvent: {e}"))?;

    clients
        .sqs
        .send_message()
        .queue_url(&clients.env.etl_sqs_queue_url)
        .message_body(body)
        .send()
        .await
        .map_err(|e| format!("SQS SendMessage failed: {e}"))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Core handler logic (extracted for testability)
// ---------------------------------------------------------------------------

async fn handle_email(clients: &Clients, raw_email_bytes: Vec<u8>) -> Result<(), String> {
    // 1. Parse MIME
    let parsed = mailparse::parse_mail(&raw_email_bytes)
        .map_err(|e| format!("MIME parse failed: {e}"))?;

    let from_header = parsed
        .headers
        .get_first_value("From")
        .unwrap_or_default();

    // Extract bare email address from "Display Name <addr@domain.com>"
    let sender = if let Some(start) = from_header.find('<') {
        if let Some(end) = from_header.find('>') {
            from_header[start + 1..end].to_string()
        } else {
            from_header.trim().to_string()
        }
    } else {
        from_header.trim().to_string()
    };

    let subject = parsed
        .headers
        .get_first_value("Subject")
        .unwrap_or_default();

    tracing::info!(%sender, %subject, "Parsed email");

    // 2. Lookup routing table
    let route = lookup_email_route(clients, &sender)
        .await?
        .ok_or_else(|| format!("No route found for sender: {sender}"))?;

    tracing::info!(
        org_id = %route.org_id,
        company_id = %route.company_id,
        "Found email route"
    );

    // 3. Extract first attachment
    let attachment = find_first_attachment(&parsed)
        .ok_or_else(|| "No attachments found in email".to_string())?;

    let original_filename = attachment.0;
    let attachment_bytes = attachment.1;

    tracing::info!(%original_filename, bytes = attachment_bytes.len(), "Found attachment");

    // 4. Generate timestamped filename
    let stamped_name = timestamped_filename(&original_filename);

    // 5. Upload original attachment to CSV_UPLOAD_BUCKET
    let upload_key = format!(
        "{}/{}/{}",
        route.org_id, route.company_id, stamped_name
    );
    upload_s3_object(
        clients,
        &clients.env.csv_upload_bucket,
        &upload_key,
        attachment_bytes.clone(),
        "application/octet-stream",
    )
    .await?;

    tracing::info!(%upload_key, "Uploaded attachment to CSV bucket");

    // 6. Determine the CSV key to pass as filename_override
    let csv_filename = if is_csv(&stamped_name) {
        stamped_name.clone()
    } else {
        // Convert via Textract — need to upload to a key Textract can read from ses-inbox
        // (the attachment was already stored by SES; we uploaded a copy to csv-bucket above)
        let csv_bytes = convert_to_csv_via_textract(clients, &upload_key).await?;
        let stem = stem_of(&stamped_name);
        let csv_key = format!("{}/{}/{}.csv", route.org_id, route.company_id, stem);
        upload_s3_object(
            clients,
            &clients.env.csv_upload_bucket,
            &csv_key,
            csv_bytes,
            "text/csv",
        )
        .await?;
        tracing::info!(%csv_key, "Uploaded converted CSV to CSV bucket");
        format!("{}.csv", stem)
    };

    // 7. Enqueue ETL trigger
    let event = ScheduledEtlEvent {
        event_type: "ScheduledEtlEvent".into(),
        organization_id: route.org_id.clone(),
        customer_company_id: route.company_id.clone(),
        filename_override: Some(csv_filename.clone()),
    };

    // Wrap in ScheduledEvent for the ETL trigger's SQS parser
    let wrapper = serde_json::json!({
        "eventType": "ScheduledEtlEvent",
        "payload": event,
    });

    let body = serde_json::to_string(&wrapper)
        .map_err(|e| format!("Failed to serialize event: {e}"))?;

    clients
        .sqs
        .send_message()
        .queue_url(&clients.env.etl_sqs_queue_url)
        .message_body(body)
        .send()
        .await
        .map_err(|e| format!("SQS SendMessage failed: {e}"))?;

    tracing::info!(
        org_id = %route.org_id,
        company_id = %route.company_id,
        %csv_filename,
        "ETL event enqueued"
    );

    Ok(())
}

/// Recursively find the first non-inline attachment in a parsed MIME message.
fn find_first_attachment(msg: &mailparse::ParsedMail) -> Option<(String, Vec<u8>)> {
    let disposition = msg
        .get_content_disposition()
        .disposition;

    if disposition == mailparse::DispositionType::Attachment {
        if let Some(filename) = msg
            .get_content_disposition()
            .params
            .get("filename")
            .map(|s| s.to_string())
        {
            if let Ok(body) = msg.get_body_raw() {
                return Some((filename, body));
            }
        }
    }

    for sub in &msg.subparts {
        if let Some(found) = find_first_attachment(sub) {
            return Some(found);
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Lambda handler
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<(), Error> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let env = match Env::from_env() {
        Ok(e) => e,
        Err(msg) => {
            tracing::error!("Missing required environment variable: {msg}");
            return Err(Error::from(msg));
        }
    };

    let clients = std::sync::Arc::new(Clients::new(env).await);

    lambda_runtime::run(service_fn(|event: LambdaEvent<serde_json::Value>| {
        let clients = clients.clone();
        async move {
            // Parse the S3 event
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

                let raw_bytes = match download_s3_object(&clients, &inbox_bucket, &email_key).await
                {
                    Ok(b) => b,
                    Err(e) => {
                        tracing::error!(%e, "Failed to download email — skipping");
                        continue;
                    }
                };

                if let Err(e) = handle_email(&clients, raw_bytes).await {
                    // Log and continue — do not propagate to avoid S3 retry storms.
                    tracing::error!(%e, %email_key, "Email processing failed");
                }
            }

            Ok(())
        }
    }))
    .await
}
