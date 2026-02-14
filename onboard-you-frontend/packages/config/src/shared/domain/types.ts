/* ── Backend-aligned types ───────────────────────────────── */

/** Matches the Rust PipelineConfig exactly (camelCase via serde) */
export interface PipelineConfig {
  name: string;
  image?: string;
  cron: string;
  organizationId: string;
  customerCompanyId: string;
  lastEdited: string;
  pipeline: Manifest;
}

/** Rust Manifest — flat list of actions */
export interface Manifest {
  version: string;
  actions: ActionConfig[];
}

/** Single pipeline action (Rust ActionConfig) */
export interface ActionConfig {
  id: string;
  actionType: string;
  config: Record<string, unknown>;
}

/** Dry-run validation result */
export interface ValidationResult {
  steps: StepValidation[];
  finalColumns: string[];
}

export interface StepValidation {
  actionId: string;
  actionType: string;
  columnsAfter: string[];
}

/** Known action-type categories for React Flow node styling. */
export const ACTION_CATEGORIES: Record<string, 'ingestion' | 'logic' | 'egress'> = {
  csv_hris_connector: 'ingestion',
  workday_hris_connector: 'ingestion',
  rest_api_connector: 'ingestion',
  odata_connector: 'ingestion',
  sftp_connector: 'ingestion',
  // logic / transform steps
  scd_type_2: 'logic',
  pii_masking: 'logic',
  identity_deduplicator: 'logic',
  regex_replace: 'logic',
  iso_country_sanitizer: 'logic',
  cellphone_sanitizer: 'logic',
  handle_diacritics: 'logic',
  rename_column: 'logic',
  drop_column: 'logic',
  filter_by_value: 'logic',
  // egress steps (dispatched externally today, but may appear in manifests)
  api_dispatch: 'egress',
  dynamodb_writer: 'egress',
  s3_writer: 'egress',
};

export function actionCategory(actionType: string): 'ingestion' | 'logic' | 'egress' {
  return ACTION_CATEGORIES[actionType] ?? 'logic';
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
}

export interface User {
  id: string;
  email: string;
  name: string;
  organizationId: string;
  role: string;
}

export interface Organization {
  id: string;
  name: string;
  plan: string;
}

export type NotificationType = 'success' | 'error' | 'warning' | 'info';

export interface ApiErrorResponse {
  error: string;
}
