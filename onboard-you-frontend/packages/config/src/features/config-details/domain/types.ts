import type { Node, Edge } from '@xyflow/react';
import type { PipelineConfig, ValidationResult, WorkdayResponseGroup } from '@/generated/api';

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
  { id: 'workday', name: 'Workday', icon: '🏢' },
  { id: 'csv', name: 'CSV File Upload', icon: '📄' },
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
  { value: 'include_personal_information', label: 'Personal Information' },
  { value: 'include_employment_information', label: 'Employment Information' },
  { value: 'include_compensation', label: 'Compensation' },
  { value: 'include_organizations', label: 'Organizations' },
  { value: 'include_roles', label: 'Roles' },
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
