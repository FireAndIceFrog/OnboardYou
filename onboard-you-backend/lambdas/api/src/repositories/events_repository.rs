//! SQS-backed event publishing for async workflows (e.g. plan generation).

use async_trait::async_trait;

use crate::models::ApiError;

pub struct SqsEventsRepository {
    pub sqs: aws_sdk_sqs::Client,
    pub queue_url: String,
}

#[async_trait]
pub trait EventsRepo: Send + Sync {
    /// Publish a serialised event to the queue.
    async fn publish(&self, message_body: &str) -> Result<(), ApiError>;
}

#[async_trait]
impl EventsRepo for SqsEventsRepository {
    async fn publish(&self, message_body: &str) -> Result<(), ApiError> {
        self.sqs
            .send_message()
            .queue_url(&self.queue_url)
            .message_body(message_body)
            .send()
            .await
            .map_err(|e| ApiError::Repository(format!("Failed to send SQS message: {e}")))?;

        Ok(())
    }
}
