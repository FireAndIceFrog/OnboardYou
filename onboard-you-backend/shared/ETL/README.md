# OnboardYou

Zero-persistence employee onboarding pipeline built in Rust with Polars.

Data is processed entirely in-memory as Polars LazyFrames — nothing is persisted to disk during execution, meeting GDPR/HIPAA pass-through requirements.

## Architecture

```
src/
├── domain/           # Core types & business interfaces (the Contract)
│   ├── traits/       #   OnboardingAction trait
│   └── engine/       #   RosterContext, Manifest, Errors
├── capabilities/     # Functional logic steps (the Muscle)
│   ├── ingestion/    #   Data acquisition (CSV, HRIS APIs)
│   ├── logic/        #   Transformation engines
│   └── egress/       #   Data delivery & observability
└── orchestration/    # Pipeline assembly & execution (the Mediator)
    ├── factory.rs    #   Maps manifest action_type → Rust struct
    └── pipeline_runner.rs
```

## Quick Start

```bash
# Run all tests
cargo test

# Generate HTML documentation from source comments
cargo doc --open
```

## Pipeline Manifest

The pipeline is driven by a declarative JSON manifest. Each action declares its
`action_type` (resolved by the factory) and a `config` object whose shape
depends on the action.

```json
{
  "version": "1.0",
  "actions": [
    {
      "id": "ingest_csv",
      "action_type": "generic_ingestion_connector",
      "config": { "csv_path": "/data/employees.csv" }
    },
    {
      "id": "history",
      "action_type": "scd_type_2",
      "config": { "entity_column": "employee_id", "date_column": "start_date" }
    },
    {
      "id": "dedup",
      "action_type": "identity_deduplicator",
      "config": { "columns": ["national_id", "email"] }
    },
    {
      "id": "fuzzy",
      "action_type": "identity_fuzzy_match",
      "config": { "columns": ["first_name", "last_name"], "threshold": 0.85 }
    },
    {
      "id": "mask_pii",
      "action_type": "pii_masking",
      "config": {
        "columns": [
          { "name": "ssn", "strategy": "redact", "keep_last": 4, "mask_prefix": "***-**-" },
          { "name": "salary", "strategy": "zero" }
        ]
      }
    }
  ]
}
```

---

## Action Config Reference

Every action shares the outer envelope:

| Field         | Type   | Description                                         |
|---------------|--------|-----------------------------------------------------|
| `id`          | string | Unique identifier for this pipeline step            |
| `action_type` | string | Factory key — selects the Rust implementation       |
| `config`      | object | Action-specific configuration (see sections below)  |

---

### `generic_ingestion_connector`

Reads a CSV file and populates the `RosterContext` LazyFrame. Every ingested
column is tagged with `HRIS_CONNECTOR` field-ownership metadata.

| Field      | Type   | Required | Default | Description                  |
|------------|--------|----------|---------|------------------------------|
| `csv_path` | string | **yes**  | —       | Absolute path to the CSV file |

```json
{
  "action_type": "generic_ingestion_connector",
  "config": {
    "csv_path": "/data/employees.csv"
  }
}
```

---

### `scd_type_2`

Slowly Changing Dimension Type 2 — adds `effective_from`, `effective_to`, and
`is_current` columns using Polars window functions.

| Field           | Type   | Required | Default         | Description                                       |
|-----------------|--------|----------|-----------------|---------------------------------------------------|
| `entity_column` | string | no       | `"employee_id"` | Column that identifies the entity (partition key)  |
| `date_column`   | string | no       | `"start_date"`  | Column holding the date used for versioning        |

**Behaviour:**

1. Sorts by `(entity_column, date_column)` ascending.
2. Renames `date_column` → `effective_from`.
3. Computes `effective_to` = next row's `effective_from` within each entity partition.
4. Sets `is_current = true` where `effective_to` is null (latest record).

```json
{
  "action_type": "scd_type_2",
  "config": {
    "entity_column": "employee_id",
    "date_column": "start_date"
  }
}
```

---

### `identity_deduplicator`

Deterministic deduplication. Iterates configured columns in priority order —
the first non-null value becomes the dedup key. Rows sharing a key are flagged
as duplicates, with the first occurrence designated as canonical.

| Field                | Type     | Required | Default                        | Description                                             |
|----------------------|----------|----------|--------------------------------|---------------------------------------------------------|
| `columns`            | string[] | no       | `["national_id", "email"]`     | Columns to inspect in priority order for the dedup key   |
| `employee_id_column` | string   | no       | `"employee_id"`                | Column used as the canonical ID value                    |

**Output columns:**

| Column         | Type   | Description                                        |
|----------------|--------|----------------------------------------------------|
| `canonical_id` | string | The `employee_id_column` value of the first occurrence |
| `is_duplicate` | bool   | `true` for every row after the first in a group     |

```json
{
  "action_type": "identity_deduplicator",
  "config": {
    "columns": ["national_id", "email"],
    "employee_id_column": "employee_id"
  }
}
```

---

### `identity_fuzzy_match`

Probabilistic matching using Levenshtein similarity. Configured columns are
concatenated into a composite string per row, then every pair is compared.
Matches above the threshold are grouped via Union-Find.

| Field                | Type     | Required | Default                          | Description                                        |
|----------------------|----------|----------|----------------------------------|----------------------------------------------------|
| `columns`            | string[] | no       | `["first_name", "last_name"]`    | Columns concatenated for comparison                 |
| `threshold`          | float    | no       | `0.80`                           | Minimum similarity (0.0–1.0) to consider a match    |
| `employee_id_column` | string   | no       | `"employee_id"`                  | Column used to label match groups                   |

**Output columns:**

| Column             | Type   | Description                                          |
|--------------------|--------|------------------------------------------------------|
| `match_group_id`   | string | Group label (`grp_<employee_id>` of the group root)  |
| `match_confidence` | float  | Highest pairwise similarity within the group          |

```json
{
  "action_type": "identity_fuzzy_match",
  "config": {
    "columns": ["first_name", "last_name"],
    "threshold": 0.85,
    "employee_id_column": "employee_id"
  }
}
```

---

### `pii_masking`

Applies per-column masking rules. Supports two strategies:

| Strategy | Effect                                                        |
|----------|---------------------------------------------------------------|
| `redact` | Keeps the last N characters, replaces prefix with a mask string |
| `zero`   | Replaces all numeric values with `0`                           |

#### New format (recommended)

| Field               | Type     | Required | Default   | Description                                  |
|---------------------|----------|----------|-----------|----------------------------------------------|
| `columns`           | object[] | **yes**  | —         | Array of column masking rules                 |
| `columns[].name`    | string   | **yes**  | —         | Column name to mask                           |
| `columns[].strategy`| string   | no       | `"redact"`| `"redact"` or `"zero"`                        |
| `columns[].keep_last`| int     | no       | `4`       | Characters to preserve (redact only)          |
| `columns[].mask_prefix`| string| no       | `"***-**-"`| Prefix replacing the redacted portion        |

```json
{
  "action_type": "pii_masking",
  "config": {
    "columns": [
      { "name": "ssn", "strategy": "redact", "keep_last": 4, "mask_prefix": "***-**-" },
      { "name": "salary", "strategy": "zero" },
      { "name": "phone", "strategy": "redact", "keep_last": 4, "mask_prefix": "***-***-" }
    ]
  }
}
```

#### Legacy format (still supported)

| Field         | Type | Required | Default | Description              |
|---------------|------|----------|---------|--------------------------|
| `mask_ssn`    | bool | no       | `true`  | Mask the `ssn` column    |
| `mask_salary` | bool | no       | `true`  | Zero the `salary` column |

```json
{
  "action_type": "pii_masking",
  "config": {
    "mask_ssn": true,
    "mask_salary": true
  }
}
```

---

## Generating Documentation

Rust doc comments (`///` and `//!`) are compiled into HTML by `cargo doc`.
Every config struct, field, and engine in this codebase is documented with
these comments, including JSON examples.

```bash
# Build and open in browser
cargo doc --open --no-deps

# Build only (CI)
cargo doc --no-deps
```

The generated docs live in `target/doc/onboard_you/index.html`.

---

## Running Tests

```bash
cargo test              # Full suite (unit + integration + E2E)
cargo test --lib        # Library unit tests only
cargo test -- --nocapture  # Show stdout/tracing output
```
