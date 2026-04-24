//! WorkdayHrisConnector: Workday SOAP API HRIS data ingestion
//!
//! Connects to Workday's Human Resources SOAP Web Service (WWS v45.2) to fetch
//! worker data via the `Get_Workers` operation. Returns a populated `RosterContext`
//! with every column tagged to `WORKDAY_HRIS` field-ownership metadata for
//! downstream provenance tracking.
//!
//! # Workday API Reference
//!
//! Service: **Human_Resources** (v45.2 / 2025R2)
//!
//! | Operation        | Description                                                      |
//! |------------------|------------------------------------------------------------------|
//! | `Get_Workers`    | Returns public and private information for specified workers     |
//! | `Get_Employee`   | Retrieves employee employment, personal, and compensation data   |
//! | `Get_Organizations` | Returns organizations by type (Company, Cost Center, etc.)    |
//!
//! The connector builds a SOAP envelope for `Get_Workers`, authenticates via
//! WS-Security (username/password), and flattens the XML response into a
//! Polars `LazyFrame` suitable for the onboarding pipeline.

use crate::capabilities::ingestion::traits::HrisConnector;
use onboard_you_models::ColumnCalculator;
use onboard_you_models::{Error, OnboardingAction, Result, RosterContext};
use crate::orchestration::clients::soap_client::{ReqwestSoapClient, SoapClient};
use onboard_you_models::{WORKDAY_API_VERSION, WorkdayConfig};
use polars::prelude::*;

// ───────────────────────────────────────────────────────────────────────────
// SOAP Envelope Builder
// ───────────────────────────────────────────────────────────────────────────

/// Builds the SOAP envelope for the `Get_Workers` operation.
///
/// The envelope uses WS-Security `UsernameToken` authentication and includes
/// the `Response_Filter` element (with `Page` and `Count`) to control
/// pagination, plus a `Response_Group` to select which data sections are returned.
///
/// # Workday Pagination Model
///
/// The `Response_Filter` element controls which page of results to return:
///
/// | Element | Type | Description                                         |
/// |---------|------|-----------------------------------------------------|
/// | `Page`  | u32  | 1-based page number to retrieve                     |
/// | `Count` | u32  | Maximum number of results per page (default 200)    |
///
/// The response contains a `Response_Results` element with:
///
/// | Element         | Description                              |
/// |-----------------|------------------------------------------|
/// | `Total_Results` | Total number of worker records available  |
/// | `Total_Pages`   | Total number of pages                    |
/// | `Page_Results`  | Number of results on this page           |
/// | `Page`          | Current page number                      |
pub fn build_get_workers_envelope(config: &WorkdayConfig, password: &str, page: u32) -> String {
    let rg = &config.response_group;

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<env:Envelope
    xmlns:env="http://schemas.xmlsoap.org/soap/envelope/"
    xmlns:bsvc="urn:com.workday/bsvc"
    xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd">
  <env:Header>
    <wsse:Security env:mustUnderstand="1">
      <wsse:UsernameToken>
        <wsse:Username>{username}@{tenant_id}</wsse:Username>
        <wsse:Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordText">{password}</wsse:Password>
      </wsse:UsernameToken>
    </wsse:Security>
  </env:Header>
  <env:Body>
    <bsvc:Get_Workers_Request bsvc:version="{version}">
      <bsvc:Response_Filter>
        <bsvc:Page>{page}</bsvc:Page>
        <bsvc:Count>{count}</bsvc:Count>
      </bsvc:Response_Filter>
      <bsvc:Response_Group>
        <bsvc:Include_Personal_Information>{include_personal}</bsvc:Include_Personal_Information>
        <bsvc:Include_Employment_Information>{include_employment}</bsvc:Include_Employment_Information>
        <bsvc:Include_Compensation>{include_compensation}</bsvc:Include_Compensation>
        <bsvc:Include_Organizations>{include_orgs}</bsvc:Include_Organizations>
        <bsvc:Include_Roles>{include_roles}</bsvc:Include_Roles>
      </bsvc:Response_Group>
    </bsvc:Get_Workers_Request>
  </env:Body>
</env:Envelope>"#,
        username = config.username,
        tenant_id = config.tenant_id,
        password = password,
        version = WORKDAY_API_VERSION,
        page = page,
        count = config.worker_count_limit,
        include_personal = rg.include_personal_information,
        include_employment = rg.include_employment_information,
        include_compensation = rg.include_compensation,
        include_orgs = rg.include_organizations,
        include_roles = rg.include_roles,
    )
}

// ───────────────────────────────────────────────────────────────────────────
// XML Response Parsing → Flat worker records
// ───────────────────────────────────────────────────────────────────────────

/// A single flattened worker record extracted from Workday's XML response.
///
/// The `Get_Workers` response nests data deeply; this struct represents the
/// denormalised view used to build the Polars DataFrame.
#[derive(Debug, Clone, Default)]
pub struct WorkdayWorkerRecord {
    pub worker_id: String,
    pub employee_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub job_title: String,
    pub business_title: String,
    pub department: String,
    pub location: String,
    pub hire_date: String,
    pub worker_type: String,   // "Employee" | "Contingent_Worker"
    pub worker_status: String, // "Active" | "Terminated" | etc.
    pub manager_id: String,
    pub manager_name: String,
    pub position_id: String,
    pub compensation_grade: String,
    pub pay_rate_type: String,
}

/// Extract text content for a given simple tag name from an XML fragment.
///
/// This is intentionally simple — production XML parsing should use `quick-xml`
/// but for the MVP we do best-effort tag extraction so the connector compiles
/// without a heavy XML dependency.
fn extract_tag(xml: &str, tag: &str) -> String {
    // Look for <bsvc:Tag>...</bsvc:Tag> or <wd:Tag>...</wd:Tag> or <Tag>...</Tag>
    for prefix in &["bsvc:", "wd:", ""] {
        let open = format!("<{}{}>", prefix, tag);
        let close = format!("</{}{}>", prefix, tag);
        if let Some(start) = xml.find(&open) {
            let content_start = start + open.len();
            if let Some(end) = xml[content_start..].find(&close) {
                let value = &xml[content_start..content_start + end];
                // Strip any nested XML — take only text before first '<'
                let text = value.split('<').next().unwrap_or("").trim();
                if !text.is_empty() {
                    return text.to_string();
                }
            }
        }
    }
    String::new()
}

/// Parse the raw XML SOAP response from `Get_Workers` into flat records.
///
/// Each `<bsvc:Worker>` (or `<wd:Worker>`) element is flattened into a
/// [`WorkdayWorkerRecord`].
pub fn parse_get_workers_response(xml: &str) -> Result<Vec<WorkdayWorkerRecord>> {
    let mut records = Vec::new();

    // Split on Worker elements — handles both bsvc: and wd: prefixes
    let worker_tag_variants = ["<bsvc:Worker>", "<wd:Worker>", "<Worker>"];
    let close_variants = ["</bsvc:Worker>", "</wd:Worker>", "</Worker>"];

    let mut remaining = xml;

    loop {
        // Find the next worker block using any prefix variant
        let mut found = None;
        for (open, close) in worker_tag_variants.iter().zip(close_variants.iter()) {
            if let Some(start) = remaining.find(open) {
                let content_start = start + open.len();
                if let Some(end_offset) = remaining[content_start..].find(close) {
                    let block = &remaining[content_start..content_start + end_offset];
                    remaining = &remaining[content_start + end_offset + close.len()..];
                    found = Some(block.to_string());
                    break;
                }
            }
        }

        let block = match found {
            Some(b) => b,
            None => break,
        };

        let record = WorkdayWorkerRecord {
            worker_id: extract_tag(&block, "Worker_ID").or_else(|| extract_tag(&block, "WID")),
            employee_id: extract_tag(&block, "Employee_ID"),
            first_name: extract_tag(&block, "First_Name")
                .or_else(|| extract_tag(&block, "Legal_First_Name")),
            last_name: extract_tag(&block, "Last_Name")
                .or_else(|| extract_tag(&block, "Legal_Last_Name")),
            email: extract_tag(&block, "Email_Address"),
            phone: extract_tag(&block, "Phone_Number")
                .or_else(|| extract_tag(&block, "Formatted_Phone")),
            job_title: extract_tag(&block, "Job_Title"),
            business_title: extract_tag(&block, "Business_Title"),
            department: extract_tag(&block, "Organization_Name")
                .or_else(|| extract_tag(&block, "Department_Name")),
            location: extract_tag(&block, "Location_Name")
                .or_else(|| extract_tag(&block, "Business_Site_Name")),
            hire_date: extract_tag(&block, "Hire_Date")
                .or_else(|| extract_tag(&block, "Original_Hire_Date")),
            worker_type: extract_tag(&block, "Worker_Type"),
            worker_status: extract_tag(&block, "Status")
                .or_else(|| extract_tag(&block, "Active_Status_Description")),
            manager_id: extract_tag(&block, "Manager_ID"),
            manager_name: extract_tag(&block, "Manager_Name"),
            position_id: extract_tag(&block, "Position_ID"),
            compensation_grade: extract_tag(&block, "Compensation_Grade"),
            pay_rate_type: extract_tag(&block, "Pay_Rate_Type"),
        };

        records.push(record);
    }

    Ok(records)
}

// Helper trait so we can chain `or_else` on String
trait StringOrElse {
    fn or_else<F: FnOnce() -> String>(self, f: F) -> Self;
}

impl StringOrElse for String {
    fn or_else<F: FnOnce() -> String>(self, f: F) -> Self {
        if self.is_empty() {
            f()
        } else {
            self
        }
    }
}

/// Defines the column-name ↔ field-accessor mapping in one place.
///
/// Generates:
/// - `WORKDAY_COLUMNS` — the fixed column list for schema / provenance.
/// - `workers_to_dataframe` — columnar extraction that is guaranteed
///   to stay aligned because both the column name and the struct field
///   come from the same macro arm.
macro_rules! workday_fields {
    ( $( ($col:expr, $field:ident) ),* $(,)? ) => {
        /// The fixed set of columns produced by the Workday `Get_Workers` connector.
        const WORKDAY_COLUMNS: &[&str] = &[ $( $col ),* ];

        /// Convert a vector of flat worker records into a Polars DataFrame.
        pub fn workers_to_dataframe(records: &[WorkdayWorkerRecord]) -> Result<DataFrame> {
            $(
                let $field: Vec<&str> = records.iter().map(|r| r.$field.as_str()).collect();
            )*

            let df = DataFrame::new_infer_height(vec![
                $(
                    Column::new($col.into(), &$field),
                )*
            ])
            .map_err(|e| Error::IngestionError(format!("Failed to build DataFrame: {}", e)))?;

            Ok(df)
        }
    };
}

workday_fields!(
    ("worker_id",           worker_id),
    ("employee_id",         employee_id),
    ("first_name",          first_name),
    ("last_name",           last_name),
    ("email",               email),
    ("phone",               phone),
    ("job_title",           job_title),
    ("business_title",      business_title),
    ("department",          department),
    ("location",            location),
    ("hire_date",           hire_date),
    ("worker_type",         worker_type),
    ("worker_status",       worker_status),
    ("manager_id",          manager_id),
    ("manager_name",        manager_name),
    ("position_id",         position_id),
    ("compensation_grade",  compensation_grade),
    ("pay_rate_type",       pay_rate_type),
);

// ───────────────────────────────────────────────────────────────────────────
// Response_Results pagination metadata
// ───────────────────────────────────────────────────────────────────────────

/// Parsed pagination metadata from the `Response_Results` element in a
/// Workday SOAP response.
///
/// The Workday WWS API returns this block alongside every paged Get request:
///
/// ```xml
/// <wd:Response_Results>
///   <wd:Total_Results>1254</wd:Total_Results>
///   <wd:Total_Pages>7</wd:Total_Pages>
///   <wd:Page_Results>200</wd:Page_Results>
///   <wd:Page>1</wd:Page>
/// </wd:Response_Results>
/// ```
#[derive(Debug, Clone)]
pub struct ResponseResults {
    pub total_results: u32,
    pub total_pages: u32,
    pub page_results: u32,
    pub page: u32,
}

/// Extract pagination metadata from a Workday SOAP response XML string.
///
/// Falls back to `{ total_results: 0, total_pages: 1, page_results: 0, page: 1 }`
/// when the tags are absent (e.g. single-page responses from older endpoints).
pub fn parse_response_results(xml: &str) -> ResponseResults {
    let total_results = extract_tag(xml, "Total_Results")
        .parse::<u32>()
        .unwrap_or(0);
    let total_pages = extract_tag(xml, "Total_Pages").parse::<u32>().unwrap_or(1);
    let page_results = extract_tag(xml, "Page_Results").parse::<u32>().unwrap_or(0);
    let page = extract_tag(xml, "Page").parse::<u32>().unwrap_or(1);

    ResponseResults {
        total_results,
        total_pages,
        page_results,
        page,
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Connector
// ───────────────────────────────────────────────────────────────────────────

/// HRIS connector that ingests worker data from the Workday SOAP API.
///
/// Implements both [`HrisConnector`] and [`OnboardingAction`].
///
/// ## Pipeline manifest example
///
/// ```json
/// {
///   "id": "ingest_workday",
///   "action_type": "workday_hris_connector",
///   "config": {
///     "tenant_url": "https://wd3-impl-services1.workday.com",
///     "tenant_id": "acme_corp",
///     "username": "ISU_Onboarding",
///     "password": "env:WORKDAY_PASSWORD",
///     "worker_count_limit": 200,
///     "response_group": {
///       "include_personal_information": true,
///       "include_employment_information": true,
///       "include_compensation": true,
///       "include_organizations": true,
///       "include_roles": false
///     }
///   }
/// }
/// ```
pub struct WorkdayHrisConnector {
    config: WorkdayConfig,
    client: Box<dyn SoapClient>,
}

impl std::fmt::Debug for WorkdayHrisConnector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkdayHrisConnector")
            .field("config", &self.config)
            .finish()
    }
}

impl WorkdayHrisConnector {
    /// Create a new connector with the production HTTP client.
    pub fn new(config: WorkdayConfig) -> Self {
        Self {
            config,
            client: Box::new(ReqwestSoapClient),
        }
    }

    /// Create a connector with a custom [`SoapClient`] (for testing).
    pub fn with_client(config: WorkdayConfig, client: Box<dyn SoapClient>) -> Self {
        Self { config, client }
    }

    /// Construct from a deserialised config.
    pub fn from_action_config(config: &WorkdayConfig) -> Result<Self> {
        Ok(Self::new(config.clone()))
    }

    /// Fetch workers from Workday, paging through all results via
    /// `Response_Filter.Page`.
    ///
    /// Workday pagination is 1-based: the first request sets `Page = 1`,
    /// and `Response_Results.Total_Pages` tells us how many pages exist.
    /// Each page returns up to `worker_count_limit` workers.  All pages
    /// are concatenated into a single `Vec<WorkdayWorkerRecord>`.
    fn fetch_all_workers(&self) -> Result<Vec<WorkdayWorkerRecord>> {
        let password = self.config.resolved_password()?;
        let endpoint = self.config.soap_endpoint();

        let mut all_records: Vec<WorkdayWorkerRecord> = Vec::new();
        let mut current_page: u32 = 1;
        let mut total_pages: u32 = 1; // will be updated after first response

        loop {
            let envelope = build_get_workers_envelope(&self.config, &password, current_page);

            tracing::info!(
                endpoint = %endpoint,
                tenant = %self.config.tenant_id,
                page = current_page,
                total_pages = total_pages,
                "WorkdayHrisConnector: calling Get_Workers page {}/{}",
                current_page,
                total_pages,
            );

            let response_xml = self.client.post_soap(&endpoint, &envelope)?;

            // Parse pagination metadata from Response_Results
            let results_meta = parse_response_results(&response_xml);
            total_pages = results_meta.total_pages;

            tracing::info!(
                total_results = results_meta.total_results,
                total_pages = results_meta.total_pages,
                page_results = results_meta.page_results,
                page = results_meta.page,
                "WorkdayHrisConnector: response metadata"
            );

            // Parse worker records from this page
            let page_records = parse_get_workers_response(&response_xml)?;

            tracing::info!(
                workers = page_records.len(),
                page = current_page,
                "WorkdayHrisConnector: parsed {} workers from page {}",
                page_records.len(),
                current_page,
            );

            all_records.extend(page_records);

            // Move to the next page if there are more
            if current_page >= total_pages {
                break;
            }
            current_page += 1;
        }

        tracing::info!(
            total_workers = all_records.len(),
            total_pages = total_pages,
            "WorkdayHrisConnector: finished pagination — {} workers across {} page(s)",
            all_records.len(),
            total_pages,
        );

        Ok(all_records)
    }
}



impl HrisConnector for WorkdayHrisConnector {
    fn fetch_data(&self) -> Result<LazyFrame> {
        let records = self.fetch_all_workers()?;
        let df = workers_to_dataframe(&records)?;
        Ok(df.lazy())
    }
}

impl ColumnCalculator for WorkdayHrisConnector {
    fn calculate_columns(&self, context: RosterContext) -> Result<RosterContext> {
        // Build an empty DataFrame with the known Workday columns (all Utf8)
        let columns: Vec<Column> = WORKDAY_COLUMNS
            .iter()
            .map(|name| Column::new((*name).into(), Vec::<&str>::new()))
            .collect();
        let empty_df = DataFrame::new_infer_height(columns).map_err(|e| {
            Error::IngestionError(format!("Failed to build empty Workday schema: {}", e))
        })?;

        let mut ctx = RosterContext::with_deps(empty_df.lazy(), context.deps.clone());
        for name in WORKDAY_COLUMNS {
            ctx.set_field_source(name.to_string(), "WORKDAY_HRIS".into());
        }
        Ok(ctx)
    }
}

impl OnboardingAction for WorkdayHrisConnector {
    fn id(&self) -> &str {
        "workday_hris_connector"
    }

    fn execute(&self, context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            tenant = %self.config.tenant_id,
            "WorkdayHrisConnector: executing ingestion"
        );

        // 1. Fetch data → LazyFrame
        let mut lf = self.fetch_data()?;

        // 2. Discover column names for field-ownership metadata
        let schema = lf.collect_schema().map_err(|e| {
            Error::IngestionError(format!("Failed to collect Workday schema: {}", e))
        })?;

        // 3. Build the RosterContext with WORKDAY_HRIS provenance
        let mut ctx = RosterContext::with_deps(lf, context.deps.clone());
        for field_name in schema.iter_names() {
            ctx.set_field_source(field_name.to_string(), "WORKDAY_HRIS".into());
        }

        tracing::info!(
            fields = schema.len(),
            "WorkdayHrisConnector: ingested {} fields from Workday",
            schema.len()
        );

        Ok(ctx)
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Unit tests
// ───────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use onboard_you_models::ETLDependancies;
    use onboard_you_models::WorkdayResponseGroup;

    use super::*;

    // Re-export the trait locally so mock impls work
    use crate::orchestration::clients::soap_client::SoapClient;

    // ── Mock SOAP Client ───────────────────────────────────────────────

    /// Mock client that returns a canned XML response
    struct MockSoapClient {
        response_xml: String,
    }

    impl MockSoapClient {
        fn new(xml: &str) -> Self {
            Self {
                response_xml: xml.to_string(),
            }
        }
    }

    impl SoapClient for MockSoapClient {
        fn post_soap(&self, _endpoint: &str, _envelope: &str) -> Result<String> {
            Ok(self.response_xml.clone())
        }
    }

    /// Mock client that simulates an HTTP error
    struct ErrorSoapClient;

    impl SoapClient for ErrorSoapClient {
        fn post_soap(&self, _endpoint: &str, _envelope: &str) -> Result<String> {
            Err(Error::IngestionError("Connection refused".to_string()))
        }
    }

    // ── Test Data ──────────────────────────────────────────────────────

    fn sample_config() -> WorkdayConfig {
        WorkdayConfig {
            tenant_url: "https://wd3-impl-services1.workday.com".into(),
            tenant_id: "acme_test".into(),
            username: "ISU_Test".into(),
            password: "test_password_123".into(),
            worker_count_limit: 100,
            response_group: WorkdayResponseGroup::default(),
        }
    }

    const SAMPLE_WORKDAY_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<env:Envelope xmlns:env="http://schemas.xmlsoap.org/soap/envelope/">
  <env:Body>
    <wd:Get_Workers_Response xmlns:wd="urn:com.workday/bsvc">
      <wd:Response_Results>
        <wd:Total_Results>3</wd:Total_Results>
        <wd:Total_Pages>1</wd:Total_Pages>
        <wd:Page_Results>3</wd:Page_Results>
        <wd:Page>1</wd:Page>
      </wd:Response_Results>
      <wd:Response_Data>
        <wd:Worker>
          <wd:Worker_ID>WK-001</wd:Worker_ID>
          <wd:Employee_ID>EMP-001</wd:Employee_ID>
          <wd:First_Name>John</wd:First_Name>
          <wd:Last_Name>Doe</wd:Last_Name>
          <wd:Email_Address>john.doe@acme.com</wd:Email_Address>
          <wd:Phone_Number>+1-555-0101</wd:Phone_Number>
          <wd:Job_Title>Software Engineer</wd:Job_Title>
          <wd:Business_Title>Senior Developer</wd:Business_Title>
          <wd:Organization_Name>Engineering</wd:Organization_Name>
          <wd:Location_Name>San Francisco</wd:Location_Name>
          <wd:Hire_Date>2023-01-15</wd:Hire_Date>
          <wd:Worker_Type>Employee</wd:Worker_Type>
          <wd:Status>Active</wd:Status>
          <wd:Manager_ID>MGR-010</wd:Manager_ID>
          <wd:Manager_Name>Alice Manager</wd:Manager_Name>
          <wd:Position_ID>P-100</wd:Position_ID>
          <wd:Compensation_Grade>G7</wd:Compensation_Grade>
          <wd:Pay_Rate_Type>Salary</wd:Pay_Rate_Type>
        </wd:Worker>
        <wd:Worker>
          <wd:Worker_ID>WK-002</wd:Worker_ID>
          <wd:Employee_ID>EMP-002</wd:Employee_ID>
          <wd:First_Name>Jane</wd:First_Name>
          <wd:Last_Name>Smith</wd:Last_Name>
          <wd:Email_Address>jane.smith@acme.com</wd:Email_Address>
          <wd:Phone_Number>+1-555-0102</wd:Phone_Number>
          <wd:Job_Title>Product Manager</wd:Job_Title>
          <wd:Business_Title>Sr Product Manager</wd:Business_Title>
          <wd:Organization_Name>Product</wd:Organization_Name>
          <wd:Location_Name>New York</wd:Location_Name>
          <wd:Hire_Date>2022-06-01</wd:Hire_Date>
          <wd:Worker_Type>Employee</wd:Worker_Type>
          <wd:Status>Active</wd:Status>
          <wd:Manager_ID>MGR-020</wd:Manager_ID>
          <wd:Manager_Name>Bob Director</wd:Manager_Name>
          <wd:Position_ID>P-200</wd:Position_ID>
          <wd:Compensation_Grade>G8</wd:Compensation_Grade>
          <wd:Pay_Rate_Type>Salary</wd:Pay_Rate_Type>
        </wd:Worker>
        <wd:Worker>
          <wd:Worker_ID>WK-003</wd:Worker_ID>
          <wd:Employee_ID>CW-001</wd:Employee_ID>
          <wd:First_Name>Carlos</wd:First_Name>
          <wd:Last_Name>Contractor</wd:Last_Name>
          <wd:Email_Address>carlos.c@acme.com</wd:Email_Address>
          <wd:Phone_Number>+1-555-0103</wd:Phone_Number>
          <wd:Job_Title>UX Designer</wd:Job_Title>
          <wd:Business_Title>Contract UX Designer</wd:Business_Title>
          <wd:Organization_Name>Design</wd:Organization_Name>
          <wd:Location_Name>Austin</wd:Location_Name>
          <wd:Hire_Date>2024-03-01</wd:Hire_Date>
          <wd:Worker_Type>Contingent_Worker</wd:Worker_Type>
          <wd:Status>Active</wd:Status>
          <wd:Manager_ID>MGR-010</wd:Manager_ID>
          <wd:Manager_Name>Alice Manager</wd:Manager_Name>
          <wd:Position_ID>P-300</wd:Position_ID>
          <wd:Compensation_Grade>G5</wd:Compensation_Grade>
          <wd:Pay_Rate_Type>Hourly</wd:Pay_Rate_Type>
        </wd:Worker>
      </wd:Response_Data>
    </wd:Get_Workers_Response>
  </env:Body>
</env:Envelope>"#;

    // ── Config Tests ───────────────────────────────────────────────────

    #[test]
    fn test_config_from_json_valid() {
        let json = serde_json::json!({
            "tenant_url": "https://wd3.workday.com",
            "tenant_id": "test_tenant",
            "username": "ISU_Test",
            "password": "secret123",
            "worker_count_limit": 50
        });
        let config = WorkdayConfig::from_json(&json).unwrap();
        assert_eq!(config.tenant_id, "test_tenant");
        assert_eq!(config.worker_count_limit, 50);
        // defaults
        assert!(config.response_group.include_personal_information);
        assert!(config.response_group.include_employment_information);
        assert!(!config.response_group.include_compensation);
    }

    #[test]
    fn test_config_from_json_missing_required() {
        let json = serde_json::json!({ "tenant_url": "https://wd3.workday.com" });
        let result = WorkdayConfig::from_json(&json);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_default_worker_count() {
        let json = serde_json::json!({
            "tenant_url": "https://wd3.workday.com",
            "tenant_id": "t",
            "username": "u",
            "password": "p"
        });
        let config = WorkdayConfig::from_json(&json).unwrap();
        assert_eq!(config.worker_count_limit, 200);
    }

    #[test]
    fn test_soap_endpoint() {
        let config = sample_config();
        assert_eq!(
            config.soap_endpoint(),
            "https://wd3-impl-services1.workday.com/ccx/service/acme_test/Human_Resources/v45.2"
        );
    }

    #[test]
    fn test_soap_endpoint_trailing_slash() {
        let mut config = sample_config();
        config.tenant_url = "https://wd3.workday.com/".into();
        assert!(config.soap_endpoint().contains("/ccx/service/"));
        assert!(!config.soap_endpoint().contains("//ccx"));
    }

    #[test]
    fn test_password_env_resolution() {
        let mut config = sample_config();

        // Plain password
        assert_eq!(config.resolved_password().unwrap(), "test_password_123");

        // Env-based password
        std::env::set_var("TEST_WD_PWD", "env_secret");
        config.password = "env:TEST_WD_PWD".into();
        assert_eq!(config.resolved_password().unwrap(), "env_secret");
        std::env::remove_var("TEST_WD_PWD");

        // Missing env var
        config.password = "env:NONEXISTENT_VAR_ZZZZZ".into();
        assert!(config.resolved_password().is_err());
    }

    // ── SOAP Envelope Tests ────────────────────────────────────────────

    #[test]
    fn test_get_workers_envelope_structure() {
        let config = sample_config();
        let envelope = build_get_workers_envelope(&config, "secret", 1);

        assert!(envelope.contains("ISU_Test@acme_test"));
        assert!(envelope.contains("secret"));
        assert!(envelope.contains("Get_Workers_Request"));
        assert!(envelope.contains("<bsvc:Page>1</bsvc:Page>"));
        assert!(envelope.contains("<bsvc:Count>100</bsvc:Count>"));
        assert!(envelope.contains("Include_Personal_Information"));
        assert!(envelope.contains("UsernameToken"));
    }

    #[test]
    fn test_envelope_response_group_flags() {
        let mut config = sample_config();
        config.response_group.include_compensation = true;
        config.response_group.include_roles = true;
        let envelope = build_get_workers_envelope(&config, "pw", 1);

        assert!(envelope.contains("<bsvc:Include_Compensation>true</bsvc:Include_Compensation>"));
        assert!(envelope.contains("<bsvc:Include_Roles>true</bsvc:Include_Roles>"));
    }

    #[test]
    fn test_envelope_page_number() {
        let config = sample_config();
        let env_p3 = build_get_workers_envelope(&config, "pw", 3);
        assert!(env_p3.contains("<bsvc:Page>3</bsvc:Page>"));
    }

    // ── XML Parsing Tests ──────────────────────────────────────────────

    #[test]
    fn test_extract_tag_with_wd_prefix() {
        let xml = "<wd:First_Name>Alice</wd:First_Name>";
        assert_eq!(extract_tag(xml, "First_Name"), "Alice");
    }

    #[test]
    fn test_extract_tag_with_bsvc_prefix() {
        let xml = "<bsvc:Employee_ID>E123</bsvc:Employee_ID>";
        assert_eq!(extract_tag(xml, "Employee_ID"), "E123");
    }

    #[test]
    fn test_extract_tag_no_prefix() {
        let xml = "<Status>Active</Status>";
        assert_eq!(extract_tag(xml, "Status"), "Active");
    }

    #[test]
    fn test_extract_tag_missing() {
        let xml = "<wd:Something>Value</wd:Something>";
        assert_eq!(extract_tag(xml, "Nonexistent"), "");
    }

    #[test]
    fn test_parse_workers_response_count() {
        let records = parse_get_workers_response(SAMPLE_WORKDAY_XML).unwrap();
        assert_eq!(records.len(), 3, "should parse 3 worker records");
    }

    #[test]
    fn test_parse_workers_first_record() {
        let records = parse_get_workers_response(SAMPLE_WORKDAY_XML).unwrap();
        let r = &records[0];
        assert_eq!(r.worker_id, "WK-001");
        assert_eq!(r.employee_id, "EMP-001");
        assert_eq!(r.first_name, "John");
        assert_eq!(r.last_name, "Doe");
        assert_eq!(r.email, "john.doe@acme.com");
        assert_eq!(r.job_title, "Software Engineer");
        assert_eq!(r.department, "Engineering");
        assert_eq!(r.location, "San Francisco");
        assert_eq!(r.hire_date, "2023-01-15");
        assert_eq!(r.worker_type, "Employee");
        assert_eq!(r.worker_status, "Active");
    }

    #[test]
    fn test_parse_workers_contingent_worker() {
        let records = parse_get_workers_response(SAMPLE_WORKDAY_XML).unwrap();
        let r = &records[2];
        assert_eq!(r.worker_type, "Contingent_Worker");
        assert_eq!(r.first_name, "Carlos");
        assert_eq!(r.pay_rate_type, "Hourly");
    }

    #[test]
    fn test_parse_workers_empty_response() {
        let xml = r#"<env:Envelope><env:Body>
            <wd:Get_Workers_Response><wd:Response_Data></wd:Response_Data></wd:Get_Workers_Response>
        </env:Body></env:Envelope>"#;
        let records = parse_get_workers_response(xml).unwrap();
        assert!(records.is_empty());
    }

    // ── DataFrame Conversion Tests ─────────────────────────────────────

    #[test]
    fn test_workers_to_dataframe() {
        let records = parse_get_workers_response(SAMPLE_WORKDAY_XML).unwrap();
        let df = workers_to_dataframe(&records).unwrap();

        assert_eq!(df.height(), 3);
        assert_eq!(df.width(), 18, "should have 18 columns");

        // Verify column existence
        let expected_cols = [
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
        ];
        for col in &expected_cols {
            assert!(df.column(col).is_ok(), "missing column: {}", col);
        }

        // Verify data values
        let names = df.column("first_name").unwrap();
        let name0 = names.str().unwrap().get(0).unwrap();
        assert_eq!(name0, "John");
    }

    #[test]
    fn test_workers_to_dataframe_empty() {
        let df = workers_to_dataframe(&[]).unwrap();
        assert_eq!(df.height(), 0);
        assert_eq!(df.width(), 18);
    }

    // ── Connector / OnboardingAction Tests ─────────────────────────────

    #[test]
    fn test_connector_id() {
        let connector = WorkdayHrisConnector::new(sample_config());
        assert_eq!(connector.id(), "workday_hris_connector");
    }

    #[test]
    fn test_connector_execute_with_mock() {
        let config = sample_config();
        let client = Box::new(MockSoapClient::new(SAMPLE_WORKDAY_XML));
        let connector = WorkdayHrisConnector::with_client(config, client);

        let initial = RosterContext::with_deps(LazyFrame::default(), ETLDependancies::default());
        let ctx = connector.execute(initial).expect("execute should succeed");

        // Verify DataFrame
        let df = ctx.get_data().collect().expect("collect");
        assert_eq!(df.height(), 3);
        assert_eq!(df.width(), 18);

        // Verify field-ownership metadata
        assert_eq!(ctx.field_metadata().len(), 18);
        for meta in ctx.field_metadata().values() {
            assert_eq!(meta.source, "WORKDAY_HRIS");
            assert!(meta.modified_by.is_none());
        }
    }

    #[test]
    fn test_connector_execute_empty_response() {
        let xml = r#"<env:Envelope><env:Body>
            <wd:Get_Workers_Response><wd:Response_Data></wd:Response_Data></wd:Get_Workers_Response>
        </env:Body></env:Envelope>"#;

        let config = sample_config();
        let client = Box::new(MockSoapClient::new(xml));
        let connector = WorkdayHrisConnector::with_client(config, client);

        let initial = RosterContext::with_deps(LazyFrame::default(), ETLDependancies::default());
        let ctx = connector.execute(initial).expect("execute should succeed");
        let df = ctx.get_data().clone().collect().expect("collect");
        assert_eq!(df.height(), 0);
        assert_eq!(df.width(), 18);
    }

    #[test]
    fn test_connector_execute_soap_error() {
        let config = sample_config();
        let client = Box::new(ErrorSoapClient);
        let connector = WorkdayHrisConnector::with_client(config, client);

        let initial = RosterContext::with_deps(LazyFrame::default(), ETLDependancies::default());
        let result = connector.execute(initial);
        assert!(result.is_err());
    }

    #[test]
    fn test_connector_hris_trait() {
        let config = sample_config();
        let client = Box::new(MockSoapClient::new(SAMPLE_WORKDAY_XML));
        let connector = WorkdayHrisConnector::with_client(config, client);

        let lf = connector.fetch_data().expect("fetch_data");
        let df = lf.collect().expect("collect");
        assert_eq!(df.height(), 3);
    }

    #[test]
    fn test_from_action_config() {
        let json = serde_json::json!({
            "tenant_url": "https://wd3.workday.com",
            "tenant_id": "my_tenant",
            "username": "ISU_User",
            "password": "pass123"
        });
        let cfg: WorkdayConfig = serde_json::from_value(json.clone()).unwrap();
        let connector = WorkdayHrisConnector::from_action_config(&cfg).unwrap();
        assert_eq!(connector.id(), "workday_hris_connector");
        assert_eq!(connector.config.tenant_id, "my_tenant");
    }

    #[test]
    fn test_field_metadata_keys() {
        let config = sample_config();
        let client = Box::new(MockSoapClient::new(SAMPLE_WORKDAY_XML));
        let connector = WorkdayHrisConnector::with_client(config, client);

        let initial = RosterContext::with_deps(LazyFrame::default(), ETLDependancies::default());
        let ctx = connector.execute(initial).unwrap();

        let expected_fields = [
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
        ];
        for field in &expected_fields {
            assert!(
                ctx.field_metadata().contains_key(*field),
                "metadata missing field '{}'",
                field
            );
        }
    }

    // ── Response_Results / Pagination Tests ────────────────────────────

    #[test]
    fn test_parse_response_results_from_sample_xml() {
        let rr = parse_response_results(SAMPLE_WORKDAY_XML);
        assert_eq!(rr.total_results, 3);
        assert_eq!(rr.total_pages, 1);
        assert_eq!(rr.page_results, 3);
        assert_eq!(rr.page, 1);
    }

    #[test]
    fn test_parse_response_results_missing_tags() {
        let xml = "<env:Envelope><env:Body><empty/></env:Body></env:Envelope>";
        let rr = parse_response_results(xml);
        assert_eq!(rr.total_results, 0);
        assert_eq!(rr.total_pages, 1);
        assert_eq!(rr.page_results, 0);
        assert_eq!(rr.page, 1);
    }

    #[test]
    fn test_parse_response_results_multi_page() {
        let xml = r#"
        <wd:Response_Results>
          <wd:Total_Results>450</wd:Total_Results>
          <wd:Total_Pages>3</wd:Total_Pages>
          <wd:Page_Results>200</wd:Page_Results>
          <wd:Page>2</wd:Page>
        </wd:Response_Results>"#;
        let rr = parse_response_results(xml);
        assert_eq!(rr.total_results, 450);
        assert_eq!(rr.total_pages, 3);
        assert_eq!(rr.page_results, 200);
        assert_eq!(rr.page, 2);
    }

    /// Mock that returns different XML for each page, simulating multi-page
    /// Workday pagination.
    struct PaginatingMockClient;

    impl SoapClient for PaginatingMockClient {
        fn post_soap(&self, _endpoint: &str, envelope: &str) -> Result<String> {
            // Inspect the envelope to determine which page was requested
            let page = if envelope.contains("<bsvc:Page>1</bsvc:Page>") {
                1
            } else if envelope.contains("<bsvc:Page>2</bsvc:Page>") {
                2
            } else {
                panic!("unexpected page in envelope");
            };

            match page {
                1 => Ok(r#"<?xml version="1.0" encoding="UTF-8"?>
<env:Envelope xmlns:env="http://schemas.xmlsoap.org/soap/envelope/">
  <env:Body>
    <wd:Get_Workers_Response xmlns:wd="urn:com.workday/bsvc">
      <wd:Response_Results>
        <wd:Total_Results>3</wd:Total_Results>
        <wd:Total_Pages>2</wd:Total_Pages>
        <wd:Page_Results>2</wd:Page_Results>
        <wd:Page>1</wd:Page>
      </wd:Response_Results>
      <wd:Response_Data>
        <wd:Worker>
          <wd:Worker_ID>WK-P1A</wd:Worker_ID>
          <wd:Employee_ID>E-P1A</wd:Employee_ID>
          <wd:First_Name>PageOneA</wd:First_Name>
          <wd:Last_Name>Worker</wd:Last_Name>
          <wd:Email_Address>p1a@acme.com</wd:Email_Address>
          <wd:Job_Title>Engineer</wd:Job_Title>
          <wd:Status>Active</wd:Status>
        </wd:Worker>
        <wd:Worker>
          <wd:Worker_ID>WK-P1B</wd:Worker_ID>
          <wd:Employee_ID>E-P1B</wd:Employee_ID>
          <wd:First_Name>PageOneB</wd:First_Name>
          <wd:Last_Name>Worker</wd:Last_Name>
          <wd:Email_Address>p1b@acme.com</wd:Email_Address>
          <wd:Job_Title>Designer</wd:Job_Title>
          <wd:Status>Active</wd:Status>
        </wd:Worker>
      </wd:Response_Data>
    </wd:Get_Workers_Response>
  </env:Body>
</env:Envelope>"#
                    .to_string()),

                2 => Ok(r#"<?xml version="1.0" encoding="UTF-8"?>
<env:Envelope xmlns:env="http://schemas.xmlsoap.org/soap/envelope/">
  <env:Body>
    <wd:Get_Workers_Response xmlns:wd="urn:com.workday/bsvc">
      <wd:Response_Results>
        <wd:Total_Results>3</wd:Total_Results>
        <wd:Total_Pages>2</wd:Total_Pages>
        <wd:Page_Results>1</wd:Page_Results>
        <wd:Page>2</wd:Page>
      </wd:Response_Results>
      <wd:Response_Data>
        <wd:Worker>
          <wd:Worker_ID>WK-P2A</wd:Worker_ID>
          <wd:Employee_ID>E-P2A</wd:Employee_ID>
          <wd:First_Name>PageTwoA</wd:First_Name>
          <wd:Last_Name>Worker</wd:Last_Name>
          <wd:Email_Address>p2a@acme.com</wd:Email_Address>
          <wd:Job_Title>Manager</wd:Job_Title>
          <wd:Status>Active</wd:Status>
        </wd:Worker>
      </wd:Response_Data>
    </wd:Get_Workers_Response>
  </env:Body>
</env:Envelope>"#
                    .to_string()),

                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_pagination_fetches_all_pages() {
        let config = sample_config();
        let client = Box::new(PaginatingMockClient);
        let connector = WorkdayHrisConnector::with_client(config, client);

        let initial = RosterContext::with_deps(LazyFrame::default(), ETLDependancies::default());
        let ctx = connector.execute(initial).expect("paginated execute");
        let df = ctx.get_data().collect().expect("collect");

        // 2 workers from page 1 + 1 worker from page 2 = 3
        assert_eq!(df.height(), 3);

        let ids = df.column("worker_id").unwrap();
        let id_vec: Vec<Option<&str>> = ids.str().unwrap().into_iter().collect();
        assert_eq!(
            id_vec,
            vec![Some("WK-P1A"), Some("WK-P1B"), Some("WK-P2A"),]
        );
    }

    #[test]
    fn test_pagination_single_page_no_extra_calls() {
        // When Total_Pages == 1 there should be exactly one SOAP call
        let config = sample_config();
        let client = Box::new(MockSoapClient::new(SAMPLE_WORKDAY_XML));
        let connector = WorkdayHrisConnector::with_client(config, client);

        let lf = connector.fetch_data().expect("fetch_data");
        let df = lf.collect().expect("collect");
        // The sample XML has Total_Pages=1, so all 3 records are from page 1
        assert_eq!(df.height(), 3);
    }
}
