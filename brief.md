Core Philosophy
The Mediator Pattern: Centralized logic (the "Brain") lives in orchestration/, while specific work (the "Muscle") lives in capabilities/.

Zero-Persistence: Data is processed as a Polars LazyFrame in-memory. The pipeline is "Pass-Through," meeting GDPR/HIPAA recommendations from the case study.

Declarative Control: The order of operations is determined by a versioned JSON/YAML manifest, not hardcoded Rust logic.

2. Descriptive Directory Structure
Plaintext

src/
├── domain/                      # THE CONTRACT: Core types & business interfaces
│   ├── mod.rs                   # Action traits (OnboardingAction) & connector interfaces
│   ├── roster.rs                # The RosterContext: Wraps Polars LazyFrame + Source-of-Truth metadata
│   ├── manifest.rs              # Schema definitions for versioned, declarative pipeline configs
│   └── errors.rs                # Unified error handling using 'thiserror' for domain-specific failures
├── capabilities/                # THE ACTIONS: Functional logic steps (The Filters)
│   ├── ingestion/               # ENTRY: Data acquisition (Webhooks, API Polling, CSV parsing)
│   │   ├── hris_connector.rs    # Generic trait for external HRIS systems (Workday, BambooHR)
│   │   └── validator.rs         # In-stream data enforcement (Regex, Type checks, logical validation)
│   ├── logic/                   # TRANSFORMATION: Domain-specific data evolution
│   │   ├── scd_type_2.rs        # Effective dating logic (EffectiveFrom/To) for historical tracking
│   │   └── masking.rs           # PII protection (SSN/Salary masking) based on residency rules
│   │   ├── identity_deduplicator.rs      # Column-major identity resolution using NID/Email
│   │   └── identity_fuzzy_match.rs       # Probabilistic matching for high-fidelity record merging
│   └── egress/                  # EXIT: Data delivery (Bi-directional CRUD/Writebacks)
│       ├── api_dispatcher.rs    # HTTP/JSON delivery to client-facing destination APIs
│       └── observability.rs     # Real-time request/response logging & Root Cause Analysis (RCA)
├── orchestration/               # THE MEDIATOR: Pipeline assembly and execution
│   ├── factory.rs               # The Resolver: Maps manifest string IDs to Capability instances
│   └── pipeline_runner.rs       # The Loop: Sequentially executes Actions on the RosterContext
├── main.rs                      # LAMBDA ENTRY: Deserializes events & triggers the Mediator
└── lib.rs                       # Library exposure for integration testing
tests/                           # VALIDATION: Verification of the entire system
    ├── common/                  # Shared test utilities & static mock CSV/JSON data
    ├── test_e2e_pipeline.rs     # End-to-End: Verifies a full run from Webhook to API Dispatch
    └── test_identity_logic.rs   # Verifies resolution engine accuracy with complex duplicates
3. Strategic Implementation Directives
3.1. The "Source of Truth" (SoT) Context
Each column in your Polars DataFrame should ideally track its origin. When an agent implements domain/roster.rs, the RosterContext must support "Field Ownership."

Agent Task: Implement a metadata map that identifies if a field was mastered by the "HRIS Connector" or modified by a "Logic Action."

3.2. SCD Type 2 with Polars
Instead of row-based looping, use Polars window functions to calculate effective dates.

Agent Task: Use .shift() and .over() on the employee_id to determine when a record changed and set the is_current flag.

3.3. Test-Driven Development (TDD) for the Agent
The agent must not consider a capability "complete" without a local #[cfg(test)] block.

Unit Tests: Must mock the RosterContext using a small df![] macro to verify the specific logic of that file (e.g., ensuring the name_cleaner.rs actually capitalizes strings).

Integration Tests: Must use the factory.rs to ensure that a JSON configuration correctly spins up the intended Rust structs.


Getty Images
4. Summary of the Mediator Execution Flow
Ingest: main.rs receives an AWS Lambda event.

Resolve: Mediator calls the Factory to get the list of OnboardingAction trait objects defined in the Manifest.

Process: The PipelineRunner folds the RosterContext through each Action.

Step A (Ingestion): Populates the LazyFrame.

Step B (Validation): Drops malformed rows.

Step C (Logic): Appends SCD Type 2 columns.

Finalize: The Egress action calls .collect() on the LazyFrame and sends the JSON payload to the destination.