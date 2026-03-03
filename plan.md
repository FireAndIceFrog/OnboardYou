# Plan: Schema Inference & AI-Driven Pipeline Planner

## Overview

Extend the existing `ValidationEngine` (which already runs `ColumnCalculator` through the full pipeline) to also diff `final_columns` against the egress model's `schema` HashMap (`DynamicEgressModel`). Feed that diff to GitHub Models AI to generate both a recommended pipeline manifest and human-readable summary text. Surface this as a **Normal/Advanced toggle** on the pipeline editor — Normal = the plan summary UI with feature cards, toggles, and a synthetic before/after preview; Advanced = the existing React Flow editor. The generated plan is cached on `PipelineConfig` via a new `plan_summary` field.

The plan summary is a **read-only projection** of the underlying manifest. Each `PlanFeature` references the manifest action IDs it maps to, and each manifest action carries a `disabled: bool` flag. Toggling a feature in Normal mode sets `disabled` on its associated actions — no actions are ever added or removed. The ETL engine skips disabled actions at runtime.

---

## Architecture Diagram

```
 ┌──────────────────┐      ┌─────────────────────────────────┐
 │  Connection       │      │  ValidationEngine (exists)       │
 │  Wizard (exists)  │─────▶│  ColumnCalculator pipeline run   │
 │  CSV / Workday    │      │  → final_columns                 │
 └──────────────────┘      └──────────┬──────────────────────┘
                                       │
                                       ▼
                            ┌──────────────────────┐
                            │  Schema Diff (new)    │
                            │  final_columns vs     │
                            │  egress schema HashMap│
                            └──────────┬─────────────┘
                                       │
                                       ▼
                            ┌──────────────────────┐
                            │  Plan Generation      │
                            │  Engine (new)          │
                            │  - gh_models AI call  │
                            │  - Manifest output    │
                            │  - PlanSummary output  │
                            └──────────┬─────────────┘
                                       │
                                       ▼
                            ┌──────────────────────┐
                            │  PipelineConfig       │
                            │  + plan_summary field  │
                            │  (cached in DynamoDB) │
                            └──────────┬─────────────┘
                                       │
                          ┌────────────┴────────────┐
                          ▼                         ▼
                ┌─────────────────┐      ┌──────────────────┐
                │  Normal Mode    │      │  Advanced Mode    │
                │  Plan Summary   │      │  React Flow       │
                │  UI (new)       │      │  Editor (exists)  │
                └─────────────────┘      └──────────────────┘
```

---

## Existing Infrastructure to Leverage

| Asset | Location | Relevance |
|-------|----------|-----------|
| `WORKDAY_COLUMNS` (18 fields) | `ETL/src/capabilities/ingestion/engine/workday_hris_connector.rs` | Known input schema for Workday |
| `CsvHrisConnectorConfig.columns` | `models/src/models/pipeline_models/ingestion/csv_config.rs` | Known input schema for CSV |
| `DynamicEgressModel!` macro | `models/src/traits/dynamic_egress_model.rs` | Injects `schema: HashMap<String,String>` + `body_path` on egress configs |
| `ValidationEngine` | `api/src/engine/validation_engine.rs` | Dry-run column propagation — validates generated manifests |
| `SchemaGenerationStatus` enum | `models/src/models/org_settings.rs` | Already declared (`NotStarted`, `InProgress`, `Completed`, `Failed`) — **unused** |
| `gh_models` crate (v0.2.0) | Workspace dependency, wired into `etl-trigger` lambda | GitHub Models LLM access |
| `GITHUB_TOKEN` env var | `infra/variables.tf` → `var.gh_token` → Lambda env | Already provisioned in Terraform |
| `ACTION_CATALOG` | `config/src/features/config-details/domain/actionCatalog.ts` | 11 action definitions with labels, descriptions, categories |
| `ActionType` enum (13 variants) | `models/src/models/manifest.rs` | All registered pipeline actions |
| `ColumnCalculator` trait | `ETL/src/domain/mod.rs` | Schema propagation without data — used by validation engine |

---

## Tickets

### Ticket 1: Schema Diff via Existing ColumnCalculator — Backend

**Goal:** Extend the existing `ValidationEngine` to compare the pipeline's `final_columns` (already computed by `ColumnCalculator`) against the egress model's `schema` HashMap. No new crate or traits needed — this builds on the existing column propagation infrastructure.

**Location:** `onboard-you-backend/lambdas/api/src/engine/validation_engine.rs`

**Context — what already exists:**
- `ColumnCalculator` trait — every action implements `calculate_columns(RosterContext) → RosterContext`, propagating schemas without data
- `validate_pipeline()` — folds `calculate_columns` through the full manifest, returns `ValidationResult { steps, final_columns }`
- `DynamicEgressModel` — macro injects `schema: HashMap<String, String>` on egress configs (maps pipeline column → destination field)
- Ingress connectors already declare their columns: Workday = 18 hardcoded fields, CSV = `config.columns`

**Deliverables:**

1. Add a `SchemaDiff` struct to the API models:
   ```rust
   pub struct SchemaDiff {
       /// Columns present in the pipeline that have a mapping in egress schema
       pub mapped: Vec<ColumnMapping>,       // (pipeline_col, destination_field)
       /// Pipeline columns with no egress mapping
       pub unmapped_source: Vec<String>,
       /// Egress schema fields with no matching pipeline column
       pub unmapped_target: Vec<String>,
   }

   pub struct ColumnMapping {
       pub source_column: String,
       pub target_field: String,
   }
   ```

2. Add a `compute_schema_diff()` function in `validation_engine.rs`:
   - Takes the `final_columns: Vec<String>` from the existing `validate_pipeline()` result
   - Takes the egress action's `schema: HashMap<String, String>` from the manifest's last action (via `DynamicEgressModel::get_schema()`)
   - Diffs them: which pipeline columns map to a destination field, which are unmapped on each side
   - Returns `SchemaDiff`

3. Optionally extend `ValidationResult` to include `schema_diff: Option<SchemaDiff>` so the diff is returned alongside validation

**What we're NOT doing:**
- No new crate — this is a small addition to the existing `validation_engine.rs`
- No new traits — `ColumnCalculator` and `DynamicEgressModel` already exist
- No new resolvers — the connectors already declare their schemas via `calculate_columns()`

**Tests:**
- Unit: `compute_schema_diff()` with all columns mapped → empty unmapped lists
- Unit: `compute_schema_diff()` with partial mapping → correct unmapped on both sides
- Unit: `compute_schema_diff()` with empty egress schema → all source columns unmapped
- Integration: full pipeline validation + diff returns consistent results

**Acceptance:** `cargo test` passes. `SchemaDiff` is available for the plan generation engine (Ticket 2).

---

### Ticket 2: Async AI Plan Generation via SQS — Backend

**Goal:** Async plan generation flow: the API sends an SQS message and returns immediately with `InProgress` status. The etl-trigger lambda picks it up, calls `gh_models` to generate the plan, and writes the result back to DynamoDB on the `PipelineConfig`. The frontend polls until completion.

**Why async:** The GitHub Models AI call can take several seconds. Blocking the API Gateway request would cause stalls and risk Lambda timeout on cold starts. The existing SQS infrastructure already connects the API lambda → etl-trigger lambda.

**Existing infrastructure:**
- `aws_sqs_queue.etl_events` — SQS queue already exists
- API lambda has `SQS_QUEUE_URL` env var + `sqs:SendMessage` permission
- etl-trigger lambda has SQS event source mapping (`batch_size = 1`)
- `ScheduledEvent` enum already has tagged dispatch — we add a new variant

**Location:**
- **API side:** `onboard-you-backend/lambdas/api/src/engine/plan_engine.rs` (SQS send + status check)
- **ETL-trigger side:** `onboard-you-backend/lambdas/etl-trigger/src/engine/plan_generation_engine.rs` (AI call + DynamoDB write-back)

**Deliverables:**

1. **Extend `ScheduledEvent` enum** with a new variant in `models/src/models/scheduled_event.rs`:
   ```rust
   #[serde(rename = "GeneratePlanEvent")]
   GeneratePlan(GeneratePlanEvent),
   ```
   ```rust
   pub struct GeneratePlanEvent {
       pub organization_id: String,
       pub customer_company_id: String,
       pub source_system: String,     // "Workday" or "CSV"
   }
   ```

2. **API lambda — trigger side** (`plan_engine.rs`):
   - `POST /config/{id}/generate-plan` handler:
     1. Set `plan_summary.generation_status = InProgress` on the `PipelineConfig` in DynamoDB. If there is already an inprogress flag for the config it should return immediately for idempotency
     2. Send `GeneratePlanEvent` to the existing SQS queue via `aws-sdk-sqs` (already a dependency, `SQS_QUEUE_URL` already available)
     3. Return `202 Accepted` with `{ "status": "InProgress" }` immediately
   - `GET /config/{id}` — already returns the full `PipelineConfig`, so the frontend can poll this to check `plan_summary.generation_status`

3. **ETL-trigger lambda — execution side** (`plan_generation_engine.rs`):
   - Handle `ScheduledEvent::GeneratePlan(event)` in `main.rs` dispatch
   - Add `gh_models` dependency (already in `Cargo.toml`, currently unused)
   - Fetch the `PipelineConfig` from DynamoDB (already has `GetItem` permission)
   - Run `validate_pipeline()` + `compute_schema_diff()` to get the schema context
   - Construct a structured prompt including:
     - Available `ActionType` variants with descriptions and valid config shapes
     - The pipeline's `final_columns` from validation
     - The `SchemaDiff` showing mapped/unmapped columns
     - The source system name for context
     - Instruction to return structured JSON matching the `GeneratedPlan` shape
   - Call `gh_models` with the prompt, parse response into `GeneratedPlan { manifest, summary }`
   - Post-validate: run the generated `Manifest` through `ActionFactory::create()` to verify it's valid
   - Fallback: if AI fails, generate a deterministic default (rename mapped fields, drop unmapped, generic summary text)
   - **Write back** to DynamoDB: update `PipelineConfig.plan_summary` with the result + `generation_status = Completed` (or `Failed(msg)`)

4. **Add DynamoDB write permission** to etl-trigger IAM policy (currently read-only):
   - Add `dynamodb:UpdateItem` for the config table (scoped to `plan_summary` attribute updates)

**AI Response Schema (structured output):**

Each manifest action includes a `disabled` flag (defaulting to `false`). Each summary feature includes `action_ids` referencing the actions it controls. This makes the summary a read-only projection — toggling a feature just flips `disabled` on its linked actions.

```json
{
  "manifest": {
    "version": "1.0",
    "actions": [
      { "id": "step_1", "action_type": "workday_hris_connector", "disabled": false, "config": { ... } },
      { "id": "step_2", "action_type": "filter_by_value", "disabled": false, "config": { "column": "worker_status", "value": "Active" } },
      { "id": "step_3", "action_type": "rename_column", "disabled": false, "config": { "mapping": { "hire_date": "startDate" } } },
      { "id": "step_4", "action_type": "identity_deduplicator", "disabled": false, "config": { "match_column": "work_email" } },
      { "id": "step_5", "action_type": "api_dispatcher", "disabled": false, "config": { ... } }
    ]
  },
  "summary": {
    "headline": "Here's the plan to connect Workday to your App.",
    "description": "We've designed a simple sync for your active employees, running daily at 2 AM.",
    "features": [
      {
        "id": "sync_start_dates",
        "icon": "calendar",
        "label": "Sync Start Dates",
        "description": "Use the employee's original start date.",
        "action_ids": ["step_3"]
      },
      {
        "id": "active_only",
        "icon": "users",
        "label": "Active Employees Only",
        "description": "Only sync people currently employed.",
        "action_ids": ["step_2"]
      },
      {
        "id": "prevent_duplicates",
        "icon": "shield",
        "label": "Prevent Duplicates",
        "description": "Match people by work email to avoid copies.",
        "action_ids": ["step_4"]
      }
    ],
    "preview": {
      "source_label": "In Workday",
      "target_label": "In Your App",
      "before": {
        "name": "Jane Doe",
        "status": "Active",
        "email": "jane.doe@example.com",
        "dept": "Sales_NE_123",
        "start": "2023-01-15"
      },
      "after": {
        "name": "Jane Doe",
        "email": "jane.doe@example.com",
        "department": "Northeast Sales",
        "startDate": "Jan 15, 2023"
      }
    }
  }
}
```

**Async flow diagram:**
```
Frontend                    API Lambda                SQS Queue          ETL-Trigger Lambda       DynamoDB
   │                           │                        │                       │                    │
   │ POST /generate-plan       │                        │                       │                    │
   │──────────────────────────▶│                        │                       │                    │
   │                           │ UpdateItem: status=InProgress                  │                    │
   │                           │────────────────────────────────────────────────────────────────────▶│
   │                           │ SendMessage(GeneratePlanEvent)                 │                    │
   │                           │───────────────────────▶│                       │                    │
   │   202 { InProgress }      │                        │                       │                    │
   │◀──────────────────────────│                        │                       │                    │
   │                           │                        │  Poll                 │                    │
   │                           │                        │──────────────────────▶│                    │
   │                           │                        │                       │ GetItem(config)    │
   │                           │                        │                       │───────────────────▶│
   │                           │                        │                       │  validate + diff   │
   │                           │                        │                       │  call gh_models    │
   │                           │                        │                       │  UpdateItem:       │
   │                           │                        │                       │  plan_summary +    │
   │                           │                        │                       │  status=Completed  │
   │                           │                        │                       │───────────────────▶│
   │                           │                        │                       │                    │
   │ GET /config/{id} (poll)   │                        │                       │                    │
   │──────────────────────────▶│                        │                       │                    │
   │                           │ GetItem                │                       │                    │
   │                           │────────────────────────────────────────────────────────────────────▶│
   │  200 { planSummary: {     │                        │                       │                    │
   │    status: Completed,     │                        │                       │                    │
   │    ...plan data           │                        │                       │                    │
   │  }}                       │                        │                       │                    │
   │◀──────────────────────────│                        │                       │                    │
```

5. **PlanSummary model + PipelineConfig field** (prerequisite — do this first):
   - Add `PlanSummary`, `PlanFeature`, `PlanPreview` structs to `onboard-you-backend/shared/models/src/models/plan_summary.rs`:
     ```rust
     #[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
     #[serde(rename_all = "camelCase")]
     pub struct PlanSummary {
         pub headline: String,
         pub description: String,
         pub features: Vec<PlanFeature>,
         pub preview: PlanPreview,
         pub generation_status: SchemaGenerationStatus,
     }

     #[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
     #[serde(rename_all = "camelCase")]
     pub struct PlanFeature {
         pub id: String,
         pub icon: String,
         pub label: String,
         pub description: String,
         /// References to the manifest action IDs this feature maps to
         pub action_ids: Vec<String>,
     }

     #[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
     #[serde(rename_all = "camelCase")]
     pub struct PlanPreview {
         pub source_label: String,
         pub target_label: String,
         pub before: HashMap<String, String>,
         pub after: HashMap<String, String>,
     }
     ```
   - Add `pub plan_summary: Option<PlanSummary>` to `PipelineConfig`
   - Add `#[serde(default)] pub disabled: bool` to the manifest `Action` struct in `manifest.rs` — the ETL engine skips actions where `disabled == true`
   - Wire `SchemaGenerationStatus` from `org_settings.rs` (already exists, finally gets used)
   - Update OpenAPI annotations (auto-generated via `utoipa`)
   - Regenerate frontend API client via `pnpm openapi-ts`

**Files to modify:**
- `onboard-you-backend/shared/models/src/models/scheduled_event.rs` — new `GeneratePlan` variant
- `onboard-you-backend/shared/models/src/models/plan_summary.rs` — new file (PlanSummary, PlanFeature, PlanPreview)
- `onboard-you-backend/shared/models/src/models/pipeline_config.rs` — add `plan_summary` field
- `onboard-you-backend/shared/models/src/models/manifest.rs` — add `disabled: bool` to `Action` struct
- `onboard-you-backend/shared/models/src/models/org_settings.rs` — re-export `SchemaGenerationStatus`
- `onboard-you-backend/lambdas/api/src/engine/plan_engine.rs` — SQS send + status set (new file)
- `onboard-you-backend/lambdas/api/src/main.rs` — new route
- `onboard-you-backend/lambdas/etl-trigger/src/main.rs` — dispatch new event variant
- `onboard-you-backend/lambdas/etl-trigger/src/engine/plan_generation_engine.rs` — AI call + write-back (new file)
- `infra/main.tf` — add `dynamodb:UpdateItem` to etl-trigger IAM policy
- `openapi.json` — regenerated
- Frontend generated client — regenerated

**Tests:**
- Unit: `PipelineConfig` serialization round-trips with `plan_summary: Some(...)` and `None`
- Unit: `PlanSummary` serialization with all `SchemaGenerationStatus` variants
- Unit: `GeneratePlanEvent` serialization round-trips
- Unit: prompt construction includes all required context
- Unit: AI response parsing handles valid JSON
- Unit: AI response parsing handles malformed responses → falls back gracefully
- Unit: fallback manifest generation works
- Integration: `ActionFactory::create()` accepts every action in the generated manifest
- Smoke: POST generate-plan returns 202, poll GET returns Completed with plan data

**Acceptance:** `cargo test` passes. Models, SQS trigger, AI engine, and DynamoDB write-back all work end-to-end.

---

### Ticket 3: Plan Summary UI — Normal Mode — Frontend

**Goal:** Build the plan summary screen matching the mockup, accessible via Normal/Advanced toggle on the pipeline editor.

**Location:** `onboard-you-frontend/packages/config/src/features/config-details/`

**Deliverables:**

1. **New components** in `ui/components/plan-summary/`:
   - `PlanSummaryView.tsx` — main container matching the screenshot layout:
     - Header: "Here's the plan to connect {source} to your App."
     - Subtitle: dynamic description text
     - "How it will work" section: grid of `PlanFeatureCard` components
     - "Preview" section: before/after employee card comparison
     - Action buttons: "Looks Good, Start Syncing" + "Make Changes"
   - `PlanFeatureCard.tsx` — individual feature card with icon, label, description, toggle switch
   - `PlanPreviewCard.tsx` — side-by-side before/after employee display with arrow between
   - `NormalAdvancedToggle.tsx` — pill toggle (Normal / Advanced) that switches views

2. **State updates** in `configDetailsSlice.ts`:
   - Add `planSummary: PlanSummary | null` to state
   - Add `viewMode: 'normal' | 'advanced'` to state (default `'normal'` if `planSummary` exists)
   - Add `generatePlanThunk` async thunk: calls `POST /config/{id}/generate-plan` (returns 202), then polls `GET /config/{id}` until `planSummary.generationStatus === 'Completed'` or `'Failed'`
   - Add `toggleFeature(featureId)` reducer — looks up the feature's `action_ids`, sets `disabled` on those manifest actions (feature enabled → `disabled: false`, feature disabled → `disabled: true`), and updates the feature's `enabled` flag in the plan summary. This is the only mutation — no actions are added or removed.
   - Add `applyPlan` thunk — saves the manifest (with current `disabled` flags) as the pipeline config

3. **Screen integration** in `ConfigDetailsScreen.tsx`:
   - After connection wizard completes → auto-trigger `generatePlanThunk`
   - While generating (status = `InProgress`): show skeleton/loading state with progress message
   - Poll `GET /config/{id}` every 2–3 seconds until `generationStatus` flips to `Completed` or `Failed`
   - Once generated: show `PlanSummaryView` (Normal mode)
   - `NormalAdvancedToggle` switches between `PlanSummaryView` and existing React Flow editor
   - "Looks Good, Start Syncing" → saves the manifest and navigates to config list
   - "Make Changes" → switches to Advanced mode (React Flow editor)

4. **Services** in `services/planService.ts`:
   - `generatePlan(configId: string): Promise<PlanSummary>` — calls the new API endpoint
   - No `applyPlanFeatureToggle` needed — toggling is just setting `disabled` on actions via the reducer

**Component hierarchy:**
```
ConfigDetailsScreen
├── NormalAdvancedToggle
├── [Normal mode]
│   └── PlanSummaryView
│       ├── PlanFeatureCard (×N)
│       └── PlanPreviewCard
│           ├── BeforeCard (source system)
│           └── AfterCard (your app)
└── [Advanced mode]
    └── ReactFlow editor (existing)
```

**Design tokens (from screenshot):**
- Feature cards: white background, subtle border, rounded corners
- Toggle: purple active state, grey inactive
- Preview cards: left = grey border, right = purple border
- "Looks Good" button: purple gradient, full width
- "Make Changes" button: outlined/secondary style

**Tests:**
- Component renders with mock PlanSummary data
- Feature toggle sets `disabled` on linked manifest actions via `action_ids`
- Toggling a feature off sets `disabled: true` on the correct actions
- Toggling it back on sets `disabled: false` — no actions added or removed
- Normal/Advanced toggle switches views
- "Looks Good" triggers save
- "Make Changes" switches to Advanced mode
- Loading state renders skeleton while generating

**Acceptance:** UI matches the mockup screenshot. Toggle between Normal and Advanced works. Feature toggles set `disabled` on the linked manifest actions. The plan summary is a read-only view derived from the manifest.

---

### Ticket 4: Destination Schema Configuration UI — Frontend

**Goal:** Allow users to define the expected output schema (what their destination API expects) before plan generation.

**Context:** The `DynamicEgressModel` macro injects a `schema: HashMap<String, String>` onto egress configs. This is the mapping from pipeline columns → destination API fields. Currently there is no UI for editing this.

**Deliverables:**

1. Add a step to the connection wizard (or as part of egress config) where users can define their destination schema:
   - Option A: Paste an OpenAPI spec URL → fetch and parse to extract request body schema
   - Option B: Manually define field names and types
   - Option C: Upload a sample JSON payload → infer schema from structure

2. Build `DestinationSchemaEditor` component:
   - Table/list of expected output fields
   - Add/remove/rename fields
   - Type selector (string, number, date, boolean)
   - Visual indicator of which input columns map to which output fields

3. Store the result as the egress config's `schema` HashMap and `body_path` (JSON path to the data array in the request body)

4. This schema feeds into the egress `DynamicEgressModel` config, which is then diffed against `final_columns` by `compute_schema_diff()` (Ticket 1) during plan generation (Ticket 2)

**Tests:**
- Schema editor renders and allows field CRUD
- OpenAPI spec parsing extracts request body fields (if Option A implemented)
- Schema persists on egress config save

**Acceptance:** Users can define what their destination API expects. This information flows into plan generation.

---

### Ticket 5: OpenAPI Spec Fetcher — Backend (Optional Enhancement)

**Goal:** If the customer provides an OpenAPI spec URL for their destination API, fetch and parse it to auto-populate the egress `schema` HashMap.

**Location:** `onboard-you-backend/lambdas/api/src/engine/openapi_engine.rs`

**Deliverables:**

1. Implement `OpenApiSchemaFetcher`:
   - `fetch_spec(url: &str) -> Result<serde_json::Value>`
   - `extract_request_body_schema(spec, path, method) -> Result<HashMap<String, String>>`
2. Parse OpenAPI 3.x JSON/YAML specs
3. Extract field names + types from request body `$ref` schemas
4. Convert to `HashMap<String, String>` for direct use as the egress `schema` field (populates `DynamicEgressModel`)

**Tests:**
- Parse a sample OpenAPI spec and extract correct fields
- Handle nested `$ref` schemas
- Handle missing/malformed specs gracefully

**Acceptance:** Given an OpenAPI spec URL, returns a `HashMap<String, String>` that can be set on the egress config.

---

## Dependency Graph

```
Ticket 1 (Schema Diff via ColumnCalculator)
    │
    └──▶ Ticket 2 (Async AI Plan Generation + Models + SQS)
              │
              ├──▶ Ticket 3 (Plan Summary UI + feature toggle via disabled flag)
              │
              └──▶ Ticket 4 (Destination Schema UI)

Ticket 5 (OpenAPI Spec Fetcher — optional, independent)
```

**Parallelizable:**
- Tickets 1 + 3 (UI shell with mock data) can start simultaneously
- Ticket 4 can start alongside Ticket 3
- Ticket 5 is independent/optional

---

## Estimated Effort

| Ticket | Effort | Priority |
|--------|--------|----------|
| 1 — Schema Diff via ColumnCalculator | 0.5–1 day | P0 |
| 2 — Async AI Plan Generation + Models + SQS | 3–4 days | P0 |
| 3 — Plan Summary UI (incl. feature toggle via `disabled` flag) | 2–3 days | P0 |
| 4 — Destination Schema UI | 2–3 days | P1 |
| 5 — OpenAPI Spec Fetcher | 1–2 days | P2 (optional) |

**Total: ~8–12 days of work**

---

## Key Design Decisions

1. **AI via GitHub Models** — `gh_models` crate already in workspace, `GITHUB_TOKEN` already in Terraform. No new infra needed.

2. **Cache on `PipelineConfig`** — `plan_summary: Option<PlanSummary>` field. Regenerate on demand, not on every edit. `SchemaGenerationStatus` enum already exists unused in `org_settings.rs`.

3. **Normal/Advanced toggle** — Normal mode shows the AI-generated plan summary (screenshot UI). Advanced mode shows the existing React Flow editor. They share the same underlying `Manifest`.

4. **No new crate or traits** — `ColumnCalculator` already propagates schemas, `DynamicEgressModel` already holds the target schema. Ticket 1 is just a diff function on top of existing `validate_pipeline()` output. The connectors (Workday's 18 columns, CSV's declared columns) already declare their schemas via `calculate_columns()`.

5. **Validation engine reuse** — After plan generation, run the generated manifest through the existing `validate_pipeline()` to verify column propagation works end-to-end.

6. **Synthetic preview data** — The before/after preview uses synthetic data generated by the AI (like "Jane Doe") rather than fetching real records. This avoids auth complexity and data privacy concerns during setup.

7. **`disabled` flag on actions** — Each manifest action carries `disabled: bool`. The ETL engine skips disabled actions at runtime. The plan summary's `PlanFeature` references actions via `action_ids` — toggling a feature just flips `disabled` on those actions. No actions are ever added/removed by the toggle. The summary is a read-only projection of the manifest, not an independent data structure.
