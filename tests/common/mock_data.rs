//! Mock CSV and JSON data for testing

/// Sample HRIS data for testing
pub const SAMPLE_HRIS_CSV: &str = r#"employee_id,first_name,last_name,email,ssn,salary,start_date
001,John,Doe,john.doe@example.com,123-45-6789,75000,2024-01-01
002,Jane,Smith,jane.smith@example.com,987-65-4321,85000,2024-02-15
"#;

/// Sample manifest configuration for testing
pub const SAMPLE_MANIFEST_JSON: &str = r#"{
  "version": "1.0",
  "actions": [
    {
      "id": "hris_connector",
      "action_type": "ingestion",
      "config": {}
    },
    {
      "id": "validator",
      "action_type": "validation",
      "config": {}
    }
  ]
}"#;
