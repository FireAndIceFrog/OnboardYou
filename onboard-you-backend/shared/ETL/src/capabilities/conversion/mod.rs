//! File conversion module — converts any supported file type to CSV bytes.
//!
//! ## Inline conversions (no AWS dependency)
//! These run directly inside `start-conversion` in the API Lambda:
//! - `.csv`            — pass-through; validates and extracts the header row
//! - `.xlsx` / `.xls` — Excel via `calamine`
//! - `.json`           — array-of-objects or object-of-arrays
//! - `.xml`            — flat repeated-element XML records
//!
//! ## Textract conversions (PDF + images)
//! For `.pdf`, `.png`, `.jpg`, `.jpeg`, `.tiff`, `.tif` the caller must use
//! the `TextractRepo` in the API Lambda — this module only handles the pure
//! Rust conversions.

pub mod excel;
pub mod json_conv;
pub mod textract_parser;
pub mod xml_conv;

use onboard_you_models::{Error, Result};

/// Whether a given file extension requires Textract (i.e. cannot be converted
/// inline in pure Rust).
pub fn needs_textract(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().trim_start_matches('.'),
        "pdf" | "png" | "jpg" | "jpeg" | "tiff" | "tif"
    )
}

/// Whether a file is already a CSV (no conversion needed).
pub fn is_csv(filename: &str) -> bool {
    filename.to_lowercase().ends_with(".csv")
}

/// Extract the file extension from a filename (lowercased, without the dot).
pub fn file_extension(filename: &str) -> &str {
    filename
        .rfind('.')
        .map(|i| &filename[i + 1..])
        .unwrap_or("")
}

/// Convert a supported file to `(csv_bytes, column_names)` without AWS.
///
/// Returns `Err` for PDF/image types — those must go through Textract.
pub fn convert_inline(
    filename: &str,
    bytes: &[u8],
    table_index: usize,
) -> Result<(Vec<u8>, Vec<String>)> {
    let ext = file_extension(filename).to_lowercase();
    match ext.as_str() {
        "csv" => {
            let columns = csv_headers(bytes)?;
            Ok((bytes.to_vec(), columns))
        }
        "xlsx" | "xls" => excel::to_csv(bytes, table_index),
        "json" => json_conv::to_csv(bytes, table_index),
        "xml" => xml_conv::to_csv(bytes),
        other => Err(Error::ConfigurationError(format!(
            "Inline conversion not supported for .{other}; use Textract for PDF/image types"
        ))),
    }
}

fn csv_headers(bytes: &[u8]) -> Result<Vec<String>> {
    let text = String::from_utf8_lossy(bytes);
    let first_line = text.lines().next().ok_or_else(|| {
        Error::ValidationError("File is empty — no header row found".into())
    })?;

    let columns: Vec<String> = first_line
        .split(',')
        .map(|col| col.trim().trim_matches('"').to_string())
        .filter(|col| !col.is_empty())
        .collect();

    if columns.is_empty() {
        return Err(Error::ValidationError(
            "CSV header row contains no valid column names".into(),
        ));
    }

    Ok(columns)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_csv_detects_csv() {
        assert!(is_csv("employees.csv"));
        assert!(is_csv("data.CSV"));
        assert!(!is_csv("report.pdf"));
    }

    #[test]
    fn needs_textract_for_pdf_and_images() {
        assert!(needs_textract("pdf"));
        assert!(needs_textract("PNG"));
        assert!(needs_textract(".jpg"));
        assert!(!needs_textract("xlsx"));
        assert!(!needs_textract("xml"));
    }

    #[test]
    fn csv_passthrough() {
        let csv = b"Name,Age\nAlice,30\n";
        let (bytes, cols) = convert_inline("employees.csv", csv, 0).unwrap();
        assert_eq!(cols, vec!["Name", "Age"]);
        assert_eq!(bytes, csv.as_slice());
    }

    #[test]
    fn unsupported_extension_errors() {
        assert!(convert_inline("report.pdf", b"", 0).is_err());
    }

    #[test]
    fn json_inline_conversion() {
        let json = br#"[{"id":"1","name":"Alice"}]"#;
        let (_, cols) = convert_inline("employees.json", json, 0).unwrap();
        assert_eq!(cols, vec!["id", "name"]);
    }

    #[test]
    fn xml_inline_conversion() {
        let xml = b"<Employees><Employee><Id>1</Id><Name>Alice</Name></Employee></Employees>";
        let (_, cols) = convert_inline("employees.xml", xml, 0).unwrap();
        assert!(cols.contains(&"Name".to_string()));
    }
}
