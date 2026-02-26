//! Schedule repository — trait + EventBridge Scheduler implementation.
//!
//! Creates or updates a per-organization schedule that invokes the ETL Lambda
//! on the cron defined in the pipeline config.

use async_trait::async_trait;
use aws_sdk_eventbridge::types::PutEventsRequestEntry;
use aws_sdk_scheduler::types::{FlexibleTimeWindow, FlexibleTimeWindowMode, Target};

use crate::{dependancies::Env, models::ApiError};
use onboard_you::{PipelineConfig, ScheduledDynamicApiEvent, ScheduledEtlEvent, ScheduledEvent};

/// Abstract schedule management for pipeline configs.
#[async_trait]
pub trait ScheduleRepo: Send + Sync {
    async fn upsert_schedule(&self, config: &PipelineConfig) -> Result<(), ApiError>;

    async fn delete_schedule(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<(), ApiError>;

    async fn trigger_dynamic_api_event(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<(), ApiError>;
}

/// EventBridge Scheduler backed implementation.
pub struct EventBridgeScheduleRepo {
    pub scheduler: aws_sdk_scheduler::Client,
    pub eventbridge: aws_sdk_eventbridge::Client,
    pub env: Env,
}

/// Schedule name convention: onboardyou-{organizationId}-{customerCompanyId}
fn schedule_name(organization_id: &str, customer_company_id: &str) -> String {
    format!("onboardyou-{organization_id}-{customer_company_id}")
}

#[async_trait]
impl ScheduleRepo for EventBridgeScheduleRepo {
    async fn trigger_dynamic_api_event(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<(), ApiError> {
        let input_payload = serde_json::to_string(&ScheduledEvent::DynamicApi(
            ScheduledDynamicApiEvent {
                event_type: "ScheduledDynamicApiEvent".to_string(),
                organization_id: organization_id.to_string(),
                customer_company_id: customer_company_id.to_string(),
            },
        ))
        .map_err(|e| ApiError::Repository(format!("Failed to serialize event payload: {e}")))?;

        let event = PutEventsRequestEntry::builder()
            .source("onboardyou.scheduler")
            .detail_type("ScheduledDynamicApiEvent")
            .detail(input_payload)
            .event_bus_name(self.env.dynamic_api_event_stream_name.clone())
            .build();

        self.eventbridge
            .put_events()
            .entries(event)
            .send()
            .await
            .map_err(|e| ApiError::Repository(format!("Failed to put event: {e}")))?;

        tracing::info!(
            organization_id = %organization_id,
            customer_company_id = %customer_company_id,
            "Dynamic API event triggered"
        );
        Ok(())
    }

    async fn upsert_schedule(&self, config: &PipelineConfig) -> Result<(), ApiError> {
        let name = schedule_name(&config.organization_id, &config.customer_company_id);

        let input_payload = serde_json::to_string(&ScheduledEvent::Etl(ScheduledEtlEvent {
            event_type: "ScheduledEtlEvent".to_string(),
            organization_id: config.organization_id.clone(),
            customer_company_id: config.customer_company_id.clone(),
        }))
        .map_err(|e| ApiError::Repository(format!("Failed to serialize event payload: {e}")))?;

        let target = Target::builder()
            .arn(&self.env.etl_lambda_arn)
            .role_arn(&self.env.scheduler_role_arn)
            .input(input_payload)
            .build()
            .map_err(|e| ApiError::Repository(format!("Failed to build Target: {e}")))?;

        let flex_window = FlexibleTimeWindow::builder()
            .mode(FlexibleTimeWindowMode::Off)
            .build()
            .map_err(|e| {
                ApiError::Repository(format!("Failed to build FlexibleTimeWindow: {e}"))
            })?;

        // Try update first; fall back to create if the schedule doesn't exist yet
        let update_result = self
            .scheduler
            .update_schedule()
            .name(&name)
            .schedule_expression(&config.cron)
            .target(target.clone())
            .flexible_time_window(flex_window.clone())
            .send()
            .await;

        match update_result {
            Ok(_) => {
                tracing::info!(schedule = %name, cron = %config.cron, "Schedule updated");
            }
            Err(_) => {
                self.scheduler
                    .create_schedule()
                    .name(&name)
                    .schedule_expression(&config.cron)
                    .target(target)
                    .flexible_time_window(flex_window)
                    .send()
                    .await
                    .map_err(|e| ApiError::Repository(format!("Failed to create schedule: {e}")))?;

                tracing::info!(schedule = %name, cron = %config.cron, "Schedule created");
            }
        }

        Ok(())
    }

    async fn delete_schedule(
        &self,
        organization_id: &str,
        customer_company_id: &str,
    ) -> Result<(), ApiError> {
        let name = schedule_name(organization_id, customer_company_id);

        self.scheduler
            .delete_schedule()
            .name(&name)
            .send()
            .await
            .map_err(|e| ApiError::Repository(format!("Failed to delete schedule: {e}")))?;

        tracing::info!(schedule = %name, "Schedule deleted");
        Ok(())
    }
}
