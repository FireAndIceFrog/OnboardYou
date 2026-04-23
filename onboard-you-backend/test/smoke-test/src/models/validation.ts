// ── Validation result DTOs ──────────────────────────────────

/** Result of validating a single step in the pipeline. */
export interface StepValidation {
  /** Action id from the manifest. */
  action_id: string;
  /** Action type (e.g. generic_ingestion_connector, drop_column). */
  action_type: string;
  /** Columns present after this step completes. */
  columns_after: string[];
}

/** Overall validation result for the entire pipeline. */
export interface ValidationResult {
  /** Per-step column snapshots (in execution order). */
  steps: StepValidation[];
  /** Final column set after the last step. */
  final_columns: string[];
}
