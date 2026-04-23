/// Request body for `POST /config/{id}/start-conversion`.
///
/// Instructs the backend to convert the already-uploaded file to CSV.
/// CSV files are short-circuited — no conversion is needed and column names
/// are returned immediately.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct StartConversionRequest {
    /// The filename of the already-uploaded file (e.g. `"employees.pdf"`).
    pub filename: String,

    /// Zero-based index of the Textract table / Excel sheet / JSON array to
    /// extract from multi-table documents.  Defaults to `0` when absent.
    #[serde(default)]
    pub table_index: Option<usize>,
}

/// Response from `POST /config/{id}/start-conversion`.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct StartConversionResponse {
    /// `"not_needed"` — file was already a CSV; columns returned inline.
    /// `"converted"`  — file was converted to CSV synchronously; columns returned.
    pub status: String,

    /// Column names of the CSV (always present on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<String>>,
}

/// Query parameters for CSV upload / column discovery.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct CsvFileQuery {
    /// The CSV filename (e.g. `"employees.csv"`).
    pub filename: String,
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
