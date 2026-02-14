import { http, HttpResponse } from 'msw';
import type { PipelineConfig } from '@/shared/domain/types';

/**
 * Mock pipeline configs matching the Rust PipelineConfig schema exactly.
 * PK = organizationId, SK = customerCompanyId.
 * pipeline = Manifest { version, actions[] }.
 */
const MOCK_CONFIGS: PipelineConfig[] = [
  {
    name: 'Workday Employee Sync',
    image: undefined,
    cron: 'rate(1 day)',
    organizationId: 'org-001',
    customerCompanyId: 'acme-corp',
    lastEdited: '2026-02-10T14:22:00Z',
    pipeline: {
      version: '1.0',
      actions: [
        {
          id: 'ingest',
          actionType: 'workday_hris_connector',
          config: {
            name: 'Workday HCM Fetch',
            endpoint: 'https://wd5-impl.workday.com/ccx/service/acme/Human_Resources/v40.1',
            auth: 'oauth2_client_credentials',
            pageSize: 200,
          },
        },
        {
          id: 'dedup',
          actionType: 'identity_deduplicator',
          config: {
            name: 'Deduplication',
            strategy: 'composite_key',
            keys: ['employeeId', 'email'],
          },
        },
        {
          id: 'mask-pii',
          actionType: 'pii_masking',
          config: {
            name: 'PII Masking',
            columns: [
              { column: 'ssn', strategy: 'full' },
              { column: 'email', strategy: 'partial' },
            ],
          },
        },
        {
          id: 'rename',
          actionType: 'rename_column',
          config: {
            name: 'Field Normalisation',
            renames: { Worker_ID: 'employeeId', Personal_Name: 'legalName' },
          },
        },
      ],
    },
  },
  {
    name: 'BambooHR Onboarding Import',
    image: undefined,
    cron: 'rate(4 hours)',
    organizationId: 'org-001',
    customerCompanyId: 'globex-inc',
    lastEdited: '2026-02-12T11:45:00Z',
    pipeline: {
      version: '1.0',
      actions: [
        {
          id: 'ingest',
          actionType: 'csv_hris_connector',
          config: {
            name: 'BambooHR CSV Fetch',
            csv_path: 's3://onboardyou-landing/bamboohr/latest.csv',
          },
        },
        {
          id: 'filter',
          actionType: 'filter_by_value',
          config: {
            name: 'Active Employees Only',
            column: 'employmentStatus',
            operator: 'eq',
            value: 'active',
          },
        },
        {
          id: 'drop',
          actionType: 'drop_column',
          config: {
            name: 'Drop Internal Fields',
            columns: ['internalNote', 'legacyId'],
          },
        },
      ],
    },
  },
  {
    name: 'SAP SuccessFactors Benefits Sync',
    image: undefined,
    cron: 'cron(0 6 ? * MON *)',
    organizationId: 'org-001',
    customerCompanyId: 'initech-llc',
    lastEdited: '2026-01-30T16:30:00Z',
    pipeline: {
      version: '1.0',
      actions: [
        {
          id: 'ingest',
          actionType: 'csv_hris_connector',
          config: {
            name: 'SAP SFTP Fetch',
            csv_path: 's3://onboardyou-landing/sap/benefits.csv',
          },
        },
        {
          id: 'sanitize-phones',
          actionType: 'cellphone_sanitizer',
          config: {
            name: 'Phone Number Cleanup',
            column: 'phone',
            default_country: 'US',
          },
        },
        {
          id: 'sanitize-countries',
          actionType: 'iso_country_sanitizer',
          config: {
            name: 'Country Code Normalisation',
            column: 'country',
          },
        },
        {
          id: 'diacritics',
          actionType: 'handle_diacritics',
          config: {
            name: 'Diacritics Handling',
            columns: ['firstName', 'lastName'],
          },
        },
        {
          id: 'scd',
          actionType: 'scd_type_2',
          config: {
            name: 'SCD Type 2 History',
            entity_column: 'employeeId',
            date_column: 'effectiveDate',
          },
        },
      ],
    },
  },
];

const API_BASE = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:3000';

export const configHandlers = [
  // GET /config — list all configs (returns PipelineConfig[] directly)
  http.get(`${API_BASE}/config`, () => {
    return HttpResponse.json(MOCK_CONFIGS);
  }),

  // GET /config/:customerCompanyId — get single config (returns PipelineConfig directly)
  http.get(`${API_BASE}/config/:customerCompanyId`, ({ params }) => {
    const config = MOCK_CONFIGS.find((c) => c.customerCompanyId === params.customerCompanyId);
    if (!config) {
      return HttpResponse.json({ error: 'Config not found' }, { status: 404 });
    }
    return HttpResponse.json(config);
  }),

  // POST /config/:customerCompanyId — create config
  http.post(`${API_BASE}/config/:customerCompanyId`, async ({ params, request }) => {
    const body = (await request.json()) as PipelineConfig;
    const saved: PipelineConfig = {
      ...body,
      organizationId: 'org-001',
      customerCompanyId: params.customerCompanyId as string,
      lastEdited: new Date().toISOString(),
    };
    return HttpResponse.json(saved);
  }),

  // PUT /config/:customerCompanyId — update config
  http.put(`${API_BASE}/config/:customerCompanyId`, async ({ params, request }) => {
    const config = MOCK_CONFIGS.find((c) => c.customerCompanyId === params.customerCompanyId);
    if (!config) {
      return HttpResponse.json({ error: 'Config not found' }, { status: 404 });
    }
    const body = (await request.json()) as PipelineConfig;
    const updated: PipelineConfig = {
      ...config,
      ...body,
      organizationId: config.organizationId,
      customerCompanyId: config.customerCompanyId,
      lastEdited: new Date().toISOString(),
    };
    return HttpResponse.json(updated);
  }),

  // POST /config/:customerCompanyId/validate — dry-run validation
  http.post(`${API_BASE}/config/:customerCompanyId/validate`, async ({ request }) => {
    const body = (await request.json()) as PipelineConfig;
    const steps = body.pipeline.actions.map((action) => ({
      actionId: action.id,
      actionType: action.actionType,
      columnsAfter: Object.keys(action.config).filter((k) => k !== 'name'),
    }));
    return HttpResponse.json({
      steps,
      finalColumns: steps.length > 0 ? steps[steps.length - 1].columnsAfter : [],
    });
  }),
];
