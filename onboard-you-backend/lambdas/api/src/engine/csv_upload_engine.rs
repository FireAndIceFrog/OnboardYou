//! CSV upload engine — coordinates presigned URL generation and column discovery.

use crate::dependancies::Dependancies;
use crate::engine::file_converter_engine;
use crate::models::{ApiError, PresignedUploadResponse};
use chrono::Utc;

/// Build the S3 object key from runtime context.
fn s3_key(organization_id: &str, customer_company_id: &str, filename: &str) -> String {
    format!("{organization_id}/{customer_company_id}/{filename}")
}

/// Inject a UTC timestamp into a filename stem so every upload is unique.
///
/// `"employees.pdf"` → `"employees_20260425T143000Z.pdf"`
/// `"roster.csv"`    → `"roster_20260425T143000Z.csv"`
/// `"data"`          → `"data_20260425T143000Z"`
pub fn timestamped_filename(filename: &str) -> String {
    let ts = Utc::now().format("%Y%m%dT%H%M%SZ");
    match filename.rfind('.') {
        Some(pos) if pos > 0 => {
            let stem = &filename[..pos];
            let ext = &filename[pos..]; // includes the dot
            format!("{stem}_{ts}{ext}")
        }
        _ => format!("{filename}_{ts}"),
    }
}

/// Generate a presigned PUT URL for a CSV upload.
///
/// The returned `filename` in the response is a **timestamped** variant of
/// the original (e.g. `"employees_20260425T143000Z.pdf"`).  The caller must
/// use this server-assigned name — not the local `File.name` — for any
/// subsequent `start-conversion` call and for storing in the manifest config.
pub async fn presigned_upload(
    deps: &Dependancies,
    organization_id: &str,
    customer_company_id: &str,
    filename: &str,
) -> Result<PresignedUploadResponse, ApiError> {
    if filename.is_empty() {
        return Err(ApiError::Validation("filename must not be empty".into()));
    }

    // Sanitise: only allow simple filenames (no path traversal)
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(ApiError::Validation(
            "filename must not contain path separators or '..'".into(),
        ));
    }

    let stamped = timestamped_filename(filename);
    let key = s3_key(organization_id, customer_company_id, &stamped);

    let upload_url = deps.s3_repo.presigned_put_url(&key).await?;

    Ok(PresignedUploadResponse {
        upload_url,
        key,
        filename: stamped,
    })
}

/// Determine whether a filename refers to a CSV (case-insensitive extension check).
fn is_csv(filename: &str) -> bool {
    filename.to_lowercase().ends_with(".csv")
}

/// Handle a `start-conversion` request.
///
/// **CSV fast-path**: if the uploaded file is already a CSV the columns are
/// read directly from S3 and returned inline — no Textract call is made.
///
/// **Non-CSV**: validates the filename and returns `{status: "queued"}`.  The
/// actual Textract job is submitted by the `file-converter` Lambda (invoked
/// separately). The frontend should show a "converting" state and re-check
/// the pipeline run status before allowing the ETL to trigger.
pub async fn start_conversion(
    deps: &Dependancies,
    organization_id: &str,
    customer_company_id: &str,
    filename: &str,
    table_index: usize,
) -> Result<crate::models::StartConversionResponse, ApiError> {
    if filename.is_empty() {
        return Err(ApiError::Validation("filename must not be empty".into()));
    }
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(ApiError::Validation(
            "filename must not contain path separators or '..'".into(),
        ));
    }

    // Build the S3 key for the original upload.
    let original_key = s3_key(organization_id, customer_company_id, filename);

    if is_csv(filename) {
        // Fast-path: read column headers directly from the uploaded CSV.
        let columns = deps.s3_repo.read_csv_headers(&original_key).await?;
        return Ok(crate::models::StartConversionResponse {
            status: "not_needed".into(),
            columns: Some(columns),
        });
    }

    // Non-CSV: convert synchronously and store the resulting CSV in S3.
    // The CSV key reuses the same path but with a .csv extension.
    let stem = filename
        .rfind('.')
        .map(|i| &filename[..i])
        .unwrap_or(filename);
    let csv_key = s3_key(organization_id, customer_company_id, &format!("{stem}.csv"));

    // table_index defaults to 0 for now; a future API version can expose it.
    let columns = file_converter_engine::convert_to_csv(
        deps.s3_repo.as_ref(),
        deps.textract_repo.as_ref(),
        &original_key,
        &csv_key,
        table_index,
    )
    .await?;

    Ok(crate::models::StartConversionResponse {
        status: "converted".into(),
        columns: Some(columns),
    })
}

#[cfg(test)]
mod tests {
    use crate::dependancies::{Dependancies, Env};
    use crate::models::ApiError;
    use crate::repositories::s3_repository::S3Repo;
    use crate::repositories::textract_repository::TextractRepo;
    use async_trait::async_trait;
    use onboard_you::capabilities::conversion::textract_parser::TextractBlock;
    use std::sync::Arc;

    struct FakeS3 {
        presign: Option<String>,
        headers: Option<Vec<String>>,
        err: Option<String>,
    }

    #[async_trait]
    impl S3Repo for FakeS3 {
        async fn presigned_put_url(&self, _key: &str) -> Result<String, ApiError> {
            if let Some(e) = &self.err {
                return Err(ApiError::Repository(e.clone()));
            }
            Ok(self
                .presign
                .clone()
                .unwrap_or_else(|| "https://example.com/upload".into()))
        }

        async fn read_csv_headers(&self, _key: &str) -> Result<Vec<String>, ApiError> {
            if let Some(e) = &self.err {
                return Err(ApiError::Repository(e.clone()));
            }
            Ok(self.headers.clone().unwrap_or_default())
        }

        async fn get_object_bytes(&self, _key: &str) -> Result<Vec<u8>, ApiError> {
            if let Some(e) = &self.err {
                return Err(ApiError::Repository(e.clone()));
            }
            Ok(Vec::new())
        }

        async fn put_object_bytes(
            &self,
            _key: &str,
            _bytes: Vec<u8>,
            _content_type: &str,
        ) -> Result<(), ApiError> {
            if let Some(e) = &self.err {
                return Err(ApiError::Repository(e.clone()));
            }
            Ok(())
        }
    }

    struct FakeTextract;

    #[async_trait]
    impl TextractRepo for FakeTextract {
        async fn analyze_s3_document(&self, _key: &str) -> Result<Vec<TextractBlock>, ApiError> {
            Ok(Vec::new())
        }
    }

    async fn build_state(s3: FakeS3) -> Dependancies {
        let mut deps = Dependancies::new(Env::default()).await;
        deps.s3_repo = Arc::new(s3);
        deps.textract_repo = Arc::new(FakeTextract);
        deps
    }

    #[tokio::test]
    async fn presigned_upload_returns_timestamped_filename() {
        let state = build_state(FakeS3 {
            presign: Some("https://presigned.url".into()),
            headers: None,
            err: None,
        })
        .await;

        let out = super::presigned_upload(&state, "org", "comp", "file.csv")
            .await
            .unwrap();
        assert_eq!(out.upload_url, "https://presigned.url");
        // Filename must start with "file_" and end with ".csv" (timestamp in between)
        assert!(out.filename.starts_with("file_"), "filename: {}", out.filename);
        assert!(out.filename.ends_with(".csv"), "filename: {}", out.filename);
        // S3 key must use the timestamped name
        assert!(out.key.starts_with("org/comp/file_"), "key: {}", out.key);
        assert!(out.key.ends_with(".csv"), "key: {}", out.key);
    }

    #[tokio::test]
    async fn presigned_upload_validation() {
        let state = build_state(FakeS3 {
            presign: None,
            headers: None,
            err: None,
        })
        .await;
        let err = super::presigned_upload(&state, "org", "comp", "")
            .await
            .unwrap_err();
        assert!(matches!(err, ApiError::Validation(_)));

        let err2 = super::presigned_upload(&state, "org", "comp", "../evil")
            .await
            .unwrap_err();
        assert!(matches!(err2, ApiError::Validation(_)));
    }

    // -----------------------------------------------------------------------
    // timestamped_filename — table-driven
    // -----------------------------------------------------------------------

    #[test]
    fn timestamped_filename_preserves_extension() {
        struct Case { input: &'static str, prefix: &'static str, suffix: &'static str }
        let cases = vec![
            Case { input: "employees.pdf", prefix: "employees_", suffix: ".pdf" },
            Case { input: "roster.csv",    prefix: "roster_",    suffix: ".csv" },
            Case { input: "data.xml",      prefix: "data_",      suffix: ".xml" },
            Case { input: "report.xlsx",   prefix: "report_",    suffix: ".xlsx" },
            Case { input: "img.JPEG",      prefix: "img_",       suffix: ".JPEG" },
        ];
        for c in &cases {
            let out = super::timestamped_filename(c.input);
            assert!(out.starts_with(c.prefix), "input={}: got {}", c.input, out);
            assert!(out.ends_with(c.suffix), "input={}: got {}", c.input, out);
        }
    }

    #[test]
    fn timestamped_filename_no_extension() {
        let out = super::timestamped_filename("datafile");
        assert!(out.starts_with("datafile_"), "got: {out}");
        assert!(!out.contains('.'), "got: {out}");
    }

    #[test]
    fn timestamped_filename_multiple_dots_uses_last_extension() {
        let out = super::timestamped_filename("my.data.v2.csv");
        assert!(out.starts_with("my.data.v2_"), "got: {out}");
        assert!(out.ends_with(".csv"), "got: {out}");
    }
}
