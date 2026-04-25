use async_trait::async_trait;
use aws_sdk_sqs::Client as SqsClient;
use onboard_you_models::{ScheduledEtlEvent, ScheduledEvent};
use std::sync::Arc;

/// Abstraction over the ETL SQS queue.
#[async_trait]
pub trait ISqsRepo: Send + Sync {
    async fn enqueue_etl_event(
        &self,
        queue_url: &str,
        event: &ScheduledEtlEvent,
    ) -> Result<(), String>;
}

/// AWS SQS-backed implementation of [`ISqsRepo`].
pub struct SqsRepository {
    sqs: SqsClient,
}

impl SqsRepository {
    pub fn new(sqs: SqsClient) -> Arc<Self> {
        Arc::new(Self { sqs })
    }
}

#[async_trait]
impl ISqsRepo for SqsRepository {
    async fn enqueue_etl_event(
        &self,
        queue_url: &str,
        event: &ScheduledEtlEvent,
    ) -> Result<(), String> {
        let envelope = ScheduledEvent::Etl(event.clone());

        let body = serde_json::to_string(&envelope)
            .map_err(|e| format!("Failed to serialize ScheduledEtlEvent: {e}"))?;

        self.sqs
            .send_message()
            .queue_url(queue_url)
            .message_body(body)
            .send()
            .await
            .map_err(|e| format!("SQS SendMessage failed: {e}"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use onboard_you_models::{ScheduledEtlEvent, ScheduledEvent};

    /// Verify the SQS message body serializes to the envelope shape that etl-trigger expects.
    #[test]
    fn event_body_has_correct_envelope_shape() {
        let event = ScheduledEtlEvent {
            event_type: "ScheduledEtlEvent".into(),
            organization_id: "org-1".into(),
            customer_company_id: "company-1".into(),
            filename_override: Some("roster_20250425T120000Z.csv".into()),
        };

        let envelope = ScheduledEvent::Etl(event);
        let body = serde_json::to_string(&envelope).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();

        assert_eq!(parsed["eventType"], "ScheduledEtlEvent");
        assert_eq!(parsed["payload"]["organizationId"], "org-1");
        assert_eq!(parsed["payload"]["customerCompanyId"], "company-1");
        assert_eq!(
            parsed["payload"]["filenameOverride"],
            "roster_20250425T120000Z.csv"
        );
    }
}