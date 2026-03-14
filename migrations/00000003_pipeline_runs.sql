-- Pipeline run history: tracks every ETL execution with status, timing,
-- warnings, validation results, and error context.

CREATE TABLE IF NOT EXISTS pipeline_runs (
    id                   TEXT PRIMARY KEY,             -- UUID per run
    organization_id      TEXT NOT NULL,
    customer_company_id  TEXT NOT NULL,
    status               TEXT NOT NULL DEFAULT 'running',  -- running | success | failed | validation_failed
    started_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at          TIMESTAMPTZ,
    rows_processed       INTEGER,
    current_action       TEXT,                          -- action_id currently executing
    error_message        TEXT,
    error_action_id      TEXT,                          -- which action failed
    error_row            INTEGER,                       -- row index if applicable
    warnings             JSONB NOT NULL DEFAULT '[]',   -- [{action_id, message, count, detail}]
    validation_result    JSONB,                         -- StepValidation[] from dry-run

    FOREIGN KEY (organization_id) REFERENCES organisation(id) ON DELETE CASCADE,
    FOREIGN KEY (organization_id, customer_company_id)
        REFERENCES pipeline_configs(organization_id, customer_company_id) ON DELETE CASCADE
);

-- Index for listing runs by org + customer (most common query path)
CREATE INDEX IF NOT EXISTS idx_pipeline_runs_org_customer
    ON pipeline_runs (organization_id, customer_company_id, started_at DESC);
