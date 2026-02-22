/// Query parameters for CSV upload / column discovery.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct CsvFileQuery {
    /// The CSV filename (e.g. `"employees.csv"`).
    pub filename: String,
}


/// Response payload after reading the uploaded CSV headers.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct CsvColumnsResponse {
    /// The filename of the CSV.
    pub filename: String,

    /// Column names parsed from the CSV header row.
    pub columns: Vec<String>,
}


/// Response payload for the presigned upload URL request.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PresignedUploadResponse {
    /// Presigned PUT URL — the frontend uses this to upload the CSV directly.
    pub upload_url: String,

    /// The S3 object key (for reference — not needed by the frontend).
    pub key: String,

    /// The filename that was requested.
    pub filename: String,
}