use serde::Deserialize;

/// Event payload from EventBridge Scheduler.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleEvent {
    pub organization_id: String,
}
