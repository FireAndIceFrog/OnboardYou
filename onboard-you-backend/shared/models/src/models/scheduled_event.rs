use serde::{Deserialize, Serialize};

/// Domain model for scheduled events, primarily from EventBridge Scheduler.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "eventType", content = "payload")]
pub enum ScheduledEvent {
    //infer it based on the event type field in the payload
    #[serde(rename = "ScheduledEtlEvent")]
    Etl(ScheduledEtlEvent),
}

/// Event payload from EventBridge Scheduler.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledEtlEvent {
    pub event_type: String,
    pub organization_id: String,
    pub customer_company_id: String,
    /// Override filename supplied by the email-ingestor Lambda.
    ///
    /// When set, the pipeline engine uses this timestamped S3 filename instead
    /// of whatever is stored in the `EmailIngestionConnector` config.  For
    /// scheduled and manually triggered runs this is `None`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub filename_override: Option<String>,
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
                filename_override: None,
            }),
        ];

        for expected in expected_types {
            let deserialized: ScheduledEvent =
                serde_json::from_str(&serde_json::to_string(&expected).unwrap()).unwrap();

            let res = match (&expected, &deserialized) {
                (ScheduledEvent::Etl(_), ScheduledEvent::Etl(_)) => true
            };
            assert!(res, "Expected and deserialized types do not match");
            assert_eq!(format!("{:?}", deserialized), format!("{:?}", expected));
        }
    }
}
