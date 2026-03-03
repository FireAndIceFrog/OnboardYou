//! Plan engine — triggers async AI plan generation via SQS.
//!
//! `POST /config/{id}/generate-plan`:
//!   1. Sets `plan_summary.generation_status = InProgress` on the config
//!   2. Sends a `GeneratePlanEvent` to SQS
//!   3. Returns 202 Accepted immediately
//!
//! The etl-trigger lambda picks up the SQS message, calls `gh_models`,
//! and writes the result back to DynamoDB.

use crate::{dependancies::Dependancies, models::ApiError};
use onboard_you_models::{GeneratePlanEvent, PlanSummary, SchemaGenerationStatus, ScheduledEvent};

/// Trigger plan generation for a pipeline config.
///
/// Returns `Ok(())` if the SQS message was sent successfully.
/// Returns `Ok(())` immediately (idempotent) if generation is already in progress.
pub async fn generate_plan(
    deps: &Dependancies,
    organization_id: &str,
    customer_company_id: &str,
    source_system: &str,
) -> Result<(), ApiError> {
    // 1. Fetch the current config
    let mut config = deps
        .config_repo
        .get(organization_id, customer_company_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("{organization_id}/{customer_company_id}"))
        })?;

    // Idempotency: if already in progress, return immediately
    if let Some(ref summary) = config.plan_summary {
        if matches!(summary.generation_status, SchemaGenerationStatus::InProgress) {
            tracing::info!(
                %organization_id,
                %customer_company_id,
                "Plan generation already in progress — returning early"
            );
            return Ok(());
        }
    }

    // 2. Set status to InProgress and persist
    config.plan_summary = Some(PlanSummary {
        headline: String::new(),
        description: String::new(),
        features: vec![],
        preview: onboard_you_models::PlanPreview {
            source_label: String::new(),
            target_label: String::new(),
            before: Default::default(),
            after: Default::default(),
        },
        generation_status: SchemaGenerationStatus::InProgress,
    });
    deps.config_repo.put(&config).await?;

    // 3. Send SQS message
    let event = ScheduledEvent::GeneratePlan(GeneratePlanEvent {
        organization_id: organization_id.to_string(),
        customer_company_id: customer_company_id.to_string(),
        source_system: source_system.to_string(),
    });

    let message_body = serde_json::to_string(&event)
        .map_err(|e| ApiError::Repository(format!("Failed to serialize SQS message: {e}")))?;

    let events_repo = deps.events_repo.as_ref().ok_or_else(|| {
        ApiError::Repository("Events repository not configured (SQS_QUEUE_URL missing)".to_string())
    })?;

    events_repo.publish(&message_body).await?;

    tracing::info!(
        %organization_id,
        %customer_company_id,
        "Plan generation triggered — SQS message sent"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_plan_event_serializes_correctly() {
        let event = ScheduledEvent::GeneratePlan(GeneratePlanEvent {
            organization_id: "org-1".into(),
            customer_company_id: "comp-1".into(),
            source_system: "Workday".into(),
        });

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("GeneratePlanEvent"));
        assert!(json.contains("org-1"));
        assert!(json.contains("Workday"));

        // Can deserialize back
        let back: ScheduledEvent = serde_json::from_str(&json).unwrap();
        match back {
            ScheduledEvent::GeneratePlan(e) => {
                assert_eq!(e.organization_id, "org-1");
                assert_eq!(e.source_system, "Workday");
            }
            _ => panic!("wrong variant"),
        }
    }
}
