//! Plan prompt — generates the AI prompt for ETL pipeline plan generation.
//!
//! Uses the [`action_schema!`] macro to validate config field names against
//! the actual struct definitions at compile time. If a config field is renamed
//! or removed, this module will fail to compile.

use std::collections::HashMap;

/// Generates action schema text with compile-time field name validation.
///
/// Each `$field` ident is checked against the real struct — if a field is
/// renamed or removed the build breaks immediately.
macro_rules! action_schema {
    ($struct:ty, $action_type:literal, $desc:literal, {
        $($field:ident : $fdesc:literal),* $(,)?
    }) => {{
        // Compile-time: assert every listed field actually exists on the struct.
        const _: () = {
            #[allow(unused)]
            fn _assert_fields_exist(s: &$struct) {
                $( let _ = &s.$field; )*
            }
        };
        concat!(
            "### ", $action_type, "\n",
            $desc, "\n",
            "Config: {\n",
            $( "  \"", stringify!($field), "\": ", $fdesc, "\n", )*
            "}\n"
        )
    }};
}

/// Validates field names exist on a struct at compile time (for nested / enum-variant types
/// whose schema text is written manually).
macro_rules! validate_fields {
    ($struct:ty { $($field:ident),* $(,)? }) => {
        const _: () = {
            #[allow(unused)]
            fn _assert_fields_exist(s: &$struct) {
                $( let _ = &s.$field; )*
            }
        };
    };
}

// ── Compile-time validation for nested / enum-variant types ────────
// These types appear inside parent descriptions but don't get their own
// `action_schema!` call, so we validate their fields separately.

validate_fields!(crate::WorkdayResponseGroup {
    include_personal_information,
    include_employment_information,
    include_compensation,
    include_organizations,
    include_roles,
});

validate_fields!(crate::ColumnMask { name, strategy });

// ApiDispatcherConfig is an enum — validate each variant's inner struct.
validate_fields!(crate::BearerRepoConfig {
    destination_url, token, placement, placement_key, extra_headers,
});
validate_fields!(crate::OAuthRepoConfig {
    destination_url, consumer_key, consumer_secret, access_token, token_secret,
});
validate_fields!(crate::OAuth2RepoConfig {
    destination_url, client_id, client_secret, token_url, scopes, grant_type, refresh_token,
});

/// Hand-written schema for `api_dispatcher` since [`ApiDispatcherConfig`] is an enum
/// with multiple auth variants. Field names are validated above via [`validate_fields!`].
const API_DISPATCHER_SCHEMA: &str = r#"### api_dispatcher
Sends transformed data to the destination API. Supports Bearer, OAuth 1.0, and OAuth 2.0.
Config (auth_type: "bearer"): {
  "destination_url": "string" — Target API endpoint
  "token": "string" | null — Bearer token
  "placement": "authorization_header" | "custom_header" | "query_param" (default: authorization_header)
  "placement_key": "string" | null — Header/param name when not using Authorization header
  "extra_headers": { "header": "value" } — Additional HTTP headers
  "schema": { "source_col": "dest_field" } — Column mapping
}
Config (auth_type: "oauth"): {
  "destination_url": "string"
  "consumer_key": "string"
  "consumer_secret": "string"
  "access_token": "string"
  "token_secret": "string"
  "schema": { "source_col": "dest_field" }
}
Config (auth_type: "oauth2"): {
  "destination_url": "string"
  "client_id": "string"
  "client_secret": "string"
  "token_url": "string"
  "scopes": ["string"]
  "grant_type": "client_credentials" | "authorization_code" (default: client_credentials)
  "refresh_token": "string" | null
  "schema": { "source_col": "dest_field" }
}
Config (auth_type: "default"): {
  "schema": { "source_col": "dest_field" }
}
"#;

/// Context needed to generate an AI prompt for pipeline plan generation.
pub struct PlanPrompt<'a> {
    pub source_system: &'a str,
    pub final_columns: &'a [String],
    pub schema_diff: &'a str,
    pub egress_schema: &'a HashMap<String, String>,
}

/// A system/user message pair ready to be sent to an LLM.
pub struct PromptMessages {
    pub system: String,
    pub user: String,
}

impl<'a> PlanPrompt<'a> {
    /// Generate the full prompt messages including the complete ETL action schema.
    pub fn generate_prompt(&self) -> PromptMessages {
        let system = self.build_system_message();
        let user = self.build_user_message();
        PromptMessages { system, user }
    }

    fn build_system_message(&self) -> String {
        format!(
            r#"You are an ETL pipeline planner for an HR data integration tool called OnboardYou.
Your job is to generate a plan that transforms employee data from a source system into
the format expected by the customer's destination API.

{etl_schema}

RULES:
1. The first action MUST be the appropriate ingress connector ({source_system}).
2. The last action MUST be api_dispatcher (egress).
3. Include transformation actions between ingress and egress as needed.
4. Each action must have a unique "id" (e.g. "step_1", "step_2").
5. Each action has "disabled": false by default.
6. The "config" object for each action MUST conform to the schema above.
7. Generate a summary with toggleable features, each referencing the action IDs it controls.
8. Generate a synthetic preview with realistic sample data.
9. Return ONLY valid JSON matching the response schema below. No markdown, no explanation.

RESPONSE SCHEMA:
{{
  "manifest": {{
    "version": "1.0",
    "actions": [
      {{ "id": "string", "action_type": "string", "disabled": false, "config": {{...}} }}
    ]
  }},
  "summary": {{
    "headline": "string",
    "description": "string",
    "features": [
      {{
        "id": "string",
        "icon": "string (calendar|users|shield|filter|columns|globe|phone|lock|edit|zap)",
        "label": "string",
        "description": "string",
        "action_ids": ["string"]
      }}
    ],
    "preview": {{
      "source_label": "string",
      "target_label": "string",
      "before": {{ "field": "value" }},
      "after": {{ "field": "value" }}
    }}
  }}
}}"#,
            source_system = self.source_system,
            etl_schema = Self::etl_schema(),
        )
    }

    fn build_user_message(&self) -> String {
        format!(
            r#"Generate a pipeline plan for the following scenario:

SOURCE SYSTEM: {source_system}
PIPELINE COLUMNS (after ingress): {columns}
EGRESS SCHEMA (destination API fields): {egress}
SCHEMA DIFF:
{schema_diff}

Create an intelligent pipeline that:
1. Ingests from {source_system}
2. Applies appropriate data quality transformations
3. Maps source columns to destination fields
4. Dispatches to the destination API

Return the JSON plan."#,
            source_system = self.source_system,
            columns = if self.final_columns.is_empty() {
                "(not yet determined — use typical columns for the source system)".to_string()
            } else {
                self.final_columns.join(", ")
            },
            egress = if self.egress_schema.is_empty() {
                "(not yet configured — generate reasonable defaults)".to_string()
            } else {
                serde_json::to_string_pretty(self.egress_schema).unwrap_or_default()
            },
            schema_diff = if self.schema_diff.is_empty() {
                "(no diff available)"
            } else {
                self.schema_diff
            },
        )
    }

    /// Build the complete ETL action schema from the actual config struct definitions.
    ///
    /// Each `action_schema!` call validates field names against the real Rust struct
    /// at compile time. If a field is renamed or removed, this function won't compile.
    fn etl_schema() -> String {
        [
            "AVAILABLE ACTIONS AND CONFIG SCHEMAS:\n",

            "\n## INGRESS ACTIONS (pick ONE as the first action)\n",

            action_schema!(crate::CsvHrisConnectorConfig, "csv_hris_connector",
                "Ingests employee data from a CSV file.", {
                    filename: "\"string\" — Name of the CSV file",
                    columns: "[\"string\"] — Column names to ingest",
                }
            ),

            action_schema!(crate::WorkdayConfig, "workday_hris_connector",
                "Ingests employee data from the Workday SOAP API.", {
                    tenant_url: "\"string\" — Workday tenant URL",
                    tenant_id: "\"string\" — Workday tenant ID",
                    username: "\"string\" — Service account username",
                    password: "\"string\" — Service account password",
                    worker_count_limit: "number (default: 200) — Max workers per request",
                    response_group: "{ include_personal_information: bool (true), include_employment_information: bool (true), include_compensation: bool (false), include_organizations: bool (false), include_roles: bool (false) }",
                }
            ),

            "\n## TRANSFORMATION ACTIONS (zero or more, in any order)\n",

            action_schema!(crate::ScdType2Config, "scd_type_2",
                "Slowly Changing Dimension Type 2 — tracks historical changes to entities.", {
                    entity_column: "\"string\" (default: \"employee_id\") — Entity identifier column",
                    date_column: "\"string\" (default: \"start_date\") — Effective date column",
                }
            ),

            action_schema!(crate::PIIMaskingConfig, "pii_masking",
                "Masks PII fields (e.g. SSN, salary) using configurable strategies.", {
                    columns: "[{ name: \"string\", strategy: { \"Redact\": { keep_last: number, mask_prefix: \"string\" } } | \"Zero\" }]",
                }
            ),

            action_schema!(crate::DedupConfig, "identity_deduplicator",
                "Deduplicates records based on matching columns.", {
                    columns: "[\"string\"] (default: [\"national_id\", \"email\"]) — Match columns",
                    employee_id_column: "\"string\" (default: \"employee_id\") — Primary key column",
                }
            ),

            action_schema!(crate::RegexReplaceConfig, "regex_replace",
                "Applies regex find-and-replace on a column.", {
                    column: "\"string\" — Target column",
                    pattern: "\"string\" — Regex pattern (Rust syntax, max 128 chars)",
                    replacement: "\"string\" — Replacement text (max 256 chars)",
                }
            ),

            action_schema!(crate::IsoCountrySanitizerConfig, "iso_country_sanitizer",
                "Normalizes country codes to ISO 3166 format.", {
                    source_column: "\"string\" — Input column with raw country values",
                    output_column: "\"string\" — Output column for normalized codes",
                    output_format: "\"alpha2\" | \"alpha3\"",
                }
            ),

            action_schema!(crate::CellphoneSanitizerConfig, "cellphone_sanitizer",
                "Normalizes phone numbers to E.164 format.", {
                    phone_column: "\"string\" — Column with phone numbers",
                    country_columns: "[\"string\"] — Priority-ordered ISO country code columns",
                    output_column: "\"string\" — Output column for normalized numbers",
                }
            ),

            action_schema!(crate::HandleDiacriticsConfig, "handle_diacritics",
                "Normalizes diacritical characters (e.g. é → e, ñ → n).", {
                    columns: "[\"string\"] — Columns to normalize (default: all)",
                    output_suffix: "\"string\" | null — Suffix for output column, or null for in-place",
                }
            ),

            action_schema!(crate::RenameConfig, "rename_column",
                "Renames columns via a source → target mapping.", {
                    mapping: "{ \"old_name\": \"new_name\" } — Source to target column mapping",
                }
            ),

            action_schema!(crate::DropConfig, "drop_column",
                "Drops specified columns from the dataset.", {
                    columns: "[\"string\"] — Columns to remove",
                }
            ),

            action_schema!(crate::FilterByValueConfig, "filter_by_value",
                "Filters rows by regex. Matching rows are kept.", {
                    column: "\"string\" — Column to filter on",
                    pattern: "\"string\" — Regex pattern; matching rows kept",
                }
            ),

            "\n## EGRESS ACTION (must be the LAST action)\n",

            API_DISPATCHER_SCHEMA,
        ]
        .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_prompt_includes_full_schema() {
        let egress: HashMap<String, String> =
            [("name".into(), "fullName".into())].into_iter().collect();

        let prompt = PlanPrompt {
            source_system: "Workday",
            final_columns: &["name".into(), "email".into()],
            schema_diff: "  MAPPED: name → fullName",
            egress_schema: &egress,
        };

        let messages = prompt.generate_prompt();

        // System message includes full ETL schema
        assert!(messages.system.contains("csv_hris_connector"));
        assert!(messages.system.contains("workday_hris_connector"));
        assert!(messages.system.contains("pii_masking"));
        assert!(messages.system.contains("api_dispatcher"));
        assert!(messages.system.contains("iso_country_sanitizer"));
        assert!(messages.system.contains("cellphone_sanitizer"));
        assert!(messages.system.contains("handle_diacritics"));
        assert!(messages.system.contains("regex_replace"));
        assert!(messages.system.contains("scd_type_2"));
        assert!(messages.system.contains("rename_column"));
        assert!(messages.system.contains("drop_column"));
        assert!(messages.system.contains("filter_by_value"));
        assert!(messages.system.contains("identity_deduplicator"));

        // System message includes config field details (validated at compile time)
        assert!(messages.system.contains("tenant_url"));
        assert!(messages.system.contains("phone_column"));
        assert!(messages.system.contains("auth_type"));
        assert!(messages.system.contains("oauth2"));
        assert!(messages.system.contains("entity_column"));
        assert!(messages.system.contains("output_format"));

        // User message includes pipeline context
        assert!(messages.user.contains("Workday"));
        assert!(messages.user.contains("name, email"));
        assert!(messages.user.contains("fullName"));
        assert!(messages.user.contains("MAPPED: name → fullName"));
    }

    #[test]
    fn generate_prompt_handles_empty_context() {
        let prompt = PlanPrompt {
            source_system: "CSV",
            final_columns: &[],
            schema_diff: "",
            egress_schema: &HashMap::new(),
        };

        let messages = prompt.generate_prompt();
        assert!(messages.user.contains("not yet determined"));
        assert!(messages.user.contains("not yet configured"));
        assert!(messages.user.contains("no diff available"));
    }

    #[test]
    fn snapshot_system_prompt() {
        let egress: HashMap<String, String> = [
            ("name".into(), "fullName".into()),
            ("email".into(), "workEmail".into()),
        ]
        .into_iter()
        .collect();

        let prompt = PlanPrompt {
            source_system: "Workday",
            final_columns: &["name".into(), "email".into(), "department".into()],
            schema_diff: "  MAPPED: name → fullName\n  MAPPED: email → workEmail\n  UNMAPPED SOURCE: department",
            egress_schema: &egress,
        };

        let messages = prompt.generate_prompt();
        insta::assert_snapshot!("system_prompt", messages.system);
    }

    #[test]
    fn snapshot_user_prompt() {
        let egress: HashMap<String, String> = [
            ("name".into(), "fullName".into()),
            ("email".into(), "workEmail".into()),
        ]
        .into_iter()
        .collect();

        let prompt = PlanPrompt {
            source_system: "Workday",
            final_columns: &["name".into(), "email".into(), "department".into()],
            schema_diff: "  MAPPED: name → fullName\n  MAPPED: email → workEmail\n  UNMAPPED SOURCE: department",
            egress_schema: &egress,
        };

        let messages = prompt.generate_prompt();
        insta::assert_snapshot!("user_prompt", messages.user);
    }

    #[test]
    fn snapshot_empty_context_prompt() {
        let prompt = PlanPrompt {
            source_system: "CSV",
            final_columns: &[],
            schema_diff: "",
            egress_schema: &HashMap::new(),
        };

        let messages = prompt.generate_prompt();
        insta::assert_snapshot!("user_prompt_empty_context", messages.user);
    }
}
