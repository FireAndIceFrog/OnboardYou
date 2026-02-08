//! Mock CSV and JSON data for testing

use std::io::Write;
use std::path::PathBuf;

/// Sample HRIS data for testing (7 columns, 3 rows)
pub const SAMPLE_HRIS_CSV: &str = "\
employee_id,first_name,last_name,email,ssn,salary,start_date
001,John,Doe,john.doe@example.com,123-45-6789,75000,2024-01-01
002,Jane,Smith,jane.smith@example.com,987-65-4321,85000,2024-02-15
003,Alice,Johnson,alice.j@example.com,555-12-3456,92000,2024-03-10
";

/// Write `SAMPLE_HRIS_CSV` to a temp file and return the handle + path.
///
/// The caller **must** keep the returned `NamedTempFile` alive for the
/// duration of the test so the OS doesn't delete the file.
pub fn write_sample_csv() -> (tempfile::NamedTempFile, PathBuf) {
    let mut tmp = tempfile::NamedTempFile::new().expect("create temp file");
    tmp.write_all(SAMPLE_HRIS_CSV.as_bytes())
        .expect("write csv");
    let path = tmp.path().to_path_buf();
    (tmp, path)
}

/// Manifest that uses the CSV connector, pointing at a given path.
pub fn sample_csv_manifest(csv_path: &str) -> String {
    format!(
        r#"{{
  "version": "1.0",
  "actions": [
    {{
      "id": "ingest_hris",
      "action_type": "csv_hris_connector",
      "config": {{ "csv_path": "{}" }}
    }}
  ]
}}"#,
        csv_path
    )
}

/// Original sample manifest (kept for backward compatibility)
pub const SAMPLE_MANIFEST_JSON: &str = r#"{
  "version": "1.0",
  "actions": [
    {
      "id": "hris_connector",
      "action_type": "csv_hris_connector",
      "config": {}
    },
    {
      "id": "validator",
      "action_type": "validation",
      "config": {}
    }
  ]
}"#;
