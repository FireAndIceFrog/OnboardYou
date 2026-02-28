//! Integration tests for the Workday HRIS Connector
//!
//! Verifies factory resolution, config parsing, SOAP envelope generation,
//! XML response parsing, and full pipeline execution with mocked Workday data.

mod common;

use onboard_you::capabilities::ingestion::engine::{
    build_get_workers_envelope, parse_get_workers_response, parse_response_results,
    workers_to_dataframe
};
use ::onboard_you_models::{WorkdayConfig, WorkdayResponseGroup};
use onboard_you::*;
use onboard_you::ActionFactory;
use onboard_you_models::{ActionConfig, ActionConfigPayload, ActionType, RosterContext};
use polars::prelude::*;

// ── Sample Workday XML for integration tests ─────────────────────────────

const WORKDAY_RESPONSE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<env:Envelope xmlns:env="http://schemas.xmlsoap.org/soap/envelope/">
  <env:Body>
    <wd:Get_Workers_Response xmlns:wd="urn:com.workday/bsvc">
      <wd:Response_Results>
        <wd:Total_Results>2</wd:Total_Results>
        <wd:Total_Pages>1</wd:Total_Pages>
        <wd:Page_Results>2</wd:Page_Results>
        <wd:Page>1</wd:Page>
      </wd:Response_Results>
      <wd:Response_Data>
        <wd:Worker>
          <wd:Worker_ID>WK-100</wd:Worker_ID>
          <wd:Employee_ID>EMP-100</wd:Employee_ID>
          <wd:First_Name>Sarah</wd:First_Name>
          <wd:Last_Name>Connor</wd:Last_Name>
          <wd:Email_Address>sarah.connor@acme.com</wd:Email_Address>
          <wd:Phone_Number>+1-555-0200</wd:Phone_Number>
          <wd:Job_Title>Director of Operations</wd:Job_Title>
          <wd:Business_Title>VP Operations</wd:Business_Title>
          <wd:Organization_Name>Operations</wd:Organization_Name>
          <wd:Location_Name>Los Angeles</wd:Location_Name>
          <wd:Hire_Date>2020-03-15</wd:Hire_Date>
          <wd:Worker_Type>Employee</wd:Worker_Type>
          <wd:Status>Active</wd:Status>
          <wd:Manager_ID>MGR-050</wd:Manager_ID>
          <wd:Manager_Name>CEO Boss</wd:Manager_Name>
          <wd:Position_ID>P-500</wd:Position_ID>
          <wd:Compensation_Grade>G10</wd:Compensation_Grade>
          <wd:Pay_Rate_Type>Salary</wd:Pay_Rate_Type>
        </wd:Worker>
        <wd:Worker>
          <wd:Worker_ID>WK-101</wd:Worker_ID>
          <wd:Employee_ID>CW-050</wd:Employee_ID>
          <wd:First_Name>Kyle</wd:First_Name>
          <wd:Last_Name>Reese</wd:Last_Name>
          <wd:Email_Address>kyle.reese@acme.com</wd:Email_Address>
          <wd:Phone_Number>+1-555-0201</wd:Phone_Number>
          <wd:Job_Title>Security Analyst</wd:Job_Title>
          <wd:Business_Title>Contract Security Analyst</wd:Business_Title>
          <wd:Organization_Name>Security</wd:Organization_Name>
          <wd:Location_Name>Chicago</wd:Location_Name>
          <wd:Hire_Date>2024-09-01</wd:Hire_Date>
          <wd:Worker_Type>Contingent_Worker</wd:Worker_Type>
          <wd:Status>Active</wd:Status>
          <wd:Manager_ID>MGR-060</wd:Manager_ID>
          <wd:Manager_Name>CISO Chief</wd:Manager_Name>
          <wd:Position_ID>P-600</wd:Position_ID>
          <wd:Compensation_Grade>G6</wd:Compensation_Grade>
          <wd:Pay_Rate_Type>Hourly</wd:Pay_Rate_Type>
        </wd:Worker>
      </wd:Response_Data>
    </wd:Get_Workers_Response>
  </env:Body>
</env:Envelope>"#;

// ── Factory Resolution Tests ─────────────────────────────────────────────

#[test]
fn test_factory_creates_workday_connector() {
    let config = ActionConfig {
        id: "ingest_workday".into(),
        action_type: ActionType::WorkdayHrisConnector,
        config: ActionConfigPayload::WorkdayHrisConnector(
            serde_json::from_value(serde_json::json!({
                "tenant_url": "https://wd3-impl-services1.workday.com",
                "tenant_id": "integration_test_tenant",
                "username": "ISU_Integration",
                "password": "test_password"
            }))
            .unwrap(),
        ),
    };
    let action = ActionFactory::new()
        .create(&config)
        .expect("factory should create workday connector");
    assert_eq!(action.id(), "workday_hris_connector");
}

#[test]
fn test_factory_workday_with_full_config() {
    let config = ActionConfig {
        id: "ingest_workday_full".into(),
        action_type: ActionType::WorkdayHrisConnector,
        config: ActionConfigPayload::WorkdayHrisConnector(
            serde_json::from_value(serde_json::json!({
                "tenant_url": "https://wd5-impl-services1.workday.com",
                "tenant_id": "checkout_corp",
                "username": "ISU_Onboard",
                "password": "env:WORKDAY_SECRET",
                "worker_count_limit": 500,
                "response_group": {
                    "include_personal_information": true,
                    "include_employment_information": true,
                    "include_compensation": true,
                    "include_organizations": true,
                    "include_roles": true
                }
            }))
            .unwrap(),
        ),
    };
    let action = ActionFactory::new()
        .create(&config)
        .expect("factory should create workday connector");
    assert_eq!(action.id(), "workday_hris_connector");
}

#[test]
fn test_factory_workday_missing_config() {
    // With typed configs, missing required fields are caught at deserialization
    let result: std::result::Result<ActionConfig, _> = serde_json::from_str(
        r#"{"id":"bad_workday","action_type":"workday_hris_connector","config":{}}"#,
    );
    assert!(result.is_err(), "should fail with missing required fields");
}

// ── SOAP Envelope Tests ─────────────────────────────────────────────────

#[test]
fn test_soap_envelope_contains_credentials() {
    let config = WorkdayConfig {
        tenant_url: "https://wd3.workday.com".into(),
        tenant_id: "test_co".into(),
        username: "ISU_Test".into(),
        password: "p".into(),
        worker_count_limit: 100,
        response_group: WorkdayResponseGroup::default(),
    };
    let envelope = build_get_workers_envelope(&config, "my_secret", 1);

    assert!(envelope.contains("ISU_Test@test_co"));
    assert!(envelope.contains("my_secret"));
    assert!(envelope.contains("Get_Workers_Request"));
    assert!(envelope.contains("<bsvc:Page>1</bsvc:Page>"));
    assert!(envelope.contains("v45.2"));
}

// ── XML Parsing Integration Tests ────────────────────────────────────────

#[test]
fn test_parse_workday_response_integration() {
    let records = parse_get_workers_response(WORKDAY_RESPONSE_XML)
        .expect("should parse integration test XML");

    assert_eq!(records.len(), 2);

    // First worker — Employee
    assert_eq!(records[0].worker_id, "WK-100");
    assert_eq!(records[0].first_name, "Sarah");
    assert_eq!(records[0].last_name, "Connor");
    assert_eq!(records[0].worker_type, "Employee");
    assert_eq!(records[0].location, "Los Angeles");

    // Second worker — Contingent Worker
    assert_eq!(records[1].worker_id, "WK-101");
    assert_eq!(records[1].first_name, "Kyle");
    assert_eq!(records[1].worker_type, "Contingent_Worker");
    assert_eq!(records[1].pay_rate_type, "Hourly");
}

#[test]
fn test_workday_workers_to_dataframe_integration() {
    let records = parse_get_workers_response(WORKDAY_RESPONSE_XML).unwrap();
    let df = workers_to_dataframe(&records).unwrap();

    assert_eq!(df.height(), 2);
    assert_eq!(df.width(), 18);

    // Verify column data
    let emails = df.column("email").unwrap();
    let email0 = emails.str().unwrap().get(0).unwrap();
    assert_eq!(email0, "sarah.connor@acme.com");

    let departments = df.column("department").unwrap();
    let dept1 = departments.str().unwrap().get(1).unwrap();
    assert_eq!(dept1, "Security");
}

// ── End-to-End Pipeline with Workday Source ──────────────────────────────

#[test]
fn test_e2e_workday_pipeline_with_scd_type_2() {
    // Parse Workday XML → records → DataFrame → run through SCD Type 2
    let records = parse_get_workers_response(WORKDAY_RESPONSE_XML).unwrap();
    let df = workers_to_dataframe(&records).unwrap();

    // Build a RosterContext as the Workday connector would
    let mut ctx = RosterContext::new(df.lazy());
    for col in [
        "worker_id",
        "employee_id",
        "first_name",
        "last_name",
        "email",
        "phone",
        "job_title",
        "business_title",
        "department",
        "location",
        "hire_date",
        "worker_type",
        "worker_status",
        "manager_id",
        "manager_name",
        "position_id",
        "compensation_grade",
        "pay_rate_type",
    ] {
        ctx.set_field_source(col.to_string(), "WORKDAY_HRIS".into());
    }

    // Verify we can then pass this through the SCD Type 2 action
    let scd_config = ActionConfig {
        id: "scd".into(),
        action_type: ActionType::ScdType2,
        config: ActionConfigPayload::ScdType2(
            serde_json::from_value(serde_json::json!({
                "entity_column": "employee_id",
                "date_column": "hire_date"
            }))
            .unwrap(),
        ),
    };
    let scd_action = ActionFactory::new()
        .create(&scd_config)
        .expect("create scd_type_2");
    let result = scd_action
        .execute(ctx)
        .expect("scd should execute on Workday data");
    let result_df = result.data.collect().expect("collect");

    // SCD Type 2 should add effective_from, effective_to, is_current columns
    assert!(result_df.column("effective_from").is_ok());
    assert!(result_df.column("is_current").is_ok());
    assert_eq!(result_df.height(), 2);
}

#[test]
fn test_e2e_workday_pipeline_with_deduplication() {
    let records = parse_get_workers_response(WORKDAY_RESPONSE_XML).unwrap();
    let df = workers_to_dataframe(&records).unwrap();

    let ctx = RosterContext::new(df.lazy());

    let dedup_config = ActionConfig {
        id: "dedup".into(),
        action_type: ActionType::IdentityDeduplicator,
        config: ActionConfigPayload::IdentityDeduplicator(
            serde_json::from_value(serde_json::json!({ "columns": ["email"] })).unwrap(),
        ),
    };
    let dedup_action = ActionFactory::new()
        .create(&dedup_config)
        .expect("create dedup");
    let result = dedup_action
        .execute(ctx)
        .expect("dedup should succeed on Workday data");
    let result_df = result.data.collect().expect("collect");

    // Both records have unique emails, so neither should be a duplicate
    assert_eq!(result_df.height(), 2);
    let is_dup: Vec<Option<bool>> = result_df
        .column("is_duplicate")
        .unwrap()
        .bool()
        .unwrap()
        .into_iter()
        .collect();
    assert_eq!(is_dup, vec![Some(false), Some(false)]);
}

// ── Pagination Integration Tests ─────────────────────────────────────────

#[test]
fn test_parse_response_results_integration() {
    let rr = parse_response_results(WORKDAY_RESPONSE_XML);
    assert_eq!(rr.total_results, 2);
    assert_eq!(rr.total_pages, 1);
    assert_eq!(rr.page_results, 2);
    assert_eq!(rr.page, 1);
}

#[test]
fn test_envelope_page_parameter_integration() {
    let config = WorkdayConfig {
        tenant_url: "https://wd3.workday.com".into(),
        tenant_id: "page_test".into(),
        username: "ISU".into(),
        password: "pw".into(),
        worker_count_limit: 50,
        response_group: WorkdayResponseGroup::default(),
    };

    let env_p1 = build_get_workers_envelope(&config, "pw", 1);
    assert!(env_p1.contains("<bsvc:Page>1</bsvc:Page>"));
    assert!(env_p1.contains("<bsvc:Count>50</bsvc:Count>"));

    let env_p5 = build_get_workers_envelope(&config, "pw", 5);
    assert!(env_p5.contains("<bsvc:Page>5</bsvc:Page>"));
}
