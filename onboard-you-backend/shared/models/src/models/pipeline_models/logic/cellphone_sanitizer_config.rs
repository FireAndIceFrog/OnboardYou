//! Configuration model for the cellphone sanitizer engine.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Configuration for the cellphone sanitizer action.
///
/// | Field              | Type       | Description                                             |
/// |--------------------|------------|---------------------------------------------------------|
/// | `phone_column`     | string     | Column containing the raw phone number                  |
/// | `country_columns`  | `[string]` | Priority-ordered country columns (ISO 2 or 3 values)   |
/// | `output_column`    | string     | Column to write the internationalised number into       |
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct CellphoneSanitizerConfig {
    /// Column holding the raw phone number.
    pub phone_column: String,
    /// Priority-ordered list of columns whose values are ISO 2/3 country
    /// codes.  The first non-null value that resolves to a known calling
    /// code wins.
    pub country_columns: Vec<String>,
    /// Column to write the sanitised international number into.
    pub output_column: String,
}

impl CellphoneSanitizerConfig {
    /// Validate configuration at construction time.
    pub fn validate(&self) -> Result<()> {
        if self.phone_column.is_empty() {
            return Err(Error::ConfigurationError(
                "'phone_column' must not be empty".into(),
            ));
        }
        if self.country_columns.is_empty() {
            return Err(Error::ConfigurationError(
                "'country_columns' must contain at least one column".into(),
            ));
        }
        for (i, col_name) in self.country_columns.iter().enumerate() {
            if col_name.is_empty() {
                return Err(Error::ConfigurationError(format!(
                    "country_columns[{i}] must not be empty"
                )));
            }
        }
        if self.output_column.is_empty() {
            return Err(Error::ConfigurationError(
                "'output_column' must not be empty".into(),
            ));
        }
        Ok(())
    }
}
