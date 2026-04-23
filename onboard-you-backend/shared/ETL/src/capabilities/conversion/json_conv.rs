//! JSON → CSV conversion.
//!
//! Accepts:
//! - An array of objects: `[{"name":"Alice","age":30}, ...]`
//! - An object whose value at `table_index` (among its array-valued fields)
//!   is an array of objects: `{"employees":[...], "departments":[...]}`
//!
//! Column names are taken from the keys of the **first** object in the array.
//! Rows are serialised in insertion order.

use onboard_you_models::{Error, Result};
use serde_json::Value;

use super::textract_parser::rows_to_csv_bytes;

/// Convert JSON bytes to `(csv_bytes, column_names)`.
///
/// `table_index` picks which top-level array to use when the root value is an
/// object with multiple array-valued fields.
pub fn to_csv(bytes: &[u8], table_index: usize) -> Result<(Vec<u8>, Vec<String>)> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|e| Error::IngestionError(format!("Invalid JSON: {e}")))?;

    let array = find_array(&value, table_index)?;

    if array.is_empty() {
        return Err(Error::ValidationError(
            "JSON array is empty — no data rows found".into(),
        ));
    }

    // Column names from the first object's keys.
    let columns: Vec<String> = match &array[0] {
        Value::Object(map) => map.keys().cloned().collect(),
        _ => {
            return Err(Error::ValidationError(
                "JSON array elements must be objects with named fields".into(),
            ))
        }
    };

    if columns.is_empty() {
        return Err(Error::ValidationError(
            "JSON objects have no fields".into(),
        ));
    }

    // Serialise every object in column order.
    let rows: Vec<Vec<String>> = std::iter::once(columns.clone())
        .chain(array.iter().map(|item| {
            let obj = item.as_object();
            columns
                .iter()
                .map(|col| {
                    obj.and_then(|m| m.get(col))
                        .map(|v| value_to_string(v))
                        .unwrap_or_default()
                })
                .collect()
        }))
        .collect();

    let csv_bytes = rows_to_csv_bytes(&rows);
    Ok((csv_bytes, columns))
}

fn find_array<'v>(value: &'v Value, index: usize) -> Result<&'v Vec<Value>> {
    match value {
        Value::Array(arr) => Ok(arr),
        Value::Object(map) => {
            let mut arrays: Vec<&Vec<Value>> = map
                .values()
                .filter_map(|v| v.as_array())
                .collect();
            arrays.sort_by_key(|a| a.len()); // predictable ordering by size
            arrays.into_iter().nth(index).ok_or_else(|| {
                Error::ConfigurationError(format!(
                    "JSON root object has fewer than {} array-valued field(s)",
                    index + 1
                ))
            })
        }
        _ => Err(Error::ValidationError(
            "JSON root must be an array or an object containing arrays".into(),
        )),
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(), // nested objects/arrays → JSON string
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_array_of_objects() {
        let json = br#"[{"name":"Alice","age":"30"},{"name":"Bob","age":"25"}]"#;
        let (csv, cols) = to_csv(json, 0).unwrap();
        assert_eq!(cols, vec!["name", "age"]);
        let text = String::from_utf8(csv).unwrap();
        assert!(text.contains("Alice"));
        assert!(text.contains("Bob"));
    }

    #[test]
    fn object_with_array_field() {
        let json = br#"{"employees":[{"id":"1","name":"Alice"}]}"#;
        let (_, cols) = to_csv(json, 0).unwrap();
        assert_eq!(cols, vec!["id", "name"]);
    }

    #[test]
    fn empty_array_errors() {
        let json = br#"[]"#;
        assert!(to_csv(json, 0).is_err());
    }

    #[test]
    fn invalid_json_errors() {
        assert!(to_csv(b"not json", 0).is_err());
    }

    #[test]
    fn missing_field_fills_empty_string() {
        let json = br#"[{"name":"Alice","age":"30"},{"name":"Bob"}]"#;
        let (csv, _) = to_csv(json, 0).unwrap();
        let text = String::from_utf8(csv).unwrap();
        // Bob's age should be an empty cell
        assert!(text.lines().any(|l| l == "Bob,"));
    }
}
