# OnboardYou — Development Standards

> **Audience:** AI coding agents and human developers contributing to this codebase.
> Read this document **in full** before writing or modifying any code.

---

## 1. Project Overview

OnboardYou is a **zero-persistence, GDPR/HIPAA-compliant employee onboarding ETL pipeline** written in Rust. Data flows through an in-memory Polars LazyFrame and is never persisted to disk. The architecture follows the **Mediator Pattern**: a central orchestrator assembles and runs a sequence of pluggable actions defined by a declarative JSON manifest.

### 1.1 Workspace Layout

This is a **Cargo workspace** with three crates:

| Crate | Path | Role |
|---|---|---|
| `onboard_you` | `ETL/` | Core ETL library — domain types, capabilities (ingestion/logic/egress), orchestration |
| `config-api` | `lambdas/api/` | AWS Lambda — Axum-based REST API for CRUD on pipeline configs |
| `etl-trigger` | `lambdas/etl-trigger/` | AWS Lambda — EventBridge-triggered pipeline executor |

Infrastructure lives in `infra/` (OpenTofu/Terraform).

```
OnboardYou/
├── Cargo.toml              # Workspace root — declares members + shared deps
├── Makefile                 # Build & deploy targets (cargo-lambda, tofu)
├── brief.md                # Architecture brief (read-only reference)
├── standards.md            # THIS FILE
├── ETL/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs           # Library root — re-exports public API
│   │   ├── main.rs          # Lambda entrypoint (stub)
│   │   ├── domain/          # THE CONTRACT — traits, types, errors
│   │   │   ├── mod.rs
│   │   │   ├── traits/      # OnboardingAction trait
│   │   │   └── engine/      # RosterContext, Manifest, Errors, FieldMetadata
│   │   ├── capabilities/    # THE MUSCLE — pluggable action implementations
│   │   │   ├── mod.rs
│   │   │   ├── ingestion/   # Data acquisition (CSV, Workday SOAP)
│   │   │   │   ├── traits/  # HrisConnector trait
│   │   │   │   └── engine/  # CsvHrisConnector, WorkdayHrisConnector
│   │   │   ├── logic/       # Data transformation (SCD2, masking, dedup, fuzzy, rename, drop)
│   │   │   │   ├── traits/  # LogicAction, MaskingAction traits
│   │   │   │   └── engine/  # Concrete implementations
│   │   │   └── egress/      # Data delivery (stubs)
│   │   └── orchestration/   # THE MEDIATOR — factory + pipeline runner
│   │       ├── factory.rs   # ActionFactory — maps action_type strings → trait objects
│   │       ├── pipeline_runner.rs
│   │       └── clients/     # Shared HTTP/SOAP client abstractions
│   └── tests/               # Integration & E2E tests
│       ├── common/          # Shared mock data & helpers
│       ├── test_e2e_pipeline.rs
│       ├── test_identity_logic.rs
│       └── test_workday_connector.rs
├── lambdas/
│   ├── api/                 # Config API Lambda
│   │   └── src/
│   │       ├── main.rs      # Axum router + lambda_http adapter
│   │       ├── engine/      # Business logic (ConfigEngine)
│   │       ├── models/      # PipelineConfig, ApiError, AppState
│   │       └── repositories/ # DynamoDB + EventBridge Scheduler
│   └── etl-trigger/         # ETL Trigger Lambda
│       └── src/
│           ├── main.rs      # lambda_runtime handler
│           ├── engine/      # PipelineEngine
│           ├── models/      # ScheduleEvent, PipelineResult
│           └── repositories/ # DynamoDB read-only
└── infra/                   # OpenTofu — DynamoDB, Lambda, API Gateway, IAM
```

### 1.2 Architectural Layers (ETL Crate)

| Layer | Path | Responsibility | Analogy |
|---|---|---|---|
| **Domain** | `domain/` | Core traits, types, error definitions | The Contract |
| **Capabilities** | `capabilities/` | Concrete action implementations | The Muscle |
| **Orchestration** | `orchestration/` | Factory resolution + pipeline execution | The Brain / Mediator |

**Dependency rule:** `capabilities` → `domain` ← `orchestration`. Capabilities never import from orchestration. Orchestration imports from both.

### 1.3 Core Execution Flow

```
1. Lambda receives event (main.rs)
2. Manifest is deserialized from event/DynamoDB config
3. ActionFactory resolves each manifest action_type → Arc<dyn OnboardingAction>
4. PipelineRunner.run() folds RosterContext through each action sequentially
5. Egress action collects the LazyFrame and dispatches results
```

---

## 2. Architecture Standards

### 2.1 The Mediator Pattern

- **All business logic lives in `capabilities/`**. Orchestration never contains domain logic.
- **Actions are decoupled from each other.** One action must never import or depend on another action. They communicate exclusively through the shared `RosterContext`.
- **The factory is the single wiring point.** Every new action must be registered in `orchestration/factory.rs` with a string key matching its `action_type`.

### 2.2 The Fold Pattern

Pipeline execution uses a **fold/chain** pattern. Each action:
1. Receives a `RosterContext` (owns it — moved in)
2. Transforms it (modifies the `LazyFrame` and/or `field_metadata`)
3. Returns a new `RosterContext` (moved out)

```rust
fn execute(&self, context: RosterContext) -> Result<RosterContext>;
```

**Never hold onto the input context.** Always return a transformed version.

### 2.3 Zero-Persistence Principle

- Data lives only as a **Polars LazyFrame** in memory.
- **No intermediate files, databases, or caches** during pipeline execution.
- The only persistence points are the config store (DynamoDB) and the final egress dispatch.
- This is a GDPR/HIPAA compliance requirement. Do not violate it.

### 2.4 Declarative Control

- Pipeline steps are defined in a **versioned JSON manifest** (`Manifest` struct), not hardcoded.
- The manifest's `actions` array determines the order and configuration of steps.
- Adding a new capability should never require modifying `pipeline_runner.rs`.

### 2.5 Field Provenance (Source-of-Truth Tracking)

- Every column in the `RosterContext.field_metadata` must track its **origin** via `FieldMetadata { source: String }`.
- Ingestion actions tag fields with their source (e.g., `"HRIS_CONNECTOR"`, `"WORKDAY_HRIS"`).
- Logic actions that add columns must tag them with their own action name (e.g., `"scd_type_2"`, `"identity_deduplicator"`).

---

## 3. Coding Standards

### 3.1 Language & Toolchain

- **Rust edition:** 2021
- **Lambda runtime:** `provided.al2023` (binary named `bootstrap`)
- **Build tool:** `cargo-lambda` for cross-compilation
- **IaC:** OpenTofu (Terraform-compatible) in `infra/`
- **Key crates:** `polars`, `serde`/`serde_json`/`serde_yaml`, `thiserror`, `tracing`, `reqwest` (rustls), `lambda_runtime`/`lambda_http`, `aws-sdk-*`, `axum`

### 3.2 Module Organisation

Every Rust module follows this structure:

```
capability_name/
├── mod.rs       # Declares sub-modules, `pub use` re-exports
├── traits/      # Trait definitions (if applicable)
│   └── mod.rs
└── engine/      # Concrete implementations
    └── mod.rs
```

**Rules:**
- Every `.rs` file **must** begin with `//!` module-level doc comments explaining its purpose and architectural role.
- Every `mod.rs` **must** re-export its public types with `pub use` so consumers can import from the parent module directly.
- Use the pattern `pub use submodule::*` in `mod.rs` to create flat re-export chains.

**Example from `domain/mod.rs`:**
```rust
//! Core domain types and business interfaces
pub mod engine;
pub mod traits;

pub use engine::{ActionConfig, Error, FieldMetadata, Manifest, Result, RosterContext};
pub use traits::OnboardingAction;
```

### 3.3 Naming Conventions

| Concept | Convention | Examples |
|---|---|---|
| Structs | `PascalCase`, descriptive nouns | `RosterContext`, `CsvHrisConnector`, `SCDType2` |
| Config structs | `{ActionName}Config` | `CsvConfig`, `WorkdayConfig`, `PIIMaskingConfig` |
| Action type strings | `snake_case` (used in manifest JSON & factory match) | `"csv_hris_connector"`, `"scd_type_2"`, `"pii_masking"` |
| Trait names | `PascalCase`, describes capability | `OnboardingAction`, `HrisConnector`, `LogicAction` |
| Module names | `snake_case` | `identity_deduplicator`, `scd_type_2`, `pipeline_runner` |
| Test functions | `test_{what_is_being_tested}` | `test_factory_creates_csv_connector`, `test_dedup_basic` |
| Test helpers | Descriptive function names | `create_mock_csv_file()`, `create_test_manifest()` |

### 3.4 Construction Pattern for New Actions

Every new capability action **must** follow this exact pattern:

```rust
//! Brief description of what this action does

use crate::domain::{OnboardingAction, RosterContext, Result};

/// Config parsed from the manifest's `config` JSON object.
#[derive(Debug, Clone)]
pub struct MyActionConfig {
    // fields with sensible defaults
}

/// The action implementation.
#[derive(Debug)]
pub struct MyAction {
    config: MyActionConfig,
}

impl MyAction {
    /// Construct from the manifest's raw JSON config.
    pub fn from_action_config(config: &serde_json::Value) -> Self {
        // Parse config with defaults via serde or manual extraction
        Self { config: MyActionConfig { /* ... */ } }
    }
}

impl OnboardingAction for MyAction {
    fn id(&self) -> &str {
        "my_action" // must match the factory key
    }

    fn execute(&self, context: RosterContext) -> Result<RosterContext> {
        // Transform context.data (LazyFrame) and context.field_metadata
        // Return modified context
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    #[test]
    fn test_my_action_basic() {
        // Use df![] macro for inline test DataFrames
        // Verify row count, column existence, specific values
        // Verify field_metadata provenance
    }
}
```

**Checklist for adding a new action:**
1. Create the implementation file in the appropriate `capabilities/{ingestion|logic|egress}/engine/` directory
2. Add `pub mod my_action;` to the engine's `mod.rs`
3. Add `pub use engine::MyAction;` to the capability's `mod.rs`
4. Register the action in `orchestration/factory.rs` with a match arm
5. Write unit tests in the same file (`#[cfg(test)]`)
6. Write an integration test in `tests/` if the action interacts with external systems
7. Update this standards document if the action introduces new patterns

### 3.5 Error Handling

- **Domain errors** use `thiserror` with the `Error` enum in `domain/engine/errors.rs`.
- The domain `Result<T>` type alias must be used throughout the ETL crate: `pub type Result<T> = std::result::Result<T, Error>;`
- **Error variants** are:
  - `ConfigurationError(String)` — invalid manifest, missing config fields
  - `IngestionError(String)` — data acquisition failures
  - `TransformationError(String)` — logic step failures
  - `EgressError(String)` — dispatch failures
  - `PipelineError(String)` — orchestration-level failures
- **Lambda errors** use their own error types implementing `IntoResponse` (for Axum) or converting to JSON (for lambda_runtime).
- Always provide **descriptive context** in error messages using `format!()`:

```rust
// GOOD
.map_err(|e| Error::IngestionError(format!("Failed to parse CSV at '{}': {}", path, e)))?;

// BAD
.map_err(|e| Error::IngestionError(e.to_string()))?;
```

### 3.6 Polars Usage

- **Prefer lazy evaluation.** Stay on `LazyFrame` as long as possible. Only call `.collect()` when you absolutely need random-access row iteration (e.g., dedup, fuzzy matching).
- **Schema introspection** (checking column existence/types) should use `.collect_schema()` — this is cheap and does not trigger a full collect.
- **Column creation** should use `.with_column(lit(...).alias("name"))` on the LazyFrame.
- **Window functions** (`.over()`, `.shift()`) are preferred over row-by-row iteration for SCD logic.
- **Sorting** should use `.sort()` with `SortMultipleOptions` for deterministic output.
- When you must collect, use `context.data.clone().collect()` to avoid consuming the LazyFrame, then rebuild a new LazyFrame from the resulting `DataFrame` with `.lazy()`.
- **Test DataFrames** should be created with the `df![]` macro:

```rust
let df = df! {
    "employee_id" => &["E001", "E002"],
    "name"        => &["Alice", "Bob"],
}
.unwrap()
.lazy();
```

### 3.7 Serde Conventions

- **Config structs** derive `Deserialize` (and `Serialize` if they need to be written back).
- **Lambda API models** use `#[serde(rename_all = "camelCase")]` for JSON APIs.
- **Internal manifest types** use `snake_case` (Rust default serde mapping).
- Parsing from raw `serde_json::Value` is acceptable for action configs when you need custom defaults that `#[serde(default)]` doesn't easily express.

### 3.8 Logging & Tracing

- Use `tracing` crate macros (`tracing::info!`, `tracing::warn!`, `tracing::error!`), **never** `println!` or `eprintln!` (except in stubs/TODOs).
- Include structured fields: `tracing::info!(action_id = action.id(), "executing action")`.
- Lambda entrypoints initialise the subscriber with `tracing_subscriber` using JSON format + env-filter.

### 3.9 AWS SDK Patterns

- Load config with `aws_config::load_defaults(BehaviorVersion::latest())`.
- Use `serde_dynamo` for DynamoDB (de)serialisation — do not manually construct `AttributeValue` maps.
- Build `AppState` from environment variables (`std::env::var`), fail fast with `.expect()` at startup.
- Scheduler resource names follow the pattern: `onboardyou-{organizationId}`.

### 3.10 Dependency Injection

- External HTTP/SOAP calls must go through **trait abstractions** for testability.
- Example: the `SoapClient` trait in `orchestration/clients/` with a production `ReqwestSoapClient` and a `MockSoapClient` for tests.
- Actions that call external services must accept the client as a constructor parameter or use the trait-based approach established in `WorkdayHrisConnector`.

### 3.11 Code Quality Rules

- `cargo build` must succeed with **zero warnings** before any commit.
- Run `cargo clippy` and address all lints.
- Run `cargo fmt` — all code must be formatted with `rustfmt` defaults.
- No `unwrap()` in production code paths. Use `?` operator or `.expect("descriptive reason")` only at initialisation boundaries.
- `unsafe` blocks are acceptable only when documented with a `// Safety:` comment explaining the invariant.

---

## 4. Testing Standards

### 4.1 Test Organisation

| Test Type | Location | Purpose |
|---|---|---|
| **Unit tests** | `#[cfg(test)] mod tests` at bottom of each `.rs` file | Test a single struct/function in isolation |
| **Integration tests** | `ETL/tests/test_*.rs` | Test multi-action pipelines, factory wiring, E2E flows |
| **Shared test helpers** | `ETL/tests/common/` | Mock data, temp file helpers, manifest builders |

### 4.2 Unit Test Requirements

Every capability action file **must** include a `#[cfg(test)]` module with tests covering:

1. **Happy path** — action produces correct output for valid input
2. **Edge cases** — empty DataFrames, missing columns, null values
3. **Config parsing** — `from_action_config()` with valid and invalid JSON
4. **Field metadata** — verify provenance is tagged correctly after execution
5. **Error paths** — invalid input returns the correct `Error` variant

**Minimum bar:** No action is considered "complete" without at least 5 unit tests.

### 4.3 Test Data Patterns

```rust
// Inline DataFrame (preferred for unit tests)
let df = df! {
    "employee_id" => &["E001", "E002", "E003"],
    "name"        => &["Alice", "Bob", "Charlie"],
}
.unwrap()
.lazy();
let ctx = RosterContext::new(df);

// Shared CSV mock data (for integration tests)
use crate::common::mock_data::{create_mock_csv_file, create_test_manifest};

// Temp files (caller must keep handle alive for the file to exist)
let (temp_file, path) = create_mock_csv_file();
// ... use path ...
// temp_file is dropped here, cleaning up
```

### 4.4 Mock Pattern for External Dependencies

When testing actions that call external services, define a mock inside the test module:

```rust
#[cfg(test)]
mod tests {
    struct MockSoapClient {
        response: String,
    }
    impl SoapClient for MockSoapClient {
        fn send_request(&self, _url: &str, _envelope: &str) -> Result<String> {
            Ok(self.response.clone())
        }
    }
}
```

### 4.5 Integration Test Pattern

Integration tests in `ETL/tests/` should:
1. Use `common/mock_data.rs` helpers to build test data and manifests
2. Resolve actions through `ActionFactory::create()` — **not** by constructing them directly — to verify factory wiring
3. Run the full pipeline via `PipelineRunner::run()`
4. Assert on the final `RosterContext`: row counts, column existence, cell values, metadata

```rust
#[test]
fn test_e2e_csv_to_scd() {
    let (temp, path) = create_mock_csv_file();
    let manifest = create_test_manifest(&path);
    let actions = manifest.actions.iter()
        .map(|a| ActionFactory::create(a).unwrap())
        .collect::<Vec<_>>();
    let ctx = RosterContext::new(LazyFrame::default());
    let result = PipelineRunner::run(&manifest, actions, ctx).unwrap();
    let collected = result.data.collect().unwrap();
    assert!(collected.width() > 0);
}
```

### 4.6 Running Tests

```bash
# All tests (unit + integration)
cargo test --workspace

# ETL crate only
cargo test -p onboard_you

# Specific test
cargo test -p onboard_you test_factory_creates_csv_connector

# With output (for debugging)
cargo test -p onboard_you -- --nocapture
```

---

## 5. Lambda Standards

### 5.1 Config API (`lambdas/api/`)

- Uses **Axum** with `lambda_http` adapter.
- Routes are: `GET /config/{org_id}`, `POST /config`, `PUT /config/{org_id}`.
- Request/response models use `#[serde(rename_all = "camelCase")]`.
- Business logic lives in `engine/config_engine.rs`, **not** in route handlers.
- Repository layer wraps AWS SDK calls (`config_repository.rs`, `schedule_repository.rs`).
- Error responses use `ApiError` enum → HTTP status codes via `IntoResponse`.
- `AppState` is constructed once at startup from env vars and shared via Axum state.

### 5.2 ETL Trigger (`lambdas/etl-trigger/`)

- Uses `lambda_runtime` directly (not Axum).
- Receives `ScheduleEvent` from EventBridge Scheduler.
- Fetches pipeline config from DynamoDB → deserialises manifest → runs pipeline via `ActionFactory` + `PipelineRunner`.
- Returns `PipelineResult` with success/failure constructors.

### 5.3 Shared Lambda Conventions

- Both lambdas produce a binary named `bootstrap` (required by `provided.al2023`).
- Both use workspace-level dependency versions from the root `Cargo.toml`.
- Build with: `cargo lambda build --release -p <crate-name> --output-format zip`.
- Environment variables are the **only** configuration mechanism for Lambdas (table names, regions, etc.).

---

## 6. Infrastructure Standards

### 6.1 OpenTofu / Terraform

- All IaC lives in `infra/`.
- Provider: AWS `~> 5.0`, OpenTofu `>= 1.6`.
- State management is configured in `main.tf`.
- Variables are defined in `variables.tf`, values in `terraform.tfvars.example` (never commit real `.tfvars`).
- Resources use the prefix `onboardyou-` for naming.
- Default tags are applied to all resources.

### 6.2 Resource Conventions

| Resource | Settings |
|---|---|
| DynamoDB | PAY_PER_REQUEST, PITR enabled, PK = `organizationId` |
| Lambdas | `provided.al2023`, arm64, memory/timeout per function |
| API Gateway | REST API, `v1` stage, CORS enabled, X-Ray tracing |
| IAM | Least-privilege roles per Lambda, separate scheduler execution role |

---

## 7. Build & Deploy

### 7.1 Makefile Targets

| Target | Action |
|---|---|
| `make build-lambdas` | Cross-compile both Lambda binaries |
| `make build-config-api` | Build config API Lambda only |
| `make build-etl-trigger` | Build ETL trigger Lambda only |
| `make tf-init` | Initialise OpenTofu |
| `make tf-plan` | Build Lambdas + plan infra changes |
| `make tf-apply` | Apply planned infra changes |
| `make deploy` | Full build + plan + apply |
| `make clean` | `cargo clean` + remove tofu artefacts |

### 7.2 Pre-Commit Checklist

Before opening a PR or committing:

1. `cargo fmt --all` — format all code
2. `cargo clippy --workspace` — zero warnings
3. `cargo build --workspace` — compiles cleanly
4. `cargo test --workspace` — all tests pass
5. New actions have ≥5 unit tests + factory registration + integration test

---

## 8. Current TODOs & Known Gaps

These items are pending implementation. When picking up work, check this list:

| Item | Status | Notes |
|---|---|---|
| `ETL/src/main.rs` Lambda entrypoint | Stub | Needs event deserialisation + mediator wiring |
| `ApiDispatcher` (egress) | Stub | Returns context unchanged; needs HTTP dispatch |
| `Observability` (egress) | Stub | Returns context unchanged; needs structured logging/RCA |
| `DataValidator` / `validator.rs` | Not yet created | Mentioned in brief — in-stream regex/type validation |
| `RenameColumn` + `DropColumn` factory registration | Missing | Engine implementations exist but not wired in `factory.rs` |
| API Gateway auth | `NONE` | Needs Cognito or API key integration |
| CI/CD pipeline | Not yet created | GitHub Actions for test + build + deploy |

---

## 9. Quick Reference for Agents

### "I need to add a new logic action"

1. Create `ETL/src/capabilities/logic/engine/my_action.rs`
2. Add `pub mod my_action;` to `ETL/src/capabilities/logic/engine/mod.rs`
3. Ensure `pub use engine::MyAction;` is re-exported up through `logic/mod.rs` → `capabilities/mod.rs`
4. Implement the Construction Pattern from §3.4
5. Register `"my_action"` in `orchestration/factory.rs` match arm
6. Write ≥5 unit tests in the same file
7. Add an integration test in `ETL/tests/`
8. Run `cargo test --workspace` — all green

### "I need to add a new ingestion connector"

Same as above but in `capabilities/ingestion/engine/`. Also implement the `HrisConnector` trait in addition to `OnboardingAction`.

### "I need to modify the pipeline config API"

1. Models go in `lambdas/api/src/models/`
2. Business logic in `lambdas/api/src/engine/`
3. DynamoDB/Scheduler calls in `lambdas/api/src/repositories/`
4. Route handlers in `lambdas/api/src/main.rs`
5. Use `#[serde(rename_all = "camelCase")]` on all API models

### "I need to add infrastructure"

1. Add to appropriate `.tf` file in `infra/`
2. Add variables to `variables.tf` + `terraform.tfvars.example`
3. Add outputs to `outputs.tf`
4. Run `make tf-plan` to verify
