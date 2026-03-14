use onboard_you_models::PipelineWarning;
use serde::Serialize;

/// Response payload returned by the ETL trigger Lambda.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineResult {
    pub run_id: String,
    pub organization_id: String,
    pub customer_company_id: String,
    pub status: String,
    pub rows_processed: Option<usize>,
    pub error: Option<String>,
    pub warnings: Vec<PipelineWarning>,
}

impl PipelineResult {
    pub fn success(
        run_id: &str,
        organization_id: &str,
        customer_company_id: &str,
        rows: Option<usize>,
        warnings: Vec<PipelineWarning>,
    ) -> Self {
        Self {
            run_id: run_id.to_string(),
            organization_id: organization_id.to_string(),
            customer_company_id: customer_company_id.to_string(),
            status: "success".to_string(),
            rows_processed: rows,
            error: None,
            warnings,
        }
    }

    pub fn failure(
        run_id: &str,
        organization_id: &str,
        customer_company_id: &str,
        error: impl std::fmt::Display,
        warnings: Vec<PipelineWarning>,
    ) -> Self {
        Self {
            run_id: run_id.to_string(),
            organization_id: organization_id.to_string(),
            customer_company_id: customer_company_id.to_string(),
            status: "error".to_string(),
            rows_processed: None,
            error: Some(error.to_string()),
            warnings,
        }
    }
}
