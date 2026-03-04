//! Plan prompt — generates the AI prompt for ETL pipeline plan generation.
//!
//! Uses `utoipa::ToSchema` to derive config schemas directly from the Rust
//! struct definitions. If a field is added, renamed, or removed, the prompt
//! automatically reflects the change — no manual sync needed.

use std::collections::HashMap;
use utoipa::ToSchema;

/// Render an OpenAPI schema from a `ToSchema` type as a pretty-printed JSON string.
fn schema_json<T: ToSchema>() -> String {
    let schema = T::schema();
    serde_json::to_string_pretty(&schema).unwrap_or_else(|_| "{}".into())
}

/// One action definition: its `action_type` key, a human description,
/// and the JSON schema derived from the Rust config struct.
struct ActionDef {
    action_type: &'static str,
    description: &'static str,
    config_schema: String,
}

/// Build the action definition list, grouped by category.
fn action_defs() -> Vec<(&'static str, Vec<ActionDef>)> {
    vec![
        ("INGRESS ACTIONS (pick ONE as the first action)", vec![
            ActionDef {
                action_type: "csv_hris_connector",
                description: "Ingests employee data from a CSV file.",
                config_schema: schema_json::<crate::CsvHrisConnectorConfig>(),
            },
            ActionDef {
                action_type: "workday_hris_connector",
                description: "Ingests employee data from the Workday SOAP API.",
                config_schema: schema_json::<crate::WorkdayConfig>(),
            },
        ]),
        ("TRANSFORMATION ACTIONS (zero or more, in any order)", vec![
            ActionDef {
                action_type: "scd_type_2",
                description: "Slowly Changing Dimension Type 2 — tracks historical changes to entities.",
                config_schema: schema_json::<crate::ScdType2Config>(),
            },
            ActionDef {
                action_type: "pii_masking",
                description: "Masks PII fields (e.g. SSN, salary) using configurable strategies.",
                config_schema: schema_json::<crate::PIIMaskingConfig>(),
            },
            ActionDef {
                action_type: "identity_deduplicator",
                description: "Deduplicates records based on matching columns.",
                config_schema: schema_json::<crate::DedupConfig>(),
            },
            ActionDef {
                action_type: "regex_replace",
                description: "Applies regex find-and-replace on a column.",
                config_schema: schema_json::<crate::RegexReplaceConfig>(),
            },
            ActionDef {
                action_type: "iso_country_sanitizer",
                description: "Normalizes country codes to ISO 3166 format.",
                config_schema: schema_json::<crate::IsoCountrySanitizerConfig>(),
            },
            ActionDef {
                action_type: "cellphone_sanitizer",
                description: "Normalizes phone numbers to E.164 format.",
                config_schema: schema_json::<crate::CellphoneSanitizerConfig>(),
            },
            ActionDef {
                action_type: "handle_diacritics",
                description: "Normalizes diacritical characters (e.g. é → e, ñ → n).",
                config_schema: schema_json::<crate::HandleDiacriticsConfig>(),
            },
            ActionDef {
                action_type: "rename_column",
                description: "Renames columns via a source → target mapping.",
                config_schema: schema_json::<crate::RenameConfig>(),
            },
            ActionDef {
                action_type: "drop_column",
                description: "Drops specified columns from the dataset.",
                config_schema: schema_json::<crate::DropConfig>(),
            },
            ActionDef {
                action_type: "filter_by_value",
                description: "Filters rows by regex. Matching rows are kept.",
                config_schema: schema_json::<crate::FilterByValueConfig>(),
            },
        ]),
        ("EGRESS ACTION (must be the LAST action)", vec![
            ActionDef {
                action_type: "api_dispatcher",
                description: "Sends transformed data to the destination API. Auth variants: Bearer, OAuth 1.0, OAuth 2.0, or Default (org-level settings). Config is one of:",
                config_schema: format!(
                    "Bearer: {bearer}\nOAuth 1.0: {oauth}\nOAuth 2.0: {oauth2}",
                    bearer = schema_json::<crate::BearerRepoConfig>(),
                    oauth = schema_json::<crate::OAuthRepoConfig>(),
                    oauth2 = schema_json::<crate::OAuth2RepoConfig>(),
                ),
            },
        ]),
    ]
}

/// Schemas for sub-types referenced via `$ref` in the action configs above.
///
/// Without these the LLM would see an opaque `$ref` string and not know the
/// shape of the type.  Each entry maps the OpenAPI component name to its
/// auto-derived JSON schema.
fn referenced_type_schemas() -> Vec<(&'static str, String)> {
    vec![
        ("WorkdayResponseGroup", schema_json::<crate::WorkdayResponseGroup>()),
        ("ColumnMask", schema_json::<crate::ColumnMask>()),
        ("MaskStrategy", schema_json::<crate::MaskStrategy>()),
        ("CountryOutputFormat", schema_json::<crate::CountryOutputFormat>()),
        ("BearerPlacement", schema_json::<crate::BearerPlacement>()),
        ("OAuth2GrantType", schema_json::<crate::OAuth2GrantType>()),
    ]
}

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
8. Generate a synthetic preview where "before" keys are ONLY column names from PIPELINE COLUMNS and "after" keys are ONLY the field names (the keys) from the EGRESS SCHEMA. The EGRESS SCHEMA format is {{"field_name": "type"}} — use the KEYS as destination field names. Use realistic sample values for each field. Do NOT invent field names that are not in PIPELINE COLUMNS or EGRESS SCHEMA.
9. Return ONLY valid JSON matching the response schema below. No markdown, no explanation.

RESPONSE SCHEMA:
{{
  "manifest": {{
    "version": "1.0",
    "actions": [
      {{ "id": "string", "action_type": "<action_type>", "disabled": false, "config": {{...}} }}
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
        "actionIds": ["string"]
      }}
    ],
    "preview": {{
      "sourceLabel": "string",
      "targetLabel": "string",
      "before": {{ "<PIPELINE_COLUMN>": "<sample_value>", ... }},
      "after": {{ "<EGRESS_SCHEMA_DESTINATION_FIELD>": "<sample_value>", ... }}
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
                let sorted: std::collections::BTreeMap<_, _> = self.egress_schema.iter().collect();
                serde_json::to_string_pretty(&sorted).unwrap_or_default()
            },
            schema_diff = if self.schema_diff.is_empty() {
                "(no diff available)"
            } else {
                self.schema_diff
            },
        )
    }

    /// Build the complete ETL action schema from `utoipa::ToSchema` definitions.
    ///
    /// Each config struct's OpenAPI schema is serialized to JSON automatically.
    /// Adding or changing a field on the struct updates the prompt with zero manual work.
    fn etl_schema() -> String {
        let mut out = String::from("AVAILABLE ACTIONS AND CONFIG SCHEMAS:\n");

        for (category, actions) in action_defs() {
            out.push_str(&format!("\n## {category}\n\n"));
            for def in actions {
                out.push_str(&format!(
                    "### action_type: {action_type}\n{description}\nConfig schema:\n{schema}\n\n",
                    action_type = def.action_type,
                    description = def.description,
                    schema = def.config_schema,
                ));
            }
        }

        out.push_str("\n## REFERENCED TYPES\n\n");
        out.push_str("The schemas above reference these component types via `$ref`:\n\n");
        for (name, schema) in referenced_type_schemas() {
            out.push_str(&format!("### {name}\n{schema}\n\n", name = name, schema = schema));
        }

        out
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
        let egress: HashMap<String, String> =
            std::collections::BTreeMap::from([
                ("email".into(), "workEmail".into()),
                ("name".into(), "fullName".into()),
            ])
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
