import type { ComponentType, SVGProps } from 'react';
import type { ActionType, ActionConfigPayload } from '@/generated/api';
import {
  SearchIcon,
  LockIcon,
  EditIcon,
  TrashIcon,
  TargetIcon,
  PhoneIcon,
  GlobeIcon,
  TypeIcon,
  EraserIcon,
  CalendarIcon,
  RocketIcon,
  TableIcon,
} from '@/shared/ui';

/* ── Catalog entry ─────────────────────────────────────────── */

export interface ActionCatalogEntry {
  actionType: ActionType;
  label: string;
  icon: ComponentType<SVGProps<SVGSVGElement> & { size?: number | string }>;
  description: string;
  category: 'ingestion' | 'logic' | 'egress';
  defaultConfig: ActionConfigPayload;
}

/**
 * Actions available in the "Add Step" panel.
 * Ingestion connectors are excluded — they're set in the connection wizard.
 */
export const ACTION_CATALOG: ActionCatalogEntry[] = [
  /* ── Logic / Transform ───────────────────────────────────── */
  {
    actionType: 'identity_deduplicator',
    label: 'Remove Duplicates',
    icon: SearchIcon,
    description: 'Find and remove duplicate employee records based on matching columns.',
    category: 'logic',
    defaultConfig: { columns: ['email'], employee_id_column: 'employee_id' },
  },
  {
    actionType: 'pii_masking',
    label: 'Mask Sensitive Data',
    icon: LockIcon,
    description: 'Hide personal information like SSNs and salaries before sending data out.',
    category: 'logic',
    defaultConfig: { columns: [{ name: 'ssn', strategy: { Redact: { keep_last: 4, mask_prefix: '***-**-' } } }] },
  },
  {
    actionType: 'rename_column',
    label: 'Rename Fields',
    icon: EditIcon,
    description: 'Change column names to match the format your target system expects.',
    category: 'logic',
    defaultConfig: { mapping: {} },
  },
  {
    actionType: 'drop_column',
    label: 'Remove Unused Fields',
    icon: TrashIcon,
    description: 'Remove columns that aren\'t needed in the output.',
    category: 'logic',
    defaultConfig: { columns: [] },
  },
  {
    actionType: 'filter_by_value',
    label: 'Filter Records',
    icon: TargetIcon,
    description: 'Keep only the records that match a specific value or pattern.',
    category: 'logic',
    defaultConfig: { column: '', pattern: '' },
  },
  {
    actionType: 'cellphone_sanitizer',
    label: 'Clean Phone Numbers',
    icon: PhoneIcon,
    description: 'Convert phone numbers to international format (e.g. +61 4xx xxx xxx).',
    category: 'logic',
    defaultConfig: { phone_column: 'phone', country_columns: ['country'], output_column: 'phone_intl' },
  },
  {
    actionType: 'iso_country_sanitizer',
    label: 'Standardise Country Codes',
    icon: GlobeIcon,
    description: 'Convert country names or codes to standard ISO format (AU, US, GB).',
    category: 'logic',
    defaultConfig: { source_column: 'country', output_column: 'country_iso', output_format: 'alpha2' },
  },
  {
    actionType: 'handle_diacritics',
    label: 'Fix Special Characters',
    icon: TypeIcon,
    description: 'Replace accented characters with their ASCII equivalents (é → e, ü → u).',
    category: 'logic',
    defaultConfig: { columns: [] },
  },
  {
    actionType: 'regex_replace',
    label: 'Clean Up Text',
    icon: EraserIcon,
    description: 'Find and replace text patterns in a column (e.g. remove extra spaces).',
    category: 'logic',
    defaultConfig: { column: '', pattern: '', replacement: '' },
  },
  {
    actionType: 'scd_type_2',
    label: 'Track Change History',
    icon: CalendarIcon,
    description: 'Add effective dating columns to track when employee records change over time.',
    category: 'logic',
    defaultConfig: { entity_column: 'employee_id', date_column: 'start_date' },
  },
  /* ── Egress ──────────────────────────────────────────────── */
  {
    actionType: 'api_dispatcher',
    label: 'Send to API',
    icon: RocketIcon,
    description: 'Send the processed data to an external API endpoint.',
    category: 'egress',
    defaultConfig: { auth_type: 'default' },
  },
  {
    actionType: 'show_data',
    label: 'Show Data',
    icon: TableIcon,
    description: 'Save a snapshot of the current data to S3 and display it in the pipeline view.',
    category: 'egress',
    defaultConfig: {},
  },
];

/* ── Field schemas for the edit panel ──────────────────────── */

export type FieldType = 'text' | 'number' | 'select' | 'columns' | 'column-select' | 'column-multi' | 'mapping' | 'readonly' | 'password';

export interface FieldSchema {
  key: string;
  label: string;
  type: FieldType;
  hint?: string;
  placeholder?: string;
  options?: { value: string; label: string }[];
}

/**
 * Human-friendly field definitions for each action type's config.
 * Used by ActionEditPanel to render proper form inputs instead of raw JSON.
 */
export const ACTION_FIELD_SCHEMAS: Partial<Record<ActionType, FieldSchema[]>> = {
  workday_hris_connector: [
    { key: 'tenant_url', label: 'Tenant URL', type: 'text', hint: 'Workday tenant base URL', placeholder: 'https://wd3-impl-services1.workday.com' },
    { key: 'tenant_id', label: 'Tenant ID', type: 'text', hint: 'Workday tenant identifier', placeholder: 'acme_corp' },
    { key: 'username', label: 'Username', type: 'text', hint: 'Integration System User (ISU)', placeholder: 'ISU_Onboarding' },
    { key: 'password', label: 'Password', type: 'password', hint: 'ISU password (prefix env: to read from env var)', placeholder: 'env:WORKDAY_PASSWORD' },
    { key: 'worker_count_limit', label: 'Worker Count Limit', type: 'number', hint: 'Max workers per page' },
    { key: 'response_group', label: 'Response Groups', type: 'readonly', hint: 'Data sections to include in the Workday response' },
  ],
  sage_hr_connector: [
    { key: 'subdomain', label: 'Subdomain', type: 'text', hint: 'Your Sage HR subdomain (e.g. acme for acme.sage.hr)', placeholder: 'acme' },
    { key: 'api_token', label: 'API Token', type: 'password', hint: 'Sage HR REST API token from Settings → API', placeholder: 'your-api-token' },
  ],
  identity_deduplicator: [
    { key: 'columns', label: 'Match Columns', type: 'column-multi', hint: 'Columns used to identify duplicates (e.g. email, national_id)' },
    { key: 'employee_id_column', label: 'Employee ID Column', type: 'column-select', hint: 'The column with the unique employee identifier' },
  ],
  pii_masking: [
    { key: 'columns', label: 'Columns to Mask', type: 'readonly', hint: 'Sensitive columns — edit below' },
  ],
  rename_column: [
    { key: 'mapping', label: 'Column Mappings', type: 'mapping', hint: 'Map current column names to new names' },
  ],
  drop_column: [
    { key: 'columns', label: 'Columns to Remove', type: 'column-multi', hint: 'Columns to drop from the output' },
  ],
  filter_by_value: [
    { key: 'column', label: 'Filter Column', type: 'column-select', hint: 'Which column to check' },
    { key: 'pattern', label: 'Keep Pattern', type: 'text', hint: 'Only keep records matching this value', placeholder: 'active' },
  ],
  cellphone_sanitizer: [
    { key: 'phone_column', label: 'Phone Column', type: 'column-select', hint: 'Column containing phone numbers' },
    { key: 'country_columns', label: 'Country Columns', type: 'column-multi', hint: 'Columns with country codes for formatting' },
    { key: 'output_column', label: 'Output Column', type: 'text', hint: 'Name for the formatted phone number column', placeholder: 'phone_intl' },
  ],
  iso_country_sanitizer: [
    { key: 'source_column', label: 'Country Column', type: 'column-select', hint: 'Column with country names or codes' },
    { key: 'output_column', label: 'Output Column', type: 'text', hint: 'Name for the standardised ISO code column', placeholder: 'country_iso' },
    { key: 'output_format', label: 'Code Format', type: 'select', hint: 'Which ISO format to use', options: [
      { value: 'alpha2', label: '2-letter code (AU, US, GB)' },
      { value: 'alpha3', label: '3-letter code (AUS, USA, GBR)' },
    ]},
  ],
  handle_diacritics: [
    { key: 'columns', label: 'Columns', type: 'column-multi', hint: 'Columns with names that may have accented characters' },
  ],
  regex_replace: [
    { key: 'column', label: 'Column', type: 'column-select', hint: 'Column to clean' },
    { key: 'pattern', label: 'Find', type: 'text', hint: 'Text or pattern to find', placeholder: '\\s+' },
    { key: 'replacement', label: 'Replace With', type: 'text', hint: 'What to replace it with', placeholder: ' ' },
  ],
  scd_type_2: [
    { key: 'entity_column', label: 'Employee ID Column', type: 'column-select', hint: 'Column that uniquely identifies each employee' },
    { key: 'date_column', label: 'Date Column', type: 'column-select', hint: 'Column with the effective date for versioning' },
  ],
};
