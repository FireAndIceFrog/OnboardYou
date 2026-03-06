//! Manifest: Versioned, declarative pipeline configuration
//!
//! Defines the schema for JSON/YAML pipeline configs that determine execution order

use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

/// All known action types in the pipeline.
///
/// This is the **single source of truth** — adding a new capability requires
/// adding a variant here, which then forces the factory to handle it
/// (exhaustive match). Serde maps these to/from `snake_case` JSON strings.
///
/// | JSON value                | Variant                |
/// |---------------------------|------------------------|
/// | `"csv_hris_connector"`    | `CsvHrisConnector`     |
/// | `"workday_hris_connector"`| `WorkdayHrisConnector` |
/// | `"sage_hr_connector"`     | `SageHrConnector`      |
/// | `"scd_type_2"`            | `ScdType2`             |
/// | `"pii_masking"`           | `PiiMasking`           |
/// | `"identity_deduplicator"` | `IdentityDeduplicator` |
/// | `"regex_replace"`         | `RegexReplace`         |
/// | `"iso_country_sanitizer"` | `IsoCountrySanitizer`  |
/// | `"cellphone_sanitizer"`   | `CellphoneSanitizer`   |
/// | `"handle_diacritics"`     | `HandleDiacritics`     |
/// | `"rename_column"`         | `RenameColumn`         |
/// | `"drop_column"`           | `DropColumn`           |
/// | `"filter_by_value"`       | `FilterByValue`        |
/// | `"api_dispatcher"`        | `ApiDispatcher`        |
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    CsvHrisConnector,
    WorkdayHrisConnector,
    SageHrConnector,
    #[serde(rename = "scd_type_2")]
    ScdType2,
    PiiMasking,
    IdentityDeduplicator,
    RegexReplace,
    IsoCountrySanitizer,
    CellphoneSanitizer,
    HandleDiacritics,
    RenameColumn,
    DropColumn,
    FilterByValue,
    ApiDispatcher,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Serialize to the same snake_case string used in JSON
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{:?}", self));
        f.write_str(&s)
    }
}

/// Version of the manifest schema
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Manifest {
    /// Schema version (e.g. "1.0")
    pub version: String,
    /// Ordered list of pipeline actions
    pub actions: Vec<ActionConfig>,
}

/// Configuration for a single action in the pipeline
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct ActionConfig {
    /// Unique identifier for this pipeline step
    pub id: String,
    /// Factory key — selects the Rust implementation (e.g. `ActionType::CsvHrisConnector`)
    pub action_type: ActionType,
    /// Action-specific configuration (shape depends on action_type)
    ///
    /// Deserialized into the concrete typed variant matching `action_type`
    /// via the custom `Deserialize` impl below.
    pub config: ActionConfigPayload,
}

/// Custom deserialiser for [`ActionConfig`].
///
/// Reads `action_type` first, then uses it to direct the `config` blob
/// into the correct [`ActionConfigPayload`] variant.
impl<'de> Deserialize<'de> for ActionConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            id: String,
            action_type: ActionType,
            #[serde(default)]
            config: serde_json::Value,
        }

        let raw = Raw::deserialize(deserializer)?;

        let config = match raw.action_type {
            ActionType::CsvHrisConnector => ActionConfigPayload::CsvHrisConnector(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
            ActionType::WorkdayHrisConnector => ActionConfigPayload::WorkdayHrisConnector(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
            ActionType::SageHrConnector => ActionConfigPayload::SageHrConnector(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
            ActionType::ScdType2 => ActionConfigPayload::ScdType2(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
            ActionType::PiiMasking => ActionConfigPayload::PiiMasking(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
            ActionType::IdentityDeduplicator => ActionConfigPayload::IdentityDeduplicator(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
            ActionType::RegexReplace => ActionConfigPayload::RegexReplace(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
            ActionType::IsoCountrySanitizer => ActionConfigPayload::IsoCountrySanitizer(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
            ActionType::CellphoneSanitizer => ActionConfigPayload::CellphoneSanitizer(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
            ActionType::HandleDiacritics => ActionConfigPayload::HandleDiacritics(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
            ActionType::RenameColumn => ActionConfigPayload::RenameColumn(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
            ActionType::DropColumn => ActionConfigPayload::DropColumn(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
            ActionType::FilterByValue => ActionConfigPayload::FilterByValue(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
            ActionType::ApiDispatcher => ActionConfigPayload::ApiDispatcher(
                serde_json::from_value(raw.config).map_err(serde::de::Error::custom)?,
            ),
        };

        Ok(ActionConfig {
            id: raw.id,
            action_type: raw.action_type,
            config,
        })
    }
}

/// Wrapper enum for action-specific config payloads.
///
/// Each variant holds the **concrete, typed** configuration struct for
/// its `ActionType`.  The `#[serde(untagged)]` attribute produces flat
/// JSON serialisation (no wrapper key), while `ToSchema` generates a
/// `oneOf` schema listing every config variant for OpenAPI.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(untagged)]
pub enum ActionConfigPayload {
    CsvHrisConnector(crate::CsvHrisConnectorConfig),
    WorkdayHrisConnector(crate::WorkdayConfig),
    SageHrConnector(crate::SageHrConfig),
    
    ScdType2(crate::ScdType2Config),
    PiiMasking(crate::PIIMaskingConfig),
    IdentityDeduplicator(crate::DedupConfig),
    RegexReplace(crate::RegexReplaceConfig),
    IsoCountrySanitizer(crate::IsoCountrySanitizerConfig),
    CellphoneSanitizer(crate::CellphoneSanitizerConfig),
    HandleDiacritics(crate::HandleDiacriticsConfig),
    RenameColumn(crate::RenameConfig),
    DropColumn(crate::DropConfig),
    FilterByValue(crate::FilterByValueConfig),

    ApiDispatcher(crate::ApiDispatcherConfig),
}

impl Manifest {
    /// Parse a manifest from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize manifest to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_deserialization() {
        let json = r#"{
            "version": "1.0",
            "actions": [
                {
                    "id": "ingest_hris",
                    "action_type": "csv_hris_connector",
                    "config": { "filename": "data.csv", "columns": ["a", "b"] }
                }
            ]
        }"#;

        let manifest = Manifest::from_json(json).expect("Failed to parse manifest");
        assert_eq!(manifest.version, "1.0");
        assert_eq!(manifest.actions.len(), 1);
        assert_eq!(
            manifest.actions[0].action_type,
            ActionType::CsvHrisConnector
        );
    }

    #[test]
    fn test_unknown_action_type_rejected() {
        let json = r#"{
            "version": "1.0",
            "actions": [
                {
                    "id": "bad",
                    "action_type": "does_not_exist",
                    "config": {}
                }
            ]
        }"#;

        let result = Manifest::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_action_type_display() {
        assert_eq!(
            ActionType::CsvHrisConnector.to_string(),
            "csv_hris_connector"
        );
        assert_eq!(ActionType::ScdType2.to_string(), "scd_type_2");
        assert_eq!(ActionType::ApiDispatcher.to_string(), "api_dispatcher");
    }
}
