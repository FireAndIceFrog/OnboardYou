-- Add manifest_snapshot column to pipeline_runs table.
-- Stores a JSON snapshot of the Manifest at the time the run was created.

ALTER TABLE pipeline_runs
ADD COLUMN IF NOT EXISTS manifest_snapshot JSONB;
