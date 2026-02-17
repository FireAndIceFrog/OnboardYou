//! Configuration model for the ISO country sanitizer engine.

use crate::domain::{Error, Result};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Desired output ISO code format.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum CountryOutputFormat {
    Alpha2,
    Alpha3,
}

/// Configuration for the ISO country sanitizer action.
///
/// | Field           | Type   | Description                                          |
/// |-----------------|--------|------------------------------------------------------|
/// | `source_column` | string | Column containing the raw country value               |
/// | `output_column` | string | Column to write the normalised ISO code into          |
/// | `output_format` | string | `"alpha2"` or `"alpha3"`                              |
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct IsoCountrySanitizerConfig {
    /// Column to read the raw country value from.
    pub source_column: String,
    /// Column to write the normalised code to.
    pub output_column: String,
    /// Desired output format.
    pub output_format: CountryOutputFormat,
}

impl IsoCountrySanitizerConfig {
    /// Validate configuration at construction time.
    pub fn validate(&self) -> Result<()> {
        if self.source_column.is_empty() {
            return Err(Error::ConfigurationError(
                "iso_country_sanitizer: 'source_column' must not be empty".into(),
            ));
        }
        if self.output_column.is_empty() {
            return Err(Error::ConfigurationError(
                "iso_country_sanitizer: 'output_column' must not be empty".into(),
            ));
        }
        Ok(())
    }
}
