import type { Node, Edge } from '@xyflow/react';
import type { PipelineConfig } from '@/generated/api';

export interface ConfigDetailsState {
  config: PipelineConfig | null;
  nodes: Node[];
  edges: Edge[];
  selectedNode: Node | null;
  isLoading: boolean;
  error: string | null;
  chatOpen: boolean;
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

export interface CsvFields {
  csvPath: string;
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
    responseGroup: 'personal_info,employment_info',
  },
  csv: {
    csvPath: '',
  },
};

export const RESPONSE_GROUP_OPTIONS = [
  { value: 'personal_info', label: 'Personal Information' },
  { value: 'employment_info', label: 'Employment Information' },
  { value: 'compensation', label: 'Compensation' },
  { value: 'organizations', label: 'Organizations' },
  { value: 'roles', label: 'Roles' },
] as const;
