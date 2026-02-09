//! EventBridge Scheduler management
//!
//! Creates or updates a per-organization schedule that invokes the ETL Lambda
//! on the cron defined in the pipeline config.

use aws_sdk_scheduler::types::{FlexibleTimeWindow, FlexibleTimeWindowMode, Target};
use lambda_http::Error;

use crate::config::{AppState, PipelineConfig};

/// Schedule name convention: onboardyou-{organizationId}
fn schedule_name(organization_id: &str) -> String {
    format!("onboardyou-{organization_id}")
}

/// Create or update an EventBridge Scheduler schedule for a given pipeline config.
///
/// The schedule invokes the ETL trigger Lambda with the organizationId as input.
pub async fn upsert_schedule(state: &AppState, config: &PipelineConfig) -> Result<(), Error> {
    let name = schedule_name(&config.organization_id);

    // The payload sent to the ETL trigger Lambda on each invocation
    let input_payload = serde_json::json!({
        "organizationId": config.organization_id
    })
    .to_string();

    let target = Target::builder()
        .arn(&state.etl_lambda_arn)
        .role_arn(&state.scheduler_role_arn)
        .input(input_payload)
        .build()
        .map_err(|e| Error::from(format!("Failed to build Target: {e}")))?;

    let flex_window = FlexibleTimeWindow::builder()
        .mode(FlexibleTimeWindowMode::Off)
        .build()
        .map_err(|e| Error::from(format!("Failed to build FlexibleTimeWindow: {e}")))?;

    // Try to update first; if the schedule doesn't exist, create it
    let update_result = state
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
            // Schedule doesn't exist — create it
            state
                .scheduler
                .create_schedule()
                .name(&name)
                .schedule_expression(&config.cron)
                .target(target)
                .flexible_time_window(flex_window)
                .send()
                .await
                .map_err(|e| Error::from(format!("Failed to create schedule: {e}")))?;

            tracing::info!(schedule = %name, cron = %config.cron, "Schedule created");
        }
    }

    Ok(())
}

/// Delete an organization's schedule (useful for cleanup / deactivation).
pub async fn delete_schedule(state: &AppState, organization_id: &str) -> Result<(), Error> {
    let name = schedule_name(organization_id);

    state
        .scheduler
        .delete_schedule()
        .name(&name)
        .send()
        .await
        .map_err(|e| Error::from(format!("Failed to delete schedule: {e}")))?;

    tracing::info!(schedule = %name, "Schedule deleted");
    Ok(())
}
