//! Auto-generated schema documentation from utoipa `ToSchema` derives.
//!
//! Each resource is a JSON Schema produced at runtime from the shared
//! models crate — so the MCP resource descriptions stay in sync with
//! the Rust types automatically.

use onboard_you_models::*;
use utoipa::PartialSchema;

pub struct SchemaResource {
    pub uri: String,
    pub name: &'static str,
    pub description: &'static str,
    pub text: String,
}

/// Helper: render a utoipa schema to pretty-printed JSON.
fn schema_json<T: PartialSchema>() -> String {
    let schema = T::schema();
    serde_json::to_string_pretty(&schema).unwrap_or_default()
}

/// Build the full list of schema resources from the shared model types.
pub fn build_schema_resources() -> Vec<SchemaResource> {
    vec![
        SchemaResource {
            uri: "onboardyou://schema/manifest".into(),
            name: "Manifest",
            description: "ETL manifest: version + ordered list of actions",
            text: schema_json::<Manifest>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/action-config".into(),
            name: "ActionConfig",
            description: "A single pipeline step: type discriminator + typed config payload",
            text: schema_json::<ActionConfig>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/action-type".into(),
            name: "ActionType",
            description: "Enum of all supported action_type values",
            text: schema_json::<ActionType>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/action-config-payload".into(),
            name: "ActionConfigPayload",
            description: "Union of all action-specific config payloads (oneOf)",
            text: schema_json::<ActionConfigPayload>(),
        },
        // ── Ingress ──────────────────────────────────────────
        SchemaResource {
            uri: "onboardyou://schema/generic-ingestion-connector-config".into(),
            name: "GenericIngestionConnectorConfig",
            description: "Ingress: reads any file (CSV, PDF, XML, Excel, JSON, image) from S3",
            text: schema_json::<GenericIngestionConnectorConfig>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/workday-config".into(),
            name: "WorkdayConfig",
            description: "Ingress: pulls worker data from a Workday HCM tenant",
            text: schema_json::<WorkdayConfig>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/sage-hr-config".into(),
            name: "SageHrConfig",
            description: "Ingress: pulls employee data from Sage HR REST API",
            text: schema_json::<SageHrConfig>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/sage-hr-api-response".into(),
            name: "SageHrApiResponse",
            description: "Sage HR API response envelope: data array + pagination meta",
            text: schema_json::<SageHrApiResponse>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/sage-hr-employee".into(),
            name: "SageHrEmployee",
            description: "Raw employee object from Sage HR API",
            text: schema_json::<SageHrEmployee>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/sage-hr-meta".into(),
            name: "SageHrMeta",
            description: "Pagination metadata from Sage HR API",
            text: schema_json::<SageHrMeta>(),
        },
        // ── Logic ────────────────────────────────────────────
        SchemaResource {
            uri: "onboardyou://schema/scd-type-2-config".into(),
            name: "ScdType2Config",
            description: "Logic: slowly-changing dimension type 2 tracking",
            text: schema_json::<ScdType2Config>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/pii-masking-config".into(),
            name: "PIIMaskingConfig",
            description: "Logic: mask PII columns with configurable strategies",
            text: schema_json::<PIIMaskingConfig>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/dedup-config".into(),
            name: "DedupConfig",
            description: "Logic: de-duplicate rows by identity columns",
            text: schema_json::<DedupConfig>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/regex-replace-config".into(),
            name: "RegexReplaceConfig",
            description: "Logic: regex find-and-replace on a single column",
            text: schema_json::<RegexReplaceConfig>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/iso-country-sanitizer-config".into(),
            name: "IsoCountrySanitizerConfig",
            description: "Logic: normalise country names/codes to ISO standard",
            text: schema_json::<IsoCountrySanitizerConfig>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/cellphone-sanitizer-config".into(),
            name: "CellphoneSanitizerConfig",
            description: "Logic: normalise phone numbers to E.164 using country context",
            text: schema_json::<CellphoneSanitizerConfig>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/handle-diacritics-config".into(),
            name: "HandleDiacriticsConfig",
            description: "Logic: strip or transliterate diacritics (accented chars → ASCII)",
            text: schema_json::<HandleDiacriticsConfig>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/rename-config".into(),
            name: "RenameConfig",
            description: "Logic: rename columns",
            text: schema_json::<RenameConfig>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/drop-config".into(),
            name: "DropConfig",
            description: "Logic: drop columns from the DataFrame",
            text: schema_json::<DropConfig>(),
        },
        SchemaResource {
            uri: "onboardyou://schema/filter-by-value-config".into(),
            name: "FilterByValueConfig",
            description: "Logic: keep only rows matching a regex on a column",
            text: schema_json::<FilterByValueConfig>(),
        },
        // ── Egress ───────────────────────────────────────────
        SchemaResource {
            uri: "onboardyou://schema/api-dispatcher-config".into(),
            name: "ApiDispatcherConfig",
            description: "Egress: dispatch processed data to an external API (discriminated by auth_type)",
            text: schema_json::<ApiDispatcherConfig>(),
        },
    ]
}
