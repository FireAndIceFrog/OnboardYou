//! Handle Diacritics: Converts non-ASCII characters to ASCII equivalents
//!
//! Ensures usernames and email addresses don't break legacy systems by
//! transliterating characters like `é` → `e`, `ñ` → `n`, `ü` → `u`.
//!
//! Configurable via manifest JSON:
//! ```json
//! {
//!   "columns": ["first_name", "last_name"],
//!   "output_suffix": "_ascii"
//! }
//! ```
//!
//! When `output_suffix` is `null` (the default), columns are replaced
//! in-place. When set, new columns with the suffix are created alongside
//! the originals.

use crate::capabilities::logic::models::HandleDiacriticsConfig;
use crate::capabilities::logic::traits::ColumnCalculator;
use crate::domain::{Error, OnboardingAction, Result, RosterContext};
use polars::prelude::*;

// ---------------------------------------------------------------------------
// Transliteration table
// ---------------------------------------------------------------------------

/// Map a single Unicode character to its closest ASCII equivalent.
fn transliterate_char(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ā' => 'a',
        'Á' | 'À' | 'Â' | 'Ä' | 'Ã' | 'Å' | 'Ā' => 'A',
        'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' => 'e',
        'É' | 'È' | 'Ê' | 'Ë' | 'Ē' | 'Ė' | 'Ę' => 'E',
        'í' | 'ì' | 'î' | 'ï' | 'ī' => 'i',
        'Í' | 'Ì' | 'Î' | 'Ï' | 'Ī' => 'I',
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ō' => 'o',
        'Ó' | 'Ò' | 'Ô' | 'Ö' | 'Õ' | 'Ō' => 'O',
        'ú' | 'ù' | 'û' | 'ü' | 'ū' => 'u',
        'Ú' | 'Ù' | 'Û' | 'Ü' | 'Ū' => 'U',
        'ñ' => 'n',
        'Ñ' => 'N',
        'ç' => 'c',
        'Ç' => 'C',
        'ý' | 'ÿ' => 'y',
        'Ý' => 'Y',
        'ð' => 'd',
        'Ð' => 'D',
        'ø' => 'o',
        'Ø' => 'O',
        'þ' => 't',
        'Þ' => 'T',
        'ł' => 'l',
        'Ł' => 'L',
        'ž' | 'ź' | 'ż' => 'z',
        'Ž' | 'Ź' | 'Ż' => 'Z',
        'š' | 'ś' => 's',
        'Š' | 'Ś' => 'S',
        'č' | 'ć' => 'c',
        'Č' | 'Ć' => 'C',
        'ř' => 'r',
        'Ř' => 'R',
        'ď' | 'đ' => 'd',
        'Ď' | 'Đ' => 'D',
        'ť' => 't',
        'Ť' => 'T',
        'ň' | 'ń' => 'n',
        'Ň' | 'Ń' => 'N',
        other => other,
    }
}

/// Transliterate a full string to ASCII-safe characters.
///
/// Most characters are 1-to-1 via `transliterate_char`, but a few
/// ligatures expand to multiple ASCII characters (e.g. `ß` → `ss`,
/// `æ` → `ae`, `Æ` → `AE`).
fn transliterate(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'ß' => out.push_str("ss"),
            'æ' => out.push_str("ae"),
            'Æ' => out.push_str("AE"),
            other => out.push(transliterate_char(other)),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Transliterates non-ASCII characters in specified columns to ASCII.
///
/// Designed to run early in the pipeline so that downstream actions
/// (username generation, email generation) receive clean ASCII input.
#[derive(Debug)]
pub struct HandleDiacritics {
    config: HandleDiacriticsConfig,
}

impl HandleDiacritics {
    pub fn from_action_config(value: &serde_json::Value) -> Self {
        Self {
            config: HandleDiacriticsConfig::from_json(value),
        }
    }
}

impl ColumnCalculator for HandleDiacritics {
    fn calculate_columns(&self, mut context: RosterContext) -> Result<RosterContext> {
        // When output_suffix is set, new columns are added alongside originals.
        // When None, columns are replaced in-place so the schema is unchanged.
        if let Some(suffix) = &self.config.output_suffix {
            let lf = std::mem::replace(&mut context.data, LazyFrame::default());
            let mut lf = lf;
            for col_name in &self.config.columns {
                let out_name = format!("{col_name}{suffix}");
                lf = lf.with_column(col(col_name.as_str()).alias(out_name.as_str()));
                context.set_field_source(out_name, "handle_diacritics".into());
            }
            context.data = lf;
        }
        Ok(context)
    }
}

impl OnboardingAction for HandleDiacritics {
    fn id(&self) -> &str {
        "handle_diacritics"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            columns = ?self.config.columns,
            "HandleDiacritics: transliterating columns"
        );

        if self.config.columns.is_empty() {
            return Ok(context);
        }

        let df = context
            .data
            .clone()
            .collect()
            .map_err(|e| Error::LogicError(format!("Failed to collect LazyFrame: {e}")))?;

        let mut result_df = df.clone();

        for col_name in &self.config.columns {
            let col = df.column(col_name).map_err(|e| {
                Error::LogicError(format!("Missing column '{col_name}': {e}"))
            })?;
            let ca = col.str().map_err(|e| {
                Error::LogicError(format!("Column '{col_name}' is not string: {e}"))
            })?;

            let transliterated: StringChunked = ca
                .into_iter()
                .map(|opt: Option<&str>| opt.map(transliterate))
                .collect();

            let out_name = match &self.config.output_suffix {
                Some(suffix) => format!("{col_name}{suffix}"),
                None => col_name.clone(),
            };

            let new_col = transliterated.into_column().with_name(out_name.clone().into());

            if self.config.output_suffix.is_some() {
                result_df = result_df.hstack(&[new_col]).map_err(|e| {
                    Error::LogicError(format!("Failed to add column '{out_name}': {e}"))
                })?;
            } else {
                result_df.replace(col_name.as_str(), new_col).map_err(|e| {
                    Error::LogicError(format!(
                        "Failed to replace column '{col_name}': {e}"
                    ))
                })?;
            }

            context.set_field_source(out_name.clone(), "handle_diacritics".into());
        }

        context.data = result_df.lazy();
        Ok(context)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_df() -> DataFrame {
        df! {
            "employee_id" => &["E001", "E002", "E003"],
            "first_name"  => &["José", "François", "Müller"],
            "last_name"   => &["García", "Lefèvre", "Straße"],
        }
        .unwrap()
    }

    #[test]
    fn test_id() {
        let action = HandleDiacritics::from_action_config(&serde_json::json!({}));
        assert_eq!(action.id(), "handle_diacritics");
    }

    #[test]
    fn test_transliterate_function() {
        assert_eq!(transliterate("José"), "Jose");
        assert_eq!(transliterate("François"), "Francois");
        assert_eq!(transliterate("Müller"), "Muller");
        assert_eq!(transliterate("García"), "Garcia");
        assert_eq!(transliterate("Straße"), "Strasse");
        assert_eq!(transliterate("Łódź"), "Lodz");
    }

    #[test]
    fn test_in_place_replacement() {
        let json = serde_json::json!({
            "columns": ["first_name", "last_name"]
        });
        let action = HandleDiacritics::from_action_config(&json);
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let first = df.column("first_name").unwrap().str().unwrap();
        assert_eq!(first.get(0).unwrap(), "Jose");
        assert_eq!(first.get(1).unwrap(), "Francois");

        let last = df.column("last_name").unwrap().str().unwrap();
        assert_eq!(last.get(0).unwrap(), "Garcia");
    }

    #[test]
    fn test_with_output_suffix() {
        let json = serde_json::json!({
            "columns": ["first_name"],
            "output_suffix": "_ascii"
        });
        let action = HandleDiacritics::from_action_config(&json);
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        // Original should still exist
        assert!(df.column("first_name").is_ok());
        // New column should exist
        let ascii = df.column("first_name_ascii").unwrap().str().unwrap();
        assert_eq!(ascii.get(0).unwrap(), "Jose");
    }

    #[test]
    fn test_empty_columns_noop() {
        let action = HandleDiacritics::from_action_config(&serde_json::json!({}));
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");
        // Should pass through unchanged
        let first = df.column("first_name").unwrap().str().unwrap();
        assert_eq!(first.get(0).unwrap(), "José");
    }

    #[test]
    fn test_field_metadata_provenance() {
        let json = serde_json::json!({ "columns": ["first_name"] });
        let action = HandleDiacritics::from_action_config(&json);
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");
        let meta = result
            .field_metadata
            .get("first_name")
            .expect("metadata");
        assert_eq!(meta.source, "handle_diacritics");
    }

    #[test]
    fn test_missing_column_errors() {
        let json = serde_json::json!({ "columns": ["nonexistent"] });
        let action = HandleDiacritics::from_action_config(&json);
        let ctx = RosterContext::new(sample_df().lazy());
        assert!(action.execute(ctx).is_err());
    }
}
