//! CSV upload engine — coordinates presigned URL generation and column discovery.

use crate::dependancies::Dependancies;
use crate::models::{ApiError, CsvColumnsResponse, PresignedUploadResponse};

/// Build the S3 object key from runtime context.
fn s3_key(organization_id: &str, customer_company_id: &str, filename: &str) -> String {
    format!("{organization_id}/{customer_company_id}/{filename}")
}

/// Generate a presigned PUT URL for a CSV upload.
pub async fn presigned_upload(
    state: &Dependancies,
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

    let key = s3_key(organization_id, customer_company_id, filename);

    let upload_url = state.s3_repo.presigned_put_url(&key).await?;

    Ok(PresignedUploadResponse {
        upload_url,
        key,
        filename: filename.to_string(),
    })
}

/// Read the columns from an already-uploaded CSV in S3.
pub async fn read_columns(
    state: &Dependancies,
    organization_id: &str,
    customer_company_id: &str,
    filename: &str,
) -> Result<CsvColumnsResponse, ApiError> {
    let key = s3_key(organization_id, customer_company_id, filename);

    let columns = state.s3_repo.read_csv_headers(&key).await?;

    Ok(CsvColumnsResponse {
        filename: filename.to_string(),
        columns,
    })
}

#[cfg(test)]
mod tests {
    use crate::dependancies::{Dependancies, Env};
    use crate::models::ApiError;
    use crate::repositories::s3_repository::S3Repo;
    use async_trait::async_trait;
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
    }

    async fn build_state(s3: FakeS3) -> Dependancies {
        let mut deps = Dependancies::new(Env::default()).await;
        deps.s3_repo = Arc::new(s3);
        deps
    }

    #[tokio::test]
    async fn presigned_upload_success() {
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
        assert_eq!(out.filename, "file.csv");
        assert_eq!(out.key, "org/comp/file.csv");
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

    #[tokio::test]
    async fn read_columns_success() {
        let headers = vec!["a".into(), "b".into()];
        let state = build_state(FakeS3 {
            presign: None,
            headers: Some(headers.clone()),
            err: None,
        })
        .await;

        let out = super::read_columns(&state, "org", "comp", "file.csv")
            .await
            .unwrap();
        assert_eq!(out.filename, "file.csv");
        assert_eq!(out.columns, headers);
    }
}
