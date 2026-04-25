use std::sync::Arc;
use onboard_you_models::ScheduledEtlEvent;
use crate::dependancies::Dependancies;
use crate::engine::{filename, mime_parser};

/// Process a raw inbound email:
///
/// 1. Parse MIME → sender / subject / best attachment.
/// 2. Look up `EmailRoutes` DynamoDB table → org + company.
/// 3. Generate a timestamped S3 key and upload the attachment.
/// 4. If not CSV → run Textract and upload the converted CSV instead.
/// 5. Enqueue a `ScheduledEtlEvent` for the `etl-trigger` Lambda.
pub async fn run(deps: &Arc<Dependancies>, raw_email_bytes: Vec<u8>) -> Result<(), String> {
    // 1. Parse MIME
    let email = mime_parser::parse(&raw_email_bytes)?;

    tracing::info!(sender = %email.sender, subject = %email.subject, "Parsed email");

    // 2. Route lookup
    let route = deps
        .email_route_repo
        .lookup(&deps.env.email_routes_table, &email.sender)
        .await?
        .ok_or_else(|| format!("No route found for sender: {}", email.sender))?;

    tracing::info!(
        org_id = %route.org_id,
        company_id = %route.company_id,
        "Found email route"
    );

    // 3. Extract best attachment (CSV/spreadsheet preferred; image if nothing better)
    let (original_filename, attachment_bytes) = email
        .attachment
        .ok_or_else(|| "No attachments found in email".to_string())?;

    tracing::info!(%original_filename, bytes = attachment_bytes.len(), "Found attachment");

    // 4. Upload original to CSV_UPLOAD_BUCKET with timestamped key
    let stamped_name = filename::timestamped(&original_filename);
    let upload_key = format!("{}/{}/{}", route.org_id, route.company_id, stamped_name);

    deps.s3_repo
        .upload(
            &deps.env.csv_upload_bucket,
            &upload_key,
            attachment_bytes.clone(),
            "application/octet-stream",
        )
        .await?;

    tracing::info!(%upload_key, "Uploaded attachment to CSV bucket");

    // 5. Determine the final CSV filename
    let csv_filename = if filename::is_csv(&stamped_name) {
        stamped_name.clone()
    } else {
        let csv_bytes = deps
            .textract_repo
            .convert_to_csv(&deps.env.ses_inbox_bucket, &upload_key)
            .await?;
        let stem = filename::stem_of(&stamped_name);
        let csv_key = format!("{}/{}/{}.csv", route.org_id, route.company_id, stem);

        deps.s3_repo
            .upload(&deps.env.csv_upload_bucket, &csv_key, csv_bytes, "text/csv")
            .await?;

        tracing::info!(%csv_key, "Uploaded converted CSV to CSV bucket");
        format!("{stem}.csv")
    };

    // 6. Enqueue ETL trigger
    let etl_event = ScheduledEtlEvent {
        event_type: "ScheduledEtlEvent".into(),
        organization_id: route.org_id.clone(),
        customer_company_id: route.company_id.clone(),
        filename_override: Some(csv_filename.clone()),
    };

    deps.sqs_repo
        .enqueue_etl_event(&deps.env.etl_sqs_queue_url, &etl_event)
        .await?;

    tracing::info!(
        org_id = %route.org_id,
        company_id = %route.company_id,
        %csv_filename,
        "ETL event enqueued"
    );

    Ok(())
}

