/// Convert Textract CELL blocks into UTF-8 CSV bytes.
///
/// Pure function — no I/O. Called by `repositories::textract_repository`.
pub fn blocks_to_csv(blocks: &[aws_sdk_textract::types::Block]) -> Vec<u8> {
    let mut grid: Vec<Vec<String>> = Vec::new();

    for block in blocks {
        if block.block_type().map(|t| t.as_str()) != Some("CELL") {
            continue;
        }
        let row = block.row_index().unwrap_or(0) as usize;
        let col = block.column_index().unwrap_or(0) as usize;
        let text = block.text().unwrap_or("").to_string();

        if row == 0 {
            continue;
        }
        while grid.len() < row {
            grid.push(Vec::new());
        }
        let r = &mut grid[row - 1];
        while r.len() < col {
            r.push(String::new());
        }
        if col > 0 {
            r[col - 1] = text;
        }
    }

    let mut out = String::new();
    for row in &grid {
        let line: Vec<String> = row
            .iter()
            .map(|c| {
                if c.contains(',') || c.contains('"') || c.contains('\n') {
                    format!("\"{}\"", c.replace('"', "\"\""))
                } else {
                    c.clone()
                }
            })
            .collect();
        out.push_str(&line.join(","));
        out.push('\n');
    }

    out.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_textract::types::{Block, BlockType};

    fn cell(row: i32, col: i32, text: &str) -> Block {
        Block::builder()
            .block_type(BlockType::Cell)
            .row_index(row)
            .column_index(col)
            .text(text)
            .build()
    }

    // ── blocks_to_csv ────────────────────────────────────────────────────────

    #[test]
    fn simple_2x2_table() {
        let blocks = vec![
            cell(1, 1, "Name"),
            cell(1, 2, "Age"),
            cell(2, 1, "Alice"),
            cell(2, 2, "30"),
        ];
        let csv = String::from_utf8(blocks_to_csv(&blocks)).unwrap();
        assert_eq!(csv, "Name,Age\nAlice,30\n");
    }

    #[test]
    fn cell_with_comma_is_quoted() {
        let blocks = vec![cell(1, 1, "Smith, Alice"), cell(1, 2, "30")];
        let csv = String::from_utf8(blocks_to_csv(&blocks)).unwrap();
        assert_eq!(csv, "\"Smith, Alice\",30\n");
    }

    #[test]
    fn cell_with_quote_is_escaped() {
        let blocks = vec![cell(1, 1, "say \"hi\"")];
        let csv = String::from_utf8(blocks_to_csv(&blocks)).unwrap();
        assert_eq!(csv, "\"say \"\"hi\"\"\"\n");
    }

    #[test]
    fn zero_row_index_is_skipped() {
        let blocks = vec![
            cell(0, 1, "ignored"),
            cell(1, 1, "kept"),
        ];
        let csv = String::from_utf8(blocks_to_csv(&blocks)).unwrap();
        assert_eq!(csv, "kept\n");
    }

    #[test]
    fn non_cell_blocks_are_ignored() {
        let page_block = Block::builder()
            .block_type(BlockType::Page)
            .build();
        let blocks = vec![page_block, cell(1, 1, "data")];
        let csv = String::from_utf8(blocks_to_csv(&blocks)).unwrap();
        assert_eq!(csv, "data\n");
    }

    #[test]
    fn empty_blocks_produces_empty_csv() {
        let csv = blocks_to_csv(&[]);
        assert!(csv.is_empty());
    }
}
