use serde::Serialize;

/// Response payload returned by the ETL trigger Lambda.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineResult {
    pub organization_id: String,
    pub customer_company_id: String,
    pub status: String,
    pub rows_processed: Option<usize>,
    pub error: Option<String>,
}

impl PipelineResult {
    pub fn success(organization_id: &str, customer_company_id: &str, rows: Option<usize>) -> Self {
        Self {
            organization_id: organization_id.to_string(),
            customer_company_id: customer_company_id.to_string(),
            status: "success".to_string(),
            rows_processed: rows,
            error: None,
        }
    }

    pub fn failure(organization_id: &str, customer_company_id: &str, error: impl std::fmt::Display) -> Self {
        Self {
            organization_id: organization_id.to_string(),
            customer_company_id: customer_company_id.to_string(),
            status: "error".to_string(),
            rows_processed: None,
            error: Some(error.to_string()),
        }
    }
}
