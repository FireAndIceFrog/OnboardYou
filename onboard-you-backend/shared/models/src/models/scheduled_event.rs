use serde::{Deserialize, Serialize};

/// Domain model for scheduled events, primarily from EventBridge Scheduler.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "eventType", content = "payload")]
pub enum ScheduledEvent {
    //infer it based on the event type field in the payload
    #[serde(rename = "ScheduledEtlEvent")]
    Etl(ScheduledEtlEvent),

    /// Trigger async AI plan generation for a pipeline config.
    #[serde(rename = "GeneratePlanEvent")]
    GeneratePlan(GeneratePlanEvent),
}

/// Event payload from EventBridge Scheduler.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledEtlEvent {
    pub event_type: String,
    pub organization_id: String,
    pub customer_company_id: String,
}

/// Event payload for async plan generation.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePlanEvent {
    pub organization_id: String,
    pub customer_company_id: String,
    /// Source system name — "Workday" or "CSV"
    pub source_system: String,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_infered_properly() {
        let expected_types = vec![
            ScheduledEvent::Etl(ScheduledEtlEvent {
                event_type: "ScheduledEtlEvent".to_string(),
                organization_id: "org123".to_string(),
                customer_company_id: "comp456".to_string(),
            }),
            ScheduledEvent::GeneratePlan(GeneratePlanEvent {
                organization_id: "org123".to_string(),
                customer_company_id: "comp456".to_string(),
                source_system: "Workday".to_string(),
            }),
        ];

        for expected in expected_types {
            let deserialized: ScheduledEvent =
                serde_json::from_str(&serde_json::to_string(&expected).unwrap()).unwrap();

            let res = match (&expected, &deserialized) {
                (ScheduledEvent::Etl(_), ScheduledEvent::Etl(_)) => true,
                (ScheduledEvent::GeneratePlan(_), ScheduledEvent::GeneratePlan(_)) => true,
                _ => false,
            };
            assert!(res, "Expected and deserialized types do not match");
            assert_eq!(format!("{:?}", deserialized), format!("{:?}", expected));
        }
    }

    #[test]
    fn test_generate_plan_event_round_trips() {
        let event = GeneratePlanEvent {
            organization_id: "org-1".into(),
            customer_company_id: "comp-1".into(),
            source_system: "CSV".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let back: GeneratePlanEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.organization_id, "org-1");
        assert_eq!(back.source_system, "CSV");
    }
}
