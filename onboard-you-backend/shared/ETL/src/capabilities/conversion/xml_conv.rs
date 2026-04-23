//! XML → CSV conversion using a heuristic record-element detector.
//!
//! Strategy:
//! 1. Parse the document with `quick-xml` in event-streaming mode.
//! 2. The "record element" is the most frequently repeated direct child
//!    of the root element.
//! 3. For each record element, collect the text content of its child elements.
//! 4. The distinct child element names (in first-occurrence order) become
//!    the CSV column headers.
//!
//! This handles the most common HR XML export shapes:
//!
//! ```xml
//! <Employees>
//!   <Employee>
//!     <Id>1</Id>
//!     <Name>Alice</Name>
//!   </Employee>
//!   ...
//! </Employees>
//! ```

use std::collections::HashMap;

use onboard_you_models::{Error, Result};
use quick_xml::events::Event;
use quick_xml::Reader;

use super::textract_parser::rows_to_csv_bytes;

/// Convert XML bytes to `(csv_bytes, column_names)`.
pub fn to_csv(bytes: &[u8]) -> Result<(Vec<u8>, Vec<String>)> {
    let records = parse_records(bytes)?;

    if records.is_empty() {
        return Err(Error::ValidationError(
            "XML document contains no repeating record elements".into(),
        ));
    }

    // Collect columns in first-occurrence order across all records.
    let mut columns: Vec<String> = Vec::new();
    let mut seen_cols: std::collections::HashSet<String> = std::collections::HashSet::new();
    for rec in &records {
        for key in rec.keys() {
            if seen_cols.insert(key.clone()) {
                columns.push(key.clone());
            }
        }
    }

    let mut rows: Vec<Vec<String>> = Vec::with_capacity(records.len() + 1);
    rows.push(columns.clone());
    for rec in &records {
        rows.push(
            columns
                .iter()
                .map(|col| rec.get(col).cloned().unwrap_or_default())
                .collect(),
        );
    }

    let csv_bytes = rows_to_csv_bytes(&rows);
    Ok((csv_bytes, columns))
}

/// Returns one `HashMap<field, value>` per record element.
fn parse_records(bytes: &[u8]) -> Result<Vec<HashMap<String, String>>> {
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    // ── Pass 1: count direct children of the root to find the record element name ──

    let mut depth: usize = 0;
    let mut root_child_counts: HashMap<String, usize> = HashMap::new();
    let mut current_elem: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if depth == 1 {
                    *root_child_counts.entry(name.clone()).or_insert(0) += 1;
                }
                current_elem = Some(name);
                depth += 1;
            }
            Ok(Event::End(_)) => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    current_elem = None;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(Error::IngestionError(format!("XML parse error: {e}")));
            }
            _ => {}
        }
        buf.clear();
    }
    let _ = current_elem;

    if root_child_counts.is_empty() {
        return Ok(Vec::new());
    }

    // The record element is whichever root child appears most often.
    let record_elem = root_child_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(name, _)| name)
        .unwrap();

    // ── Pass 2: collect field values for each record element ──

    let mut reader2 = Reader::from_reader(bytes);
    reader2.config_mut().trim_text(true);
    let mut buf2 = Vec::new();

    let mut depth2: usize = 0;
    let mut in_record = false;
    let mut current_record: HashMap<String, String> = HashMap::new();
    let mut current_field: Option<String> = None;
    let mut records: Vec<HashMap<String, String>> = Vec::new();

    loop {
        match reader2.read_event_into(&mut buf2) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                depth2 += 1;
                if depth2 == 2 && name == record_elem {
                    in_record = true;
                    current_record.clear();
                } else if in_record && depth2 == 3 {
                    current_field = Some(name);
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_record {
                    if let Some(field) = &current_field {
                        let text = e.unescape().unwrap_or_default().into_owned();
                        current_record
                            .entry(field.clone())
                            .or_insert_with(|| text.clone());
                        // If already present, append with space (multi-text nodes).
                        if let Some(existing) = current_record.get_mut(field) {
                            if existing != &text {
                                existing.push(' ');
                                existing.push_str(&text);
                            }
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                if in_record && depth2 == 3 {
                    current_field = None;
                }
                if in_record && depth2 == 2 && name == record_elem {
                    records.push(current_record.clone());
                    current_record.clear();
                    in_record = false;
                }
                depth2 = depth2.saturating_sub(1);
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(Error::IngestionError(format!("XML parse error: {e}")));
            }
            _ => {}
        }
        buf2.clear();
    }

    Ok(records)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &[u8] = br#"<?xml version="1.0"?>
<Employees>
  <Employee>
    <Id>1</Id>
    <Name>Alice</Name>
    <Email>alice@example.com</Email>
  </Employee>
  <Employee>
    <Id>2</Id>
    <Name>Bob</Name>
    <Email>bob@example.com</Email>
  </Employee>
</Employees>"#;

    #[test]
    fn parses_two_records() {
        let records = parse_records(SAMPLE_XML).unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn correct_field_values() {
        let records = parse_records(SAMPLE_XML).unwrap();
        assert_eq!(records[0].get("Name").unwrap(), "Alice");
        assert_eq!(records[1].get("Id").unwrap(), "2");
    }

    #[test]
    fn to_csv_produces_header_row() {
        let (csv, cols) = to_csv(SAMPLE_XML).unwrap();
        let text = String::from_utf8(csv).unwrap();
        assert!(cols.contains(&"Name".to_string()));
        assert!(text.lines().next().unwrap().contains("Name"));
    }

    #[test]
    fn to_csv_has_correct_row_count() {
        let (csv, _) = to_csv(SAMPLE_XML).unwrap();
        let text = String::from_utf8(csv).unwrap();
        // 1 header + 2 data rows
        assert_eq!(text.lines().count(), 3);
    }

    #[test]
    fn empty_xml_errors() {
        assert!(to_csv(b"<Root></Root>").is_err());
    }
}
