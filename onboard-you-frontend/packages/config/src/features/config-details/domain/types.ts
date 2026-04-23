import type { ComponentType, SVGProps } from 'react';
import type { Node, Edge } from '@xyflow/react';
import type { PipelineConfig, ValidationResult, WorkdayResponseGroup, SageHrConfig as SageHrConfigApi } from '@/generated/api';
import { ConnectorConfigFactory, ConnectorType } from '../state/connectorConfigs/connectorConfigFactory';
import { OfficeBuildingIcon, LeafIcon, FolderOpenIcon } from '@/shared/ui';

const connectorFactory = new ConnectorConfigFactory();

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
  addStepPanelOpen: boolean;
  /** Index at which the next added step will be inserted (null = append) */
  insertIndex: number | null;
  validationResult: ValidationResult | null;
  /** Per-action validation errors: actionId → error message */
  validationErrors: Record<string, string>;
  /** Column schema from org settings (field name → type), shown for default auth */
  settingsSchema: Record<string, string> | null;
}

/* ── Connection wizard types ─────────────────────────────── */

/** Only connectors backed by Rust ingestion code. */
export const HR_SYSTEMS = [
  { id: ConnectorType.Workday, nameKey: 'configDetails.connection.systems.workday', icon: OfficeBuildingIcon },
  { id: ConnectorType.SageHR, nameKey: 'configDetails.connection.systems.sage_hr', icon: LeafIcon },
  { id: ConnectorType.GenericIngestion, nameKey: 'configDetails.connection.systems.generic_ingestion', icon: FolderOpenIcon },
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

/** Upload status for the generic ingestion connector. */
export type GenericUploadStatus = 'idle' | 'uploading' | 'done' | 'error';

/** Form fields for the generic ingestion connector (any file type). */
export interface GenericIngestionFields {
  filename: string;
  /** Column header overrides. Empty = auto-detect from source. */
  columns: string[];
  uploadStatus: GenericUploadStatus;
  uploadError: string | null;
  /** `null` = not yet uploaded, `"not_needed"` = was CSV, `"converted"` = converted synchronously. */
  conversionStatus: 'not_needed' | 'converted' | null;
}

/** Per-field validation error map (field path → error message). */
export type ValidationErrors = Record<string, string | undefined>;

export interface ConnectionForm {
  system: SystemId;
  displayName: string;
  workday: WorkdayFields;
  sageHr: SageHrFields;
  genericIngestion: GenericIngestionFields;
}

export const INITIAL_CONNECTION_FORM: ConnectionForm = {
  system: ConnectorType.GenericIngestion,
  displayName: '',
  workday: connectorFactory.getConfig(ConnectorType.Workday).getDefaultState().workday!,
  sageHr: connectorFactory.getConfig(ConnectorType.SageHR).getDefaultState().sageHr!,
  genericIngestion: connectorFactory.getConfig(ConnectorType.GenericIngestion).getDefaultState().genericIngestion!,
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

/** Build a response-group boolean map from a CSV string of active group keys. */
export function buildResponseGroup(csv: string): Record<string, boolean> {
  const active = new Set(csv.split(',').filter(Boolean));
  return Object.fromEntries(RESPONSE_GROUP_OPTIONS.map((opt) => [opt.value, active.has(opt.value)]));
}

/* ── Sage HR ──────────────────────────────────────────────── */

/**
 * History toggle options for the Sage HR connection wizard.
 * `value` maps to the `SageHrFields` boolean key.
 */
export const SAGE_HR_HISTORY_OPTIONS = [
  { value: 'includeTeamHistory', configKey: 'include_team_history', labelKey: 'configDetails.connection.historyLabels.includeTeamHistory' },
  { value: 'includeEmploymentStatusHistory', configKey: 'include_employment_status_history', labelKey: 'configDetails.connection.historyLabels.includeEmploymentStatusHistory' },
  { value: 'includePositionHistory', configKey: 'include_position_history', labelKey: 'configDetails.connection.historyLabels.includePositionHistory' },
] as const;

/** Build the Sage HR API config payload from form fields. */
export function buildSageHrConfig(fields: SageHrFields): Record<string, unknown> {
  return {
    subdomain: fields.subdomain.trim(),
    api_token: fields.apiToken,
    include_team_history: fields.includeTeamHistory || undefined,
    include_employment_status_history: fields.includeEmploymentStatusHistory || undefined,
    include_position_history: fields.includePositionHistory || undefined,
  };
}