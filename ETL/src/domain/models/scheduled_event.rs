use serde::{Deserialize, Serialize};

/// Domain model for scheduled events, primarily from EventBridge Scheduler.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "eventType", content = "payload")]
pub enum ScheduledEvent {
    //infer it based on the event type field in the payload
    #[serde(rename = "ScheduledEtlEvent")]
    Etl(ScheduledEtlEvent),
    #[serde(rename = "ScheduledDynamicApiEvent")]
    DynamicApi(ScheduledDynamicApiEvent),
}

/// Event payload from EventBridge Scheduler.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledEtlEvent {
    pub event_type: String,
    pub organization_id: String,
    pub customer_company_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]

pub struct ScheduledDynamicApiEvent {
    pub organization_id: String,
    pub customer_company_id: String,
    pub event_type: String,
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
            ScheduledEvent::DynamicApi(ScheduledDynamicApiEvent {
                event_type: "ScheduledDynamicApiEvent".to_string(),
                organization_id: "org789".to_string(),
                customer_company_id: "comp012".to_string(),
            }),
        ];

        for expected  in expected_types {
            let deserialized: ScheduledEvent =
                serde_json::from_str(&serde_json::to_string(&expected).unwrap()).unwrap();

            let res = match (&expected, &deserialized) {
                (ScheduledEvent::Etl(_), ScheduledEvent::Etl(_)) => true,
                (
                    ScheduledEvent::DynamicApi(_),
                    ScheduledEvent::DynamicApi(_),
                ) => true,
                _ => false,
            };
            assert!(res, "Expected and deserialized types do not match");
            assert_eq!(format!("{:?}", deserialized), format!("{:?}", expected));
        }
    }
}
