//! Orchestrates converting an uploaded non-CSV file into a CSV stored back in S3.
//!
//! ## Flow
//! 1. Inspect the file extension of `original_key`.
//! 2. If it is a CSV → just read the headers (nothing to convert).
//! 3. If it is a PDF or image → call Textract, parse blocks, write CSV to S3.
//! 4. If it is Excel / JSON / XML → download the file, convert inline, write CSV to S3.
//!
//! Returns the CSV column names on success.

use onboard_you::capabilities::conversion::{
    convert_inline, file_extension, is_csv, needs_textract,
    textract_parser::{extract_tables, pick_table, rows_to_csv_bytes},
};

use crate::{
    models::ApiError,
    repositories::{
        s3_repository::S3Repo, textract_repository::TextractRepo,
    },
};

/// Convert `original_key` in S3 to a CSV stored at `csv_key`.
///
/// - `original_key` — S3 key of the user-uploaded file (e.g. `org/co/data.xlsx`)
/// - `csv_key`      — Where to write the resulting CSV (e.g. `org/co/data.csv`)
/// - `table_index`  — Which sheet / table / array to extract (0-based)
///
/// Returns the column headers of the generated CSV.
pub async fn convert_to_csv(
    s3_repo: &dyn S3Repo,
    textract_repo: &dyn TextractRepo,
    original_key: &str,
    csv_key: &str,
    table_index: usize,
) -> Result<Vec<String>, ApiError> {
    let ext = file_extension(original_key);

    // ── CSV: already the right format ──────────────────────────────────────
    if is_csv(original_key) {
        return s3_repo.read_csv_headers(original_key).await;
    }

    // ── PDF / images: Textract ─────────────────────────────────────────────
    if needs_textract(ext) {
        let blocks = textract_repo.analyze_s3_document(original_key).await?;
        let tables = extract_tables(&blocks);
        let rows = pick_table(tables, table_index).map_err(|e| {
            ApiError::Validation(format!("Textract table selection failed: {e}"))
        })?;

        if rows.is_empty() {
            return Err(ApiError::Validation(
                "Textract found no data rows in the document".into(),
            ));
        }

        let columns = rows[0].clone();
        let csv_bytes = rows_to_csv_bytes(&rows);
        s3_repo
            .put_object_bytes(csv_key, csv_bytes, "text/csv")
            .await?;

        return Ok(columns);
    }

    // ── Excel / JSON / XML: inline conversion ─────────────────────────────
    let bytes = s3_repo.get_object_bytes(original_key).await?;
    let filename = original_key.rsplit('/').next().unwrap_or(original_key);

    let (csv_bytes, columns) = convert_inline(filename, &bytes, table_index).map_err(|e| {
        ApiError::Validation(format!("File conversion failed: {e}"))
    })?;

    s3_repo
        .put_object_bytes(csv_key, csv_bytes, "text/csv")
        .await?;

    Ok(columns)
}
