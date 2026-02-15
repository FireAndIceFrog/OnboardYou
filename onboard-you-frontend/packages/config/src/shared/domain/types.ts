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

/**
 * Business-friendly labels for pipeline action types.
 * Maps technical action names to plain-English descriptions.
 */
export const ACTION_BUSINESS_LABELS: Record<string, string> = {
  // Ingestion
  csv_hris_connector: 'Import CSV File',
  workday_hris_connector: 'Connect to Workday',
  // Logic / Transform
  scd_type_2: 'Track Change History',
  pii_masking: 'Mask Sensitive Data',
  identity_deduplicator: 'Remove Duplicates',
  regex_replace: 'Clean Up Text',
  iso_country_sanitizer: 'Standardise Country Codes',
  cellphone_sanitizer: 'Clean Phone Numbers',
  handle_diacritics: 'Fix Special Characters',
  rename_column: 'Rename Fields',
  drop_column: 'Remove Unused Fields',
  filter_by_value: 'Filter Records',
  // Egress
  api_dispatch: 'Send to API',
  dynamodb_writer: 'Save to Database',
  s3_writer: 'Export to Storage',
};

export function businessLabel(actionType: string): string {
  return ACTION_BUSINESS_LABELS[actionType] ?? actionType;
}

/**
 * Converts a cron/rate expression to a human-readable sync frequency string.
 */
export function humanFrequency(cron: string): string {
  if (!cron) return 'Manual';
  const rateMatch = cron.match(/^rate\((\d+)\s+(minute|hour|day|week|month)s?\)$/i);
  if (rateMatch) {
    const num = parseInt(rateMatch[1], 10);
    const unit = rateMatch[2].toLowerCase();
    if (num === 1) return `Every ${unit}`;
    return `Every ${num} ${unit}s`;
  }
  if (cron.startsWith('cron(')) {
    // Parse common patterns
    if (cron.includes('MON')) return 'Every Monday';
    if (cron.includes('* * *')) return 'Every minute';
    return 'Custom schedule';
  }
  return cron;
}

/**
 * Derives a health status from the pipeline config for display.
 */
export type SystemStatus = 'healthy' | 'syncing' | 'paused' | 'needs-attention';

export function deriveStatus(config: PipelineConfig): SystemStatus {
  // Simple heuristic — could be replaced by API data later
  const lastEdited = new Date(config.lastEdited).getTime();
  const now = Date.now();
  const hoursSinceEdit = (now - lastEdited) / (1000 * 60 * 60);

  if (hoursSinceEdit < 1) return 'syncing';
  if (config.pipeline.actions.length === 0) return 'needs-attention';
  if (hoursSinceEdit > 24 * 30) return 'paused'; // stale > 30 days
  return 'healthy';
}

import type { BadgeVariant } from '../ui/Badge';

export const STATUS_DISPLAY: Record<SystemStatus, { label: string; variant: BadgeVariant }> = {
  healthy: { label: 'Healthy', variant: 'active' },
  syncing: { label: 'Syncing', variant: 'info' },
  paused: { label: 'Paused', variant: 'paused' },
  'needs-attention': { label: 'Needs Attention', variant: 'error' },
};

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
}

export type NotificationType = 'success' | 'error' | 'warning' | 'info';

export interface ApiErrorResponse {
  error: string;
}
