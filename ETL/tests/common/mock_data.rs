//! Mock CSV and JSON data for testing
//!
//! Provides both static sample data for unit tests and a programmatic
//! generator (`generate_hris_csv`) for creating arbitrarily large CSVs
//! for load-testing and benchmarking.

use std::io::Write;
use std::path::PathBuf;

/// Sample HRIS data for testing (7 columns, 3 rows)
pub const SAMPLE_HRIS_CSV: &str = "\
employee_id,first_name,last_name,email,ssn,salary,start_date
001,John,Doe,john.doe@example.com,123-45-6789,75000,2024-01-01
002,Jane,Smith,jane.smith@example.com,987-65-4321,85000,2024-02-15
003,Alice,Johnson,alice.j@example.com,555-12-3456,92000,2024-03-10
";

/// Write `SAMPLE_HRIS_CSV` to a temp file and return the handle + path.
///
/// The caller **must** keep the returned `NamedTempFile` alive for the
/// duration of the test so the OS doesn't delete the file.
pub fn write_sample_csv() -> (tempfile::NamedTempFile, PathBuf) {
    let mut tmp = tempfile::NamedTempFile::new().expect("create temp file");
    tmp.write_all(SAMPLE_HRIS_CSV.as_bytes())
        .expect("write csv");
    let path = tmp.path().to_path_buf();
    (tmp, path)
}

// ---------------------------------------------------------------------------
// Programmatic CSV generator (for load-testing / benchmarking)
// ---------------------------------------------------------------------------

/// First names — includes diacritics so `handle_diacritics` has real work to do.
const FIRST_NAMES: &[&str] = &[
    "John",
    "Jane",
    "Alice",
    "José",
    "François",
    "Müller",
    "Siobhán",
    "Björk",
    "André",
    "Zoë",
    "Chloë",
    "Renée",
    "László",
    "Søren",
    "Jürgen",
    "Conceição",
    "Yağız",
    "Håkon",
    "Stéphane",
    "Noël",
    "Daniël",
    "Léa",
    "Maëlle",
    "Raphaël",
    "Anaïs",
    "Ángel",
    "Héctor",
    "Inés",
    "Tomáš",
    "Lukáš",
    "Jiří",
    "Věra",
    "Gábor",
    "Ádám",
];

const LAST_NAMES: &[&str] = &[
    "Doe",
    "Smith",
    "Johnson",
    "García",
    "Müller",
    "Ó'Brien",
    "Nakamura",
    "Johansson",
    "Lefèvre",
    "Hernández",
    "Bošković",
    "Čermák",
    "Dvořák",
    "Šimůnek",
    "Kováč",
    "Šťastný",
    "Łuczak",
    "Wiśniewski",
    "Górski",
    "Petrović",
    "Đorđević",
    "Nikolić",
    "Jørgensen",
    "Ødegård",
    "Grünwald",
    "Böhm",
    "Strauß",
    "Weiß",
    "Kühn",
    "Schäfer",
    "Núñez",
    "López",
];

const COUNTRIES: &[&str] = &[
    "US", "GB", "DE", "FR", "JP", "AU", "CA", "NL", "SE", "NO", "ES", "IT", "BR", "MX", "IN", "ZA",
    "NZ", "IE", "AT", "CH",
];

/// Generates a CSV string with `n` employee rows.
///
/// The CSV has these columns (superset of the original 7), chosen so that
/// every registered action has data to operate on:
///
/// | Column         | Purpose                              |
/// |----------------|--------------------------------------|
/// | employee_id    | Primary key                          |
/// | first_name     | Contains diacritics                  |
/// | last_name      | Contains diacritics                  |
/// | email          | Identity dedup key                   |
/// | national_id    | Identity dedup key (NID-prefixed)    |
/// | ssn            | PII masking + regex_replace target   |
/// | salary         | PII masking                          |
/// | start_date     | SCD Type 2                           |
/// | country_raw    | ISO country sanitizer input          |
/// | mobile_phone   | Cellphone sanitizer input            |
///
/// Deterministic: the same `n` always produces the same output, making
/// benchmark comparisons reproducible.
pub fn generate_hris_csv(n: usize) -> String {
    let mut buf = String::with_capacity(n * 120);
    buf.push_str(
        "employee_id,first_name,last_name,email,national_id,ssn,salary,start_date,country_raw,mobile_phone\n",
    );

    for i in 0..n {
        let eid = format!("E{:06}", i + 1);
        let first = FIRST_NAMES[i % FIRST_NAMES.len()];
        let last = LAST_NAMES[i % LAST_NAMES.len()];
        let email = format!(
            "{}.{}{}@example.com",
            first.to_ascii_lowercase(),
            last.to_ascii_lowercase(),
            i
        );
        let nid = format!("NID-{:08}", i + 1);
        // SSN: formatted with dashes so regex_replace can strip them
        let ssn = format!(
            "{:03}-{:02}-{:04}",
            (i % 900) + 100,
            (i % 90) + 10,
            (i % 9000) + 1000
        );
        let salary = 50_000 + (i % 100) * 1_000;
        // Spread start_dates across 2024
        let month = (i % 12) + 1;
        let day = (i % 28) + 1;
        let start_date = format!("2024-{:02}-{:02}", month, day);
        let country = COUNTRIES[i % COUNTRIES.len()];
        // Local phone numbers — space forces Polars to read as String, not i64
        let phone = format!("0 7700 {:06}", (i % 900_000) + 100_000);

        // Intentionally duplicate some emails to give identity_deduplicator work
        let email_final = if i > 0 && i % 50 == 0 {
            // Every 50th row reuses the previous row's email → duplicate
            format!(
                "{}.{}{}@example.com",
                FIRST_NAMES[(i - 1) % FIRST_NAMES.len()].to_ascii_lowercase(),
                LAST_NAMES[(i - 1) % LAST_NAMES.len()].to_ascii_lowercase(),
                i - 1
            )
        } else {
            email
        };

        buf.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            eid, first, last, email_final, nid, ssn, salary, start_date, country, phone,
        ));
    }
    buf
}

/// Write a generated CSV of `n` rows to a temp file and return handle + path.
pub fn write_generated_csv(n: usize) -> (tempfile::NamedTempFile, PathBuf) {
    let csv = generate_hris_csv(n);
    let mut tmp = tempfile::NamedTempFile::new().expect("create temp file");
    tmp.write_all(csv.as_bytes()).expect("write csv");
    let path = tmp.path().to_path_buf();
    (tmp, path)
}

// ---------------------------------------------------------------------------
// Manifest helpers
// ---------------------------------------------------------------------------

/// Manifest that uses the CSV connector with S3 URI and declared columns.
pub fn sample_csv_manifest(s3_uri: &str, columns: &[&str]) -> String {
    let cols_json: Vec<String> = columns.iter().map(|c| format!("\"{}\"", c)).collect();
    format!(
        r#"{{
  "version": "1.0",
  "actions": [
    {{
      "id": "ingest_hris",
      "action_type": "csv_hris_connector",
      "config": {{ "s3_uri": "{}", "columns": [{}] }}
    }}
  ]
}}"#,
        s3_uri,
        cols_json.join(", "),
    )
}

/// Manifest exercising **every** registered action in dependency order.
///
/// The pipeline chain:
///  1. csv_hris_connector   — ingest from file
///  2. handle_diacritics    — transliterate first_name, last_name
///  3. iso_country_sanitizer— country_raw → country_code (alpha2)
///  4. cellphone_sanitizer  — mobile_phone + country_code → mobile_phone_intl
///  5. regex_replace         — strip dashes from SSN
///  6. identity_deduplicator — dedup on email + employee_id
///  7. filter_by_value       — keep rows where employee_id matches ^E\d+
///  8. scd_type_2            — effective dating on start_date
///  9. pii_masking           — mask ssn, salary
/// 10. rename_column         — rename national_id → nid
/// 11. drop_column           — drop the is_duplicate helper column
pub fn full_pipeline_manifest(filename: &str, columns: &[&str]) -> String {
    let cols_json: Vec<String> = columns.iter().map(|c| format!("\"{}\"", c)).collect();
    let cols_str = cols_json.join(", ");
    format!(
        r#"{{
  "version": "1.0",
  "actions": [
    {{
      "id": "ingest",
      "action_type": "csv_hris_connector",
      "config": {{ "filename": "{filename}", "columns": [{cols_str}] }}
    }},
    {{
      "id": "diacritics",
      "action_type": "handle_diacritics",
      "config": {{
        "columns": ["first_name", "last_name"]
      }}
    }},
    {{
      "id": "country_sanitize",
      "action_type": "iso_country_sanitizer",
      "config": {{
        "source_column": "country_raw",
        "output_column": "country_code",
        "output_format": "alpha2"
      }}
    }},
    {{
      "id": "phone_sanitize",
      "action_type": "cellphone_sanitizer",
      "config": {{
        "phone_column": "mobile_phone",
        "country_columns": ["country_code"],
        "output_column": "mobile_phone_intl"
      }}
    }},
    {{
      "id": "ssn_strip_dashes",
      "action_type": "regex_replace",
      "config": {{
        "column": "ssn",
        "pattern": "-",
        "replacement": ""
      }}
    }},
    {{
      "id": "dedup",
      "action_type": "identity_deduplicator",
      "config": {{
        "columns": ["email"],
        "employee_id_column": "employee_id"
      }}
    }},
    {{
      "id": "filter_active",
      "action_type": "filter_by_value",
      "config": {{
        "column": "employee_id",
        "pattern": "^E\\d+"
      }}
    }},
    {{
      "id": "scd",
      "action_type": "scd_type_2",
      "config": {{
        "entity_column": "employee_id",
        "date_column": "start_date"
      }}
    }},
    {{
      "id": "mask_pii",
      "action_type": "pii_masking",
      "config": {{
        "columns": [
          {{ "name": "ssn", "strategy": "redact", "keep_last": 4, "mask_prefix": "***-**-" }},
          {{ "name": "salary", "strategy": "zero" }}
        ]
      }}
    }},
    {{
      "id": "rename",
      "action_type": "rename_column",
      "config": {{
        "mapping": {{ "national_id": "nid" }}
      }}
    }},
    {{
      "id": "drop_helpers",
      "action_type": "drop_column",
      "config": {{
        "columns": ["is_duplicate"]
      }}
    }}
  ]
}}"#,
    )
}

/// Original sample manifest (kept for backward compatibility)
pub const SAMPLE_MANIFEST_JSON: &str = r#"{
  "version": "1.0",
  "actions": [
    {
      "id": "hris_connector",
      "action_type": "csv_hris_connector",
      "config": {
        "filename": "data.csv",
        "columns": ["employee_id", "first_name", "last_name", "email", "ssn", "salary", "start_date"]
      }
    }
  ]
}"#;
