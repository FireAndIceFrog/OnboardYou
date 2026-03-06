import type { Node, Edge } from '@xyflow/react';
import type { PipelineConfig, ValidationResult, WorkdayResponseGroup, SageHrConfig as SageHrConfigApi } from '@/generated/api';

export interface ConfigDetailsState {
  config: PipelineConfig | null;
  nodes: Node[];
  edges: Edge[];
  selectedNode: Node | null;
  isLoading: boolean;
  isSaving: boolean;
  isDeleting: boolean;
  isValidating: boolean;
  error: string | null;
  chatOpen: boolean;
  addStepPanelOpen: boolean;
  /** Index at which the next added step will be inserted (null = append) */
  insertIndex: number | null;
  validationResult: ValidationResult | null;
  /** Per-action validation errors: actionId → error message */
  validationErrors: Record<string, string>;
}

/* ── Connection wizard types ─────────────────────────────── */

/** Only connectors backed by Rust ingestion code. */
export const HR_SYSTEMS = [
  { id: 'workday', nameKey: 'configDetails.connection.systems.workday', icon: '🏢' },
  { id: 'sage_hr', nameKey: 'configDetails.connection.systems.sage_hr', icon: '🌿' },
  { id: 'csv', nameKey: 'configDetails.connection.systems.csv', icon: '📄' },
] as const;

export type SystemId = (typeof HR_SYSTEMS)[number]['id'];

export interface WorkdayFields {
  tenantUrl: string;
  tenantId: string;
  username: string;
  password: string;
  workerCountLimit: string;
  responseGroup: string;
}

export interface SageHrFields {
  subdomain: string;
  apiToken: string;
  includeTeamHistory: boolean;
  includeEmploymentStatusHistory: boolean;
  includePositionHistory: boolean;
}

export type CsvUploadStatus = 'idle' | 'uploading' | 'discovering' | 'done' | 'error';

export interface CsvFields {
  filename: string;
  columns: string[];
  uploadStatus: CsvUploadStatus;
  uploadError: string | null;
}

export interface ConnectionForm {
  system: SystemId | '';
  displayName: string;
  workday: WorkdayFields;
  sageHr: SageHrFields;
  csv: CsvFields;
}

export const INITIAL_CONNECTION_FORM: ConnectionForm = {
  system: '',
  displayName: '',
  workday: {
    tenantUrl: '',
    tenantId: '',
    username: '',
    password: '',
    workerCountLimit: '200',
    responseGroup: 'include_personal_information,include_employment_information',
  },
  sageHr: {
    subdomain: '',
    apiToken: '',
    includeTeamHistory: false,
    includeEmploymentStatusHistory: false,
    includePositionHistory: false,
  },
  csv: {
    filename: '',
    columns: [],
    uploadStatus: 'idle',
    uploadError: null,
  },
};

/**
 * Response group toggle options.
 *
 * `value` matches the boolean field name on the generated
 * `WorkdayResponseGroup` type so we can build the object directly
 * from the active toggles.
 */
export const RESPONSE_GROUP_OPTIONS = [
  { value: 'include_personal_information', labelKey: 'configDetails.connection.responseGroupLabels.include_personal_information' },
  { value: 'include_employment_information', labelKey: 'configDetails.connection.responseGroupLabels.include_employment_information' },
  { value: 'include_compensation', labelKey: 'configDetails.connection.responseGroupLabels.include_compensation' },
  { value: 'include_organizations', labelKey: 'configDetails.connection.responseGroupLabels.include_organizations' },
  { value: 'include_roles', labelKey: 'configDetails.connection.responseGroupLabels.include_roles' },
] as const;

/**
 * Convert the comma-separated toggle string kept in the connection
 * form into a typed `WorkdayResponseGroup` object for the manifest.
 */
export function buildResponseGroup(csv: string): WorkdayResponseGroup {
  const active = new Set(csv.split(',').filter(Boolean));
  return {
    include_personal_information: active.has('include_personal_information'),
    include_employment_information: active.has('include_employment_information'),
    include_compensation: active.has('include_compensation'),
    include_organizations: active.has('include_organizations'),
    include_roles: active.has('include_roles'),
  };
}

/* ── Sage HR ──────────────────────────────────────────────── */

/**
 * History toggle options for the Sage HR connection wizard.
 * `value` maps to the `SageHrFields` boolean key.
 */
export const SAGE_HR_HISTORY_OPTIONS = [
  { value: 'includeTeamHistory', labelKey: 'configDetails.connection.historyLabels.includeTeamHistory' },
  { value: 'includeEmploymentStatusHistory', labelKey: 'configDetails.connection.historyLabels.includeEmploymentStatusHistory' },
  { value: 'includePositionHistory', labelKey: 'configDetails.connection.historyLabels.includePositionHistory' },
] as const;

/**
 * Convert the wizard form fields into a typed `SageHrConfig` for the manifest.
 */
export function buildSageHrConfig(fields: SageHrFields): SageHrConfigApi {
  return {
    subdomain: fields.subdomain.trim(),
    api_token: fields.apiToken,
    include_team_history: fields.includeTeamHistory || undefined,
    include_employment_status_history: fields.includeEmploymentStatusHistory || undefined,
    include_position_history: fields.includePositionHistory || undefined,
  };
}
