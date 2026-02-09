//! Schedule repository — EventBridge Scheduler management.
//!
//! Creates or updates a per-organization schedule that invokes the ETL Lambda
//! on the cron defined in the pipeline config.

use aws_sdk_scheduler::types::{FlexibleTimeWindow, FlexibleTimeWindowMode, Target};

use crate::models::{ApiError, AppState, PipelineConfig};

/// Schedule name convention: onboardyou-{organizationId}
fn schedule_name(organization_id: &str) -> String {
    format!("onboardyou-{organization_id}")
}

/// Create or update an EventBridge Scheduler schedule for a given pipeline config.
pub async fn upsert(state: &AppState, config: &PipelineConfig) -> Result<(), ApiError> {
    let name = schedule_name(&config.organization_id);

    let input_payload = serde_json::json!({
        "organizationId": config.organization_id
    })
    .to_string();

    let target = Target::builder()
        .arn(&state.etl_lambda_arn)
        .role_arn(&state.scheduler_role_arn)
        .input(input_payload)
        .build()
        .map_err(|e| ApiError::Repository(format!("Failed to build Target: {e}")))?;

    let flex_window = FlexibleTimeWindow::builder()
        .mode(FlexibleTimeWindowMode::Off)
        .build()
        .map_err(|e| ApiError::Repository(format!("Failed to build FlexibleTimeWindow: {e}")))?;

    // Try update first; fall back to create if the schedule doesn't exist yet
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
            state
                .scheduler
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

/// Delete an organization's schedule.
#[allow(dead_code)]
pub async fn delete(state: &AppState, organization_id: &str) -> Result<(), ApiError> {
    let name = schedule_name(organization_id);

    state
        .scheduler
        .delete_schedule()
        .name(&name)
        .send()
        .await
        .map_err(|e| ApiError::Repository(format!("Failed to delete schedule: {e}")))?;

    tracing::info!(schedule = %name, "Schedule deleted");
    Ok(())
}
