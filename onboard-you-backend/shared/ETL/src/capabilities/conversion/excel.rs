//! Excel → CSV conversion using the `calamine` crate.
//!
//! Supports both `.xlsx` (Office Open XML) and `.xls` (legacy BIFF) formats.
//! The `table_index` parameter selects which sheet to extract (0-based).

use calamine::{open_workbook_auto_from_rs, Data, Reader};
use onboard_you_models::{Error, Result};
use std::io::Cursor;

use super::textract_parser::rows_to_csv_bytes;

/// Convert Excel bytes to `(csv_bytes, column_names)`.
///
/// `table_index` selects the worksheet (default 0 = first sheet).
pub fn to_csv(bytes: &[u8], table_index: usize) -> Result<(Vec<u8>, Vec<String>)> {
    let cursor = Cursor::new(bytes);
    let mut workbook = open_workbook_auto_from_rs(cursor)
        .map_err(|e| Error::IngestionError(format!("Failed to open Excel workbook: {e}")))?;

    let sheet_names = workbook.sheet_names().to_vec();
    let sheet_name = sheet_names.get(table_index).ok_or_else(|| {
        Error::ConfigurationError(format!(
            "Excel workbook has {} sheet(s) but table_index is {}",
            sheet_names.len(),
            table_index
        ))
    })?;

    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|e| Error::IngestionError(format!("Failed to read sheet '{sheet_name}': {e}")))?;

    let rows: Vec<Vec<String>> = range
        .rows()
        .map(|row| {
            row.iter()
                .map(|cell| match cell {
                    Data::String(s) => s.clone(),
                    Data::Float(f) => {
                        // Avoid "1.0" for whole numbers like employee IDs.
                        if f.fract() == 0.0 && f.abs() < 1e15 {
                            format!("{}", *f as i64)
                        } else {
                            f.to_string()
                        }
                    }
                    Data::Int(i) => i.to_string(),
                    Data::Bool(b) => b.to_string(),
                    Data::DateTime(dt) => dt.to_string(),
                    Data::DateTimeIso(s) => s.clone(),
                    Data::DurationIso(s) => s.clone(),
                    Data::Error(e) => format!("#ERR:{e:?}"),
                    Data::Empty => String::new(),
                })
                .collect()
        })
        .collect();

    if rows.is_empty() {
        return Err(Error::ValidationError(
            "Excel sheet is empty — no data found".into(),
        ));
    }

    let columns = rows[0].clone();
    if columns.is_empty() || columns.iter().all(|c| c.is_empty()) {
        return Err(Error::ValidationError(
            "Excel header row contains no column names".into(),
        ));
    }

    let csv_bytes = rows_to_csv_bytes(&rows);
    Ok((csv_bytes, columns))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    // Excel tests require real .xlsx fixture bytes which are binary — we test
    // the error paths with invalid input instead and rely on integration tests
    // for real workbooks.

    use super::*;

    #[test]
    fn empty_bytes_returns_error() {
        let result = to_csv(&[], 0);
        assert!(result.is_err(), "empty bytes should fail");
    }

    #[test]
    fn invalid_bytes_returns_error() {
        let result = to_csv(b"not an excel file", 0);
        assert!(result.is_err());
    }
}
