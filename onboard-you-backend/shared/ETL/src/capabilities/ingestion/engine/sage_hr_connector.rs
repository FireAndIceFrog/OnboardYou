//! SageHrConnector: Sage HR REST API HRIS data ingestion
//!
//! Connects to the Sage HR REST API to fetch employee data via the
//! `/api/employees` endpoint. Returns a populated `RosterContext`
//! with every column tagged to `SAGE_HR` field-ownership metadata for
//! downstream provenance tracking.
//!
//! # Sage HR API Reference
//!
//! | Endpoint              | Method | Description                              |
//! |-----------------------|--------|------------------------------------------|
//! | `/api/employees`      | GET    | Returns paginated employee records       |
//!
//! Authentication is via the `X-Auth-Token` header containing an API token.

use crate::capabilities::ingestion::traits::HrisConnector;
use crate::orchestration::clients::rest_client::{RestClient, ReqwestRestClient};
use onboard_you_models::ColumnCalculator;
use onboard_you_models::{Error, OnboardingAction, Result, RosterContext};
use onboard_you_models::{SageHrApiResponse, SageHrConfig, SageHrRecord};
use polars::prelude::*;

/// Defines the column-name ↔ field-accessor mapping in one place.
///
/// Generates:
/// - `SAGE_HR_COLUMNS` — the fixed column list for schema / provenance.
/// - `employees_to_dataframe` — columnar extraction that is guaranteed
///    to stay aligned because both the column name and the struct field
///    come from the same macro arm.
macro_rules! sage_hr_fields {
    ( $( ($col:expr, $field:ident) ),* $(,)? ) => {
        /// The fixed set of columns produced by the Sage HR connector.
        const SAGE_HR_COLUMNS: &[&str] = &[ $( $col ),* ];

        /// Convert a vector of flat employee records into a Polars DataFrame.
        pub fn employees_to_dataframe(records: &[SageHrRecord]) -> Result<DataFrame> {
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

sage_hr_fields!(
    ("id",                        id),
    ("email",                     email),
    ("first_name",                first_name),
    ("last_name",                 last_name),
    ("picture_url",               picture_url),
    ("employment_start_date",     employment_start_date),
    ("date_of_birth",             date_of_birth),
    ("team",                      team),
    ("team_id",                   team_id),
    ("position",                  position),
    ("position_id",               position_id),
    ("reports_to_employee_id",    reports_to_employee_id),
    ("work_phone",                work_phone),
    ("home_phone",                home_phone),
    ("mobile_phone",              mobile_phone),
    ("gender",                    gender),
    ("street_first",              street_first),
    ("street_second",             street_second),
    ("city",                      city),
    ("post_code",                 post_code),
    ("country",                   country),
    ("employee_number",           employee_number),
    ("employment_status",         employment_status),
);

// ───────────────────────────────────────────────────────────────────────────
// Connector
// ───────────────────────────────────────────────────────────────────────────

/// HRIS connector that ingests employee data from the Sage HR REST API.
///
/// Implements both [`HrisConnector`] and [`OnboardingAction`].
///
/// ## Pipeline manifest example
///
/// ```json
/// {
///   "id": "ingest_sage_hr",
///   "action_type": "sage_hr_connector",
///   "config": {
///     "subdomain": "acme",
///     "api_token": "env:SAGE_HR_API_TOKEN",
///     "include_team_history": false,
///     "include_employment_status_history": false,
///     "include_position_history": false
///   }
/// }
/// ```
pub struct SageHrConnector {
    config: SageHrConfig,
    client: Box<dyn RestClient>,
}

impl std::fmt::Debug for SageHrConnector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SageHrConnector")
            .field("config", &self.config)
            .finish()
    }
}

impl SageHrConnector {
    /// Create a new connector with the production HTTP client.
    pub fn new(config: SageHrConfig) -> Self {
        Self {
            config,
            client: Box::new(ReqwestRestClient),
        }
    }

    /// Create a connector with a custom [`RestClient`] (for testing).
    pub fn with_client(config: SageHrConfig, client: Box<dyn RestClient>) -> Self {
        Self { config, client }
    }

    /// Construct from a deserialised config.
    pub fn from_action_config(config: &SageHrConfig) -> Result<Self> {
        Ok(Self::new(config.clone()))
    }

    /// Fetch all employees from Sage HR, paging through all results.
    ///
    /// The API uses `meta.next_page` to indicate whether more pages exist.
    /// Runs asynchronously via the async [`RestClient`] trait.
    async fn fetch_all_employees(&self) -> Result<Vec<SageHrRecord>> {
        let api_token = self.config.api_token();
        let endpoint = self.config.employees_endpoint();

        let mut all_records: Vec<SageHrRecord> = Vec::new();
        let mut current_page: u32 = 1;
        let mut total_pages: u32 = 1;

        loop {
            let query_params = self.config.query_params(current_page);

            tracing::info!(
                endpoint = %endpoint,
                subdomain = %self.config.subdomain,
                page = current_page,
                total_pages = total_pages,
                "SageHrConnector: fetching employees page {}/{}",
                current_page,
                total_pages,
            );

            let response_body = self.client.get(
                &endpoint,
                &[("X-Auth-Token", api_token)],
                &query_params,
            ).await?;

            let response: SageHrApiResponse = serde_json::from_str(&response_body)
                .map_err(|e| Error::IngestionError(format!("Failed to parse Sage HR response: {}", e)))?;

            total_pages = response.meta.total_pages;

            tracing::info!(
                total_pages = response.meta.total_pages,
                current_page = response.meta.current_page,
                employees = response.data.len(),
                "SageHrConnector: parsed {} employees from page {}",
                response.data.len(),
                current_page,
            );

            let page_records: Vec<SageHrRecord> = response.data.into_iter().map(Into::into).collect();
            all_records.extend(page_records);

            match response.meta.next_page {
                Some(next) if next > current_page => {
                    current_page = next;
                }
                _ => break,
            }
        }

        tracing::info!(
            total_employees = all_records.len(),
            total_pages = total_pages,
            "SageHrConnector: finished pagination — {} employees across {} page(s)",
            all_records.len(),
            total_pages,
        );

        Ok(all_records)
    }

    /// Bridge from sync to async: runs the async pagination loop on the
    /// current tokio runtime without blocking the executor thread.
    fn fetch_all_employees_sync(&self) -> Result<Vec<SageHrRecord>> {
        let handle = tokio::runtime::Handle::current();
        tokio::task::block_in_place(|| handle.block_on(self.fetch_all_employees()))
    }
}


impl HrisConnector for SageHrConnector {
    fn fetch_data(&self) -> Result<LazyFrame> {
        let records = self.fetch_all_employees_sync()?;
        let df = employees_to_dataframe(&records)?;
        Ok(df.lazy())
    }
}

impl ColumnCalculator for SageHrConnector {
    fn calculate_columns(&self, _context: RosterContext) -> Result<RosterContext> {
        let columns: Vec<Column> = SAGE_HR_COLUMNS
            .iter()
            .map(|name| Column::new((*name).into(), Vec::<&str>::new()))
            .collect();
        let empty_df = DataFrame::new_infer_height(columns).map_err(|e| {
            Error::IngestionError(format!("Failed to build empty Sage HR schema: {}", e))
        })?;

        let mut ctx = RosterContext::new(empty_df.lazy());
        for name in SAGE_HR_COLUMNS {
            ctx.set_field_source(name.to_string(), "SAGE_HR".into());
        }
        Ok(ctx)
    }
}

impl OnboardingAction for SageHrConnector {
    fn id(&self) -> &str {
        "sage_hr_connector"
    }

    fn execute(&self, _context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            subdomain = %self.config.subdomain,
            "SageHrConnector: executing ingestion"
        );

        let mut lf = self.fetch_data()?;

        let schema = lf.collect_schema().map_err(|e| {
            Error::IngestionError(format!("Failed to collect Sage HR schema: {}", e))
        })?;

        let mut ctx = RosterContext::new(lf);
        for field_name in schema.iter_names() {
            ctx.set_field_source(field_name.to_string(), "SAGE_HR".into());
        }

        tracing::info!(
            fields = schema.len(),
            "SageHrConnector: ingested {} fields from Sage HR",
            schema.len()
        );

        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::clients::rest_client::RestClient;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    // ── Mock REST Client ────────────────────────────────────────────────

    struct MockRestClient {
        responses: Vec<String>,
        call_count: AtomicUsize,
    }

    impl MockRestClient {
        fn single(body: &str) -> Self {
            Self {
                responses: vec![body.to_string()],
                call_count: AtomicUsize::new(0),
            }
        }

        fn paginated(pages: Vec<&str>) -> Self {
            Self {
                responses: pages.into_iter().map(String::from).collect(),
                call_count: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait::async_trait]
    impl RestClient for MockRestClient {
        async fn get(
            &self,
            _url: &str,
            _headers: &[(&str, &str)],
            _query_params: &[(&str, String)],
        ) -> Result<String> {
            let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(self.responses[idx.min(self.responses.len() - 1)].clone())
        }
    }

    // ── Declarative Case Struct ─────────────────────────────────────────

    struct Case {
        name: &'static str,
        config: SageHrConfig,
        responses: Vec<&'static str>,
    }

    fn default_config() -> SageHrConfig {
        SageHrConfig {
            subdomain: "acme".into(),
            api_token: "test_token_123".into(),
            include_team_history: false,
            include_employment_status_history: false,
            include_position_history: false,
        }
    }

    // ── Test Data ───────────────────────────────────────────────────────

    const SINGLE_PAGE_RESPONSE: &str = r#"{
        "data": [
            {
                "id": 19,
                "email": "john@example.com",
                "first_name": "John",
                "last_name": "Doe",
                "picture_url": "https://example.com/john.png",
                "employment_start_date": "2014-08-25",
                "date_of_birth": "1991-02-13",
                "team": "Engineering",
                "team_id": 1,
                "position": "Developer",
                "position_id": 123,
                "reports_to_employee_id": 5,
                "work_phone": "555-0505",
                "home_phone": "555-0506",
                "mobile_phone": "555-0507",
                "gender": "Male",
                "street_first": "84 Glenwood Street",
                "street_second": "Peoria",
                "city": "London",
                "post_code": 99999,
                "country": "GB",
                "employee_number": "A01",
                "employment_status": "Full-time"
            },
            {
                "id": 20,
                "email": "jane@example.com",
                "first_name": "Jane",
                "last_name": "Smith",
                "employment_start_date": "2020-03-01",
                "team": "Marketing",
                "team_id": 2,
                "position": "Manager",
                "position_id": 456,
                "gender": "Female",
                "city": "New York",
                "country": "US",
                "employee_number": "A02",
                "employment_status": "Part-time"
            }
        ],
        "meta": {
            "current_page": 1,
            "next_page": null,
            "previous_page": null,
            "total_pages": 1,
            "per_page": 50,
            "total_entries": 2
        }
    }"#;

    const PAGINATED_PAGE_1: &str = r#"{
        "data": [
            { "id": 1, "email": "a@example.com", "first_name": "Alice", "last_name": "A" }
        ],
        "meta": {
            "current_page": 1,
            "next_page": 2,
            "previous_page": null,
            "total_pages": 2,
            "per_page": 1,
            "total_entries": 2
        }
    }"#;

    const PAGINATED_PAGE_2: &str = r#"{
        "data": [
            { "id": 2, "email": "b@example.com", "first_name": "Bob", "last_name": "B" }
        ],
        "meta": {
            "current_page": 2,
            "next_page": null,
            "previous_page": 1,
            "total_pages": 2,
            "per_page": 1,
            "total_entries": 2
        }
    }"#;

    const EMPTY_RESPONSE: &str = r#"{
        "data": [],
        "meta": {
            "current_page": 1,
            "next_page": null,
            "previous_page": null,
            "total_pages": 1,
            "per_page": 50,
            "total_entries": 0
        }
    }"#;

    // ── Helpers ──────────────────────────────────────────────────────────

    fn format_records(records: &[SageHrRecord]) -> String {
        let rows: Vec<String> = records
            .iter()
            .map(|r| {
                format!(
                    "id={} email={} name={} {} team={} position={} status={}",
                    r.id, r.email, r.first_name, r.last_name, r.team, r.position, r.employment_status,
                )
            })
            .collect();
        rows.join("\n")
    }

    fn format_dataframe_shape(df: &DataFrame) -> String {
        let columns: Vec<String> = df
            .get_column_names()
            .into_iter()
            .map(|n| n.to_string())
            .collect();
        format!(
            "rows={} cols={}\ncolumns=[{}]",
            df.height(),
            df.width(),
            columns.join(", ")
        )
    }

    // ── Connector Tests (insta snapshots) ───────────────────────────────

    fn all_connector_cases() -> Vec<Case> {
        vec![
            Case {
                name: "single_page_two_employees",
                config: default_config(),
                responses: vec![SINGLE_PAGE_RESPONSE],
            },
            Case {
                name: "paginated_two_pages",
                config: default_config(),
                responses: vec![PAGINATED_PAGE_1, PAGINATED_PAGE_2],
            },
            Case {
                name: "empty_response",
                config: default_config(),
                responses: vec![EMPTY_RESPONSE],
            },
            Case {
                name: "with_team_history_flag",
                config: SageHrConfig {
                    include_team_history: true,
                    ..default_config()
                },
                responses: vec![SINGLE_PAGE_RESPONSE],
            },
            Case {
                name: "with_all_history_flags",
                config: SageHrConfig {
                    include_team_history: true,
                    include_employment_status_history: true,
                    include_position_history: true,
                    ..default_config()
                },
                responses: vec![SINGLE_PAGE_RESPONSE],
            },
        ]
    }

    #[tokio::test]
    async fn sage_hr_connector_cases() {
        for case in all_connector_cases() {
            let client = MockRestClient::paginated(case.responses);
            let connector = SageHrConnector::with_client(case.config, Box::new(client));

            let records = connector
                .fetch_all_employees()
                .await
                .expect("should fetch employees");
            let output = format_records(&records);
            insta::assert_snapshot!(case.name, output);
        }
    }

    // ── DataFrame shape tests ───────────────────────────────────────────

    #[tokio::test]
    async fn sage_hr_dataframe_shape() {
        let client = MockRestClient::single(SINGLE_PAGE_RESPONSE);
        let connector = SageHrConnector::with_client(default_config(), Box::new(client));

        let records = connector.fetch_all_employees().await.unwrap();
        let df = employees_to_dataframe(&records).unwrap();

        insta::assert_snapshot!("dataframe_shape", format_dataframe_shape(&df));
    }

    // ── Row-alignment integrity ─────────────────────────────────────────
    //
    // Proves that every column in the DataFrame is aligned with the source
    // record at the same row index.  If the macro-generated extraction
    // ever re-ordered a column relative to its data, this snapshot would
    // change.

    fn format_row(df: &DataFrame, row: usize) -> String {
        SAGE_HR_COLUMNS
            .iter()
            .map(|col| {
                let val = df
                    .column(*col)
                    .unwrap()
                    .str()
                    .unwrap()
                    .get(row)
                    .unwrap_or("");
                format!("{}={}", col, val)
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }

    #[tokio::test]
    async fn sage_hr_row_alignment() {
        let client = MockRestClient::single(SINGLE_PAGE_RESPONSE);
        let connector = SageHrConnector::with_client(default_config(), Box::new(client));

        let records = connector.fetch_all_employees().await.unwrap();
        let df = employees_to_dataframe(&records).unwrap();

        let rows: Vec<String> = (0..df.height())
            .map(|i| format!("row {}: {}", i, format_row(&df, i)))
            .collect();

        insta::assert_snapshot!("row_alignment", rows.join("\n"));
    }

    // ── Column calculation tests ────────────────────────────────────────

    #[test]
    fn sage_hr_column_calculation() {
        let connector = SageHrConnector::new(default_config());
        let ctx = RosterContext::new(LazyFrame::default());

        let mut result = connector.calculate_columns(ctx).expect("should calculate columns");
        let schema = result.data.collect_schema().expect("should collect schema");

        let columns: Vec<String> = schema.iter_names().map(|n| n.to_string()).collect();
        insta::assert_snapshot!("column_calculation", columns.join(", "));
    }

    // ── Config tests ────────────────────────────────────────────────────

    struct ConfigCase {
        name: &'static str,
        config: SageHrConfig,
        expected_endpoint: &'static str,
        expected_query_page: u32,
    }

    fn all_config_cases() -> Vec<ConfigCase> {
        vec![
            ConfigCase {
                name: "basic_endpoint",
                config: default_config(),
                expected_endpoint: "https://acme.sage.hr/api/employees",
                expected_query_page: 1,
            },
            ConfigCase {
                name: "subdomain_with_spaces",
                config: SageHrConfig {
                    subdomain: "  myco  ".into(),
                    ..default_config()
                },
                expected_endpoint: "https://myco.sage.hr/api/employees",
                expected_query_page: 1,
            },
            ConfigCase {
                name: "query_params_page_3_with_history",
                config: SageHrConfig {
                    include_team_history: true,
                    include_position_history: true,
                    ..default_config()
                },
                expected_endpoint: "https://acme.sage.hr/api/employees",
                expected_query_page: 3,
            },
        ]
    }

    #[test]
    fn sage_hr_config_cases() {
        for case in all_config_cases() {
            assert_eq!(
                case.config.employees_endpoint(),
                case.expected_endpoint,
                "endpoint mismatch for '{}'",
                case.name
            );

            let params = case.config.query_params(case.expected_query_page);
            let params_str: Vec<String> = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            insta::assert_snapshot!(case.name, params_str.join("&"));
        }
    }

    // ── Identity tests ──────────────────────────────────────────────────

    #[test]
    fn sage_hr_action_id() {
        let connector = SageHrConnector::new(default_config());
        assert_eq!(connector.id(), "sage_hr_connector");
    }

    #[test]
    fn sage_hr_token_passthrough() {
        let config = SageHrConfig {
            api_token: "my_secret_token".into(),
            ..default_config()
        };
        assert_eq!(config.api_token(), "my_secret_token");
    }

    // ── Efficiency: parsing throughput ───────────────────────────────────

    #[test]
    fn sage_hr_parsing_efficiency() {
        // Generate a large response with 500 employees
        let employees: Vec<String> = (1..=500)
            .map(|i| {
                format!(
                    r#"{{ "id": {i}, "email": "emp{i}@example.com", "first_name": "First{i}", "last_name": "Last{i}", "team": "Team{t}", "team_id": {t}, "position": "Pos{i}", "position_id": {i}, "employee_number": "E{i:04}", "employment_status": "Full-time" }}"#,
                    i = i,
                    t = (i % 10) + 1,
                )
            })
            .collect();
        let json = format!(
            r#"{{ "data": [{}], "meta": {{ "current_page": 1, "next_page": null, "previous_page": null, "total_pages": 1, "per_page": 500, "total_entries": 500 }} }}"#,
            employees.join(",\n")
        );

        let start = Instant::now();
        let response: SageHrApiResponse = serde_json::from_str(&json).unwrap();
        let records: Vec<SageHrRecord> = response.data.into_iter().map(Into::into).collect();
        let df = employees_to_dataframe(&records).unwrap();
        let elapsed = start.elapsed();

        assert_eq!(df.height(), 500);
        assert_eq!(df.width(), SAGE_HR_COLUMNS.len());

        // Parsing + DataFrame construction for 500 records should complete
        // in well under 100ms on any reasonable hardware
        assert!(
            elapsed.as_millis() < 100,
            "Parsing 500 employees took {}ms (expected <100ms)",
            elapsed.as_millis()
        );

        insta::assert_snapshot!(
            "parsing_efficiency_500",
            format!(
                "rows={} cols={} elapsed_under_100ms=true",
                df.height(),
                df.width()
            )
        );
    }
}
