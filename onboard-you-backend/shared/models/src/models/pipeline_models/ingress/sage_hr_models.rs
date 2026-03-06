//! Sage HR API response models
//!
//! These models represent the JSON structures returned by the Sage HR
//! REST API. They live in the shared models crate so both the ETL connector
//! and the API can reference (and document) them.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ───────────────────────────────────────────────────────────────────────────
// API Response Envelope
// ───────────────────────────────────────────────────────────────────────────

/// Top-level response from the Sage HR `/api/employees` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SageHrApiResponse {
    pub data: Vec<SageHrEmployee>,
    pub meta: SageHrMeta,
}

/// A single employee record as returned by the Sage HR API.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SageHrEmployee {
    pub id: u64,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub picture_url: Option<String>,
    #[serde(default)]
    pub employment_start_date: Option<String>,
    #[serde(default)]
    pub date_of_birth: Option<String>,
    #[serde(default)]
    pub team: Option<String>,
    #[serde(default)]
    pub team_id: Option<u64>,
    #[serde(default)]
    pub position: Option<String>,
    #[serde(default)]
    pub position_id: Option<u64>,
    #[serde(default)]
    pub reports_to_employee_id: Option<u64>,
    #[serde(default)]
    pub work_phone: Option<String>,
    #[serde(default)]
    pub home_phone: Option<String>,
    #[serde(default)]
    pub mobile_phone: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub street_first: Option<String>,
    #[serde(default)]
    pub street_second: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub post_code: Option<serde_json::Value>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub employee_number: Option<String>,
    #[serde(default)]
    pub employment_status: Option<String>,
    #[serde(default)]
    pub team_history: Option<Vec<SageHrTeamHistory>>,
    #[serde(default)]
    pub employment_status_history: Option<Vec<SageHrEmploymentStatusHistory>>,
    #[serde(default)]
    pub position_history: Option<Vec<SageHrPositionHistory>>,
}

/// Pagination metadata from the Sage HR API.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SageHrMeta {
    pub current_page: u32,
    pub next_page: Option<u32>,
    pub previous_page: Option<u32>,
    pub total_pages: u32,
    pub per_page: u32,
    pub total_entries: u32,
}

/// A team assignment history entry.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SageHrTeamHistory {
    pub team_id: u64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub team_name: Option<String>,
}

/// An employment status history entry.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SageHrEmploymentStatusHistory {
    pub employment_status_id: u64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(alias = "employment_statu_name")]
    pub employment_statu_name: Option<String>,
}

/// A position history entry.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SageHrPositionHistory {
    pub position_id: u64,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub position_name: Option<String>,
    pub position_code: Option<String>,
}

// ───────────────────────────────────────────────────────────────────────────
// Flat record for DataFrame conversion
// ───────────────────────────────────────────────────────────────────────────

/// Flat record used to convert Sage HR employee data into a Polars DataFrame.
#[derive(Debug, Clone, Default)]
pub struct SageHrRecord {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub picture_url: String,
    pub employment_start_date: String,
    pub date_of_birth: String,
    pub team: String,
    pub team_id: String,
    pub position: String,
    pub position_id: String,
    pub reports_to_employee_id: String,
    pub work_phone: String,
    pub home_phone: String,
    pub mobile_phone: String,
    pub gender: String,
    pub street_first: String,
    pub street_second: String,
    pub city: String,
    pub post_code: String,
    pub country: String,
    pub employee_number: String,
    pub employment_status: String,
}

impl From<SageHrEmployee> for SageHrRecord {
    fn from(e: SageHrEmployee) -> Self {
        Self {
            id: e.id.to_string(),
            email: e.email.unwrap_or_default(),
            first_name: e.first_name.unwrap_or_default(),
            last_name: e.last_name.unwrap_or_default(),
            picture_url: e.picture_url.unwrap_or_default(),
            employment_start_date: e.employment_start_date.unwrap_or_default(),
            date_of_birth: e.date_of_birth.unwrap_or_default(),
            team: e.team.unwrap_or_default(),
            team_id: e.team_id.map(|v| v.to_string()).unwrap_or_default(),
            position: e.position.unwrap_or_default(),
            position_id: e.position_id.map(|v| v.to_string()).unwrap_or_default(),
            reports_to_employee_id: e
                .reports_to_employee_id
                .map(|v| v.to_string())
                .unwrap_or_default(),
            work_phone: e.work_phone.unwrap_or_default(),
            home_phone: e.home_phone.unwrap_or_default(),
            mobile_phone: e.mobile_phone.unwrap_or_default(),
            gender: e.gender.unwrap_or_default(),
            street_first: e.street_first.unwrap_or_default(),
            street_second: e.street_second.unwrap_or_default(),
            city: e.city.unwrap_or_default(),
            post_code: e
                .post_code
                .map(|v| match v {
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => s,
                    _ => String::new(),
                })
                .unwrap_or_default(),
            country: e.country.unwrap_or_default(),
            employee_number: e.employee_number.unwrap_or_default(),
            employment_status: e.employment_status.unwrap_or_default(),
        }
    }
}
