//! Pure Textract block parsing — no AWS SDK dependency.
//!
//! The API Lambda calls Textract and receives a flat list of `Block` objects.
//! This module converts those blocks into a simple 2-D table (`Vec<Vec<String>>`)
//! that the rest of the pipeline can work with.
//!
//! The AWS SDK types are NOT imported here — the caller passes the data as
//! plain slices so this module stays pure and fully unit-testable.

use std::collections::HashMap;

use onboard_you_models::{Error, Result};

/// A single Textract block, normalised to only the fields we need.
#[derive(Debug, Clone)]
pub struct TextractBlock {
    pub id: String,
    pub block_type: String,
    pub text: Option<String>,
    pub row_index: Option<i32>,
    pub column_index: Option<i32>,
    /// IDs of child blocks (CHILD relationship).
    pub child_ids: Vec<String>,
}

/// Extract all tables from a list of Textract blocks.
///
/// Each table is a `Vec<Vec<String>>` where the first row is the header row.
/// Tables are ordered as they appear in the document.
pub fn extract_tables(blocks: &[TextractBlock]) -> Vec<Vec<Vec<String>>> {
    let by_id: HashMap<&str, &TextractBlock> =
        blocks.iter().map(|b| (b.id.as_str(), b)).collect();

    let tables: Vec<&TextractBlock> = blocks
        .iter()
        .filter(|b| b.block_type.eq_ignore_ascii_case("TABLE"))
        .collect();

    tables
        .iter()
        .map(|table| extract_table(table, &by_id))
        .collect()
}

fn extract_table(table: &TextractBlock, by_id: &HashMap<&str, &TextractBlock>) -> Vec<Vec<String>> {
    // Build a (row, col) → text map from the table's CELL children.
    let mut grid: HashMap<(i32, i32), String> = HashMap::new();
    let mut max_row = 0i32;
    let mut max_col = 0i32;

    for cell_id in &table.child_ids {
        let Some(cell) = by_id.get(cell_id.as_str()) else {
            continue;
        };
        if !cell.block_type.eq_ignore_ascii_case("CELL") {
            continue;
        }

        let row = cell.row_index.unwrap_or(0);
        let col = cell.column_index.unwrap_or(0);
        max_row = max_row.max(row);
        max_col = max_col.max(col);

        // Concatenate WORD children to form the cell text.
        let text = cell
            .child_ids
            .iter()
            .filter_map(|word_id| by_id.get(word_id.as_str()))
            .filter(|b| b.block_type.eq_ignore_ascii_case("WORD"))
            .filter_map(|b| b.text.as_deref())
            .collect::<Vec<_>>()
            .join(" ");

        grid.insert((row, col), text);
    }

    // Convert the grid to row-major order (1-indexed from Textract).
    (1..=max_row)
        .map(|row| {
            (1..=max_col)
                .map(|col| grid.remove(&(row, col)).unwrap_or_default())
                .collect()
        })
        .collect()
}

/// Serialise a 2-D table to RFC 4180 CSV bytes.
/// The first row is treated as headers (no special treatment — just written as-is).
pub fn rows_to_csv_bytes(rows: &[Vec<String>]) -> Vec<u8> {
    let mut out = Vec::new();
    for row in rows {
        let line = row
            .iter()
            .map(|cell| {
                if cell.contains(',') || cell.contains('"') || cell.contains('\n') {
                    format!("\"{}\"", cell.replace('"', "\"\""))
                } else {
                    cell.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        out.extend_from_slice(line.as_bytes());
        out.push(b'\n');
    }
    out
}

/// Pick one table by index from the result of `extract_tables`.
pub fn pick_table(
    tables: Vec<Vec<Vec<String>>>,
    table_index: usize,
) -> Result<Vec<Vec<String>>> {
    let n = tables.len();
    tables.into_iter().nth(table_index).ok_or_else(|| {
        Error::ConfigurationError(format!(
            "Textract found {n} table(s) but table_index is {table_index}"
        ))
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_block(
        id: &str,
        block_type: &str,
        text: Option<&str>,
        row: Option<i32>,
        col: Option<i32>,
        children: &[&str],
    ) -> TextractBlock {
        TextractBlock {
            id: id.into(),
            block_type: block_type.into(),
            text: text.map(Into::into),
            row_index: row,
            column_index: col,
            child_ids: children.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Build a minimal Textract block list for a 2×2 table:
    ///
    ///   | Name  | Age |
    ///   | Alice | 30  |
    fn two_by_two_blocks() -> Vec<TextractBlock> {
        vec![
            // Words
            make_block("w1", "WORD", Some("Name"), None, None, &[]),
            make_block("w2", "WORD", Some("Age"), None, None, &[]),
            make_block("w3", "WORD", Some("Alice"), None, None, &[]),
            make_block("w4", "WORD", Some("30"), None, None, &[]),
            // Cells
            make_block("c1", "CELL", None, Some(1), Some(1), &["w1"]),
            make_block("c2", "CELL", None, Some(1), Some(2), &["w2"]),
            make_block("c3", "CELL", None, Some(2), Some(1), &["w3"]),
            make_block("c4", "CELL", None, Some(2), Some(2), &["w4"]),
            // Table
            make_block("t1", "TABLE", None, None, None, &["c1", "c2", "c3", "c4"]),
        ]
    }

    #[test]
    fn extract_tables_finds_one_table() {
        let blocks = two_by_two_blocks();
        let tables = extract_tables(&blocks);
        assert_eq!(tables.len(), 1);
    }

    #[test]
    fn extract_table_correct_dimensions() {
        let tables = extract_tables(&two_by_two_blocks());
        let table = &tables[0];
        assert_eq!(table.len(), 2, "should have 2 rows");
        assert_eq!(table[0].len(), 2, "should have 2 columns");
    }

    #[test]
    fn extract_table_correct_values() {
        let tables = extract_tables(&two_by_two_blocks());
        let table = &tables[0];
        assert_eq!(table[0], vec!["Name", "Age"]);
        assert_eq!(table[1], vec!["Alice", "30"]);
    }

    #[test]
    fn rows_to_csv_basic() {
        let rows = vec![
            vec!["Name".into(), "Age".into()],
            vec!["Alice".into(), "30".into()],
        ];
        let csv = rows_to_csv_bytes(&rows);
        assert_eq!(String::from_utf8(csv).unwrap(), "Name,Age\nAlice,30\n");
    }

    #[test]
    fn rows_to_csv_escapes_commas() {
        let rows = vec![vec!["Last, First".into(), "30".into()]];
        let csv = String::from_utf8(rows_to_csv_bytes(&rows)).unwrap();
        assert!(csv.contains("\"Last, First\""));
    }

    #[test]
    fn pick_table_out_of_range_errors() {
        let tables: Vec<Vec<Vec<String>>> = vec![vec![]];
        assert!(pick_table(tables, 5).is_err());
    }
}
