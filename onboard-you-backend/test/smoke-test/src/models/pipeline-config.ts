// ── Pipeline config DTOs ────────────────────────────────────

/**
 * All known action types in the pipeline.
 * Values are snake_case strings matching the Rust `ActionType` enum.
 */
export type ActionType =
  | 'generic_ingestion_connector'
  | 'workday_hris_connector'
  | 'scd_type_2'
  | 'pii_masking'
  | 'identity_deduplicator'
  | 'regex_replace'
  | 'iso_country_sanitizer'
  | 'cellphone_sanitizer'
  | 'handle_diacritics'
  | 'rename_column'
  | 'drop_column'
  | 'filter_by_value'
  | 'api_dispatcher';

/** Configuration for a single action in the pipeline. */
export interface ActionConfig {
  /** Unique identifier for this pipeline step. */
  id: string;
  /** Factory key — selects the Rust implementation. */
  action_type: ActionType;
  /** Action-specific configuration (shape depends on action_type). */
  config: Record<string, unknown>;
}

/** Versioned, declarative pipeline manifest. */
export interface Manifest {
  /** Schema version (e.g. "1.0"). */
  version: string;
  /** Ordered list of pipeline actions. */
  actions: ActionConfig[];
}

/** The pipeline config as stored in DynamoDB and exchanged via the API. */
export interface PipelineConfig {
  /** Name of the pipeline. */
  name: string;
  /** Optional image/icon for the pipeline. */
  image?: string;
  /** EventBridge-compatible schedule expression (cron or rate). */
  cron: string;
  /** Unique identifier for the organization (partition key). */
  organizationId: string;
  /** Unique identifier for the customer company (sort key). */
  customerCompanyId: string;
  /** ISO 8601 timestamp of last edit — set by the server. */
  lastEdited?: string;
  /** The full ETL pipeline manifest. */
  pipeline: Manifest;
}
