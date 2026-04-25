//! Trigger repository — sends ETL run events via SQS.

use async_trait::async_trait;

use crate::models::ApiError;
use onboard_you_models::{ScheduledEtlEvent, ScheduledEvent};

/// Abstract trigger mechanism for pipeline runs.
#[async_trait]
pub trait TriggerRepo: Send + Sync {
    async fn trigger_run(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<(), ApiError>;
}

/// SQS-backed implementation — sends a message to the ETL events queue.
pub struct SqsTriggerRepo {
    pub sqs: aws_sdk_sqs::Client,
    pub queue_url: String,
}

#[async_trait]
impl TriggerRepo for SqsTriggerRepo {
    async fn trigger_run(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<(), ApiError> {
        let event = ScheduledEvent::Etl(ScheduledEtlEvent {
            event_type: "ScheduledEtlEvent".to_string(),
            organization_id: organization_id.to_string(),
            customer_company_id: customer_company_id.to_string(),
            filename_override: None,
        });

        let body = serde_json::to_string(&event)
            .map_err(|e| ApiError::Repository(format!("Failed to serialize trigger event: {e}")))?;

        self.sqs
            .send_message()
            .queue_url(&self.queue_url)
            .message_body(body)
            .send()
            .await
            .map_err(|e| ApiError::Repository(format!("Failed to send SQS message: {e}")))?;

        tracing::info!(
            organization_id = %organization_id,
            customer_company_id = %customer_company_id,
            "Triggered pipeline run via SQS"
        );

        Ok(())
    }
}
