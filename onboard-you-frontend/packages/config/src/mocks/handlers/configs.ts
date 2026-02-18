import { http, HttpResponse } from 'msw';
import type {
  PipelineConfig,
  ActionConfigPayload,
  WorkdayConfig,
  DedupConfig,
  PiiMaskingConfig,
  RenameConfig,
  CsvHrisConnectorConfig,
  FilterByValueConfig,
  DropConfig,
  CellphoneSanitizerConfig,
  IsoCountrySanitizerConfig,
  HandleDiacriticsConfig,
  ScdType2Config,
  PresignedUploadResponse,
  CsvColumnsResponse,
} from '@/generated/api';

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
          action_type: 'workday_hris_connector',
          config: {
            tenant_url: 'https://wd5-impl.workday.com/ccx/service/acme/Human_Resources/v40.1',
            tenant_id: 'acme_corp',
            username: 'ISU_Onboarding',
            password: 'env:WORKDAY_PASSWORD',
            worker_count_limit: 200,
            response_group: {
              include_personal_information: true,
              include_employment_information: true,
              include_compensation: false,
              include_organizations: false,
              include_roles: false,
            },
          } satisfies WorkdayConfig as ActionConfigPayload,
        },
        {
          id: 'dedup',
          action_type: 'identity_deduplicator',
          config: {
            columns: ['national_id', 'email'],
            employee_id_column: 'employee_id',
          } satisfies DedupConfig as ActionConfigPayload,
        },
        {
          id: 'mask-pii',
          action_type: 'pii_masking',
          config: {
            columns: [
              { name: 'ssn', strategy: { Redact: { keep_last: 4, mask_prefix: '***-**-' } } },
              { name: 'salary', strategy: 'Zero' },
            ],
          } satisfies PiiMaskingConfig as ActionConfigPayload,
        },
        {
          id: 'rename',
          action_type: 'rename_column',
          config: {
            mapping: { Worker_ID: 'employeeId', Personal_Name: 'legalName' },
          } satisfies RenameConfig as ActionConfigPayload,
        },
        {
          id: 'dispatch',
          action_type: 'api_dispatcher',
          config: { Bearer: { destination_url: 'https://api.acme.com/employees', token: null } } as ActionConfigPayload,
        },
      ],
    },
  },
  {
    name: 'CSV Onboarding Import',
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
          action_type: 'csv_hris_connector',
          config: {
            filename: 'latest.csv',
            columns: ['employee_id', 'first_name', 'last_name', 'email', 'employmentStatus', 'internalNote', 'legacyId'],
          } satisfies CsvHrisConnectorConfig as ActionConfigPayload,
        },
        {
          id: 'filter',
          action_type: 'filter_by_value',
          config: {
            column: 'employmentStatus',
            pattern: '^active$',
          } satisfies FilterByValueConfig as ActionConfigPayload,
        },
        {
          id: 'drop',
          action_type: 'drop_column',
          config: {
            columns: ['internalNote', 'legacyId'],
          } satisfies DropConfig as ActionConfigPayload,
        },
        {
          id: 'dispatch',
          action_type: 'api_dispatcher',
          config: {
            Bearer: {
              destination_url: 'https://api.globex.com/v1/employees',
              token: 'sk-globex-custom-token',
              placement: 'authorization_header',
            },
          } as ActionConfigPayload,
        },
      ],
    },
  },
  {
    name: 'Workday Benefits Weekly Sync',
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
          action_type: 'workday_hris_connector',
          config: {
            tenant_url: 'https://wd5-impl.workday.com/ccx/service/initech/Human_Resources/v40.1',
            tenant_id: 'initech_llc',
            username: 'ISU_Benefits',
            password: 'env:WORKDAY_BENEFITS_PASSWORD',
            worker_count_limit: 100,
            response_group: {
              include_personal_information: true,
              include_employment_information: true,
              include_compensation: true,
              include_organizations: true,
              include_roles: false,
            },
          } satisfies WorkdayConfig as ActionConfigPayload,
        },
        {
          id: 'sanitize-phones',
          action_type: 'cellphone_sanitizer',
          config: {
            phone_column: 'phone',
            country_columns: ['country'],
            output_column: 'phone_intl',
          } satisfies CellphoneSanitizerConfig as ActionConfigPayload,
        },
        {
          id: 'sanitize-countries',
          action_type: 'iso_country_sanitizer',
          config: {
            source_column: 'country',
            output_column: 'country_iso',
            output_format: 'alpha2',
          } satisfies IsoCountrySanitizerConfig as ActionConfigPayload,
        },
        {
          id: 'diacritics',
          action_type: 'handle_diacritics',
          config: {
            columns: ['firstName', 'lastName'],
          } satisfies HandleDiacriticsConfig as ActionConfigPayload,
        },
        {
          id: 'scd',
          action_type: 'scd_type_2',
          config: {
            entity_column: 'employeeId',
            date_column: 'effectiveDate',
          } satisfies ScdType2Config as ActionConfigPayload,
        },
        {
          id: 'dispatch',
          action_type: 'api_dispatcher',
          config: {
            OAuth2: {
              destination_url: 'https://api.initech.com/v2/benefits',
              client_id: 'initech-app-001',
              client_secret: 'initech-secret',
              token_url: 'https://auth.initech.com/oauth/token',
              scopes: ['benefits.write'],
              grant_type: 'client_credentials',
            },
          } as ActionConfigPayload,
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

  // DELETE /config/:customerCompanyId — delete config
  http.delete(`${API_BASE}/config/:customerCompanyId`, ({ params }) => {
    const idx = MOCK_CONFIGS.findIndex((c) => c.customerCompanyId === params.customerCompanyId);
    if (idx === -1) {
      return HttpResponse.json({ error: 'Config not found' }, { status: 404 });
    }
    MOCK_CONFIGS.splice(idx, 1);
    return new HttpResponse(null, { status: 204 });
  }),

  // POST /config/:customerCompanyId/validate — dry-run validation
  http.post(`${API_BASE}/config/:customerCompanyId/validate`, async ({ request }) => {
    const body = (await request.json()) as PipelineConfig;
    const steps = body.pipeline.actions.map((action) => ({
      action_id: action.id,
      action_type: action.action_type,
      columns_after: Object.keys(action.config).filter((k) => k !== 'id'),
    }));
    return HttpResponse.json({
      steps,
      final_columns: steps.length > 0 ? steps[steps.length - 1].columns_after : [],
    });
  }),

  // POST /config/:customerCompanyId/csv-upload — presigned upload URL
  http.post(`${API_BASE}/config/:customerCompanyId/csv-upload`, ({ params, request }) => {
    const url = new URL(request.url);
    const filename = url.searchParams.get('filename') ?? 'upload.csv';
    const companyId = params.customerCompanyId as string;
    const key = `org-001/${companyId}/${filename}`;
    return HttpResponse.json({
      filename,
      key,
      upload_url: `https://mock-s3.localhost/${key}?X-Amz-Signature=mock`,
    } satisfies PresignedUploadResponse);
  }),

  // GET /config/:customerCompanyId/csv-columns — discover CSV headers
  http.get(`${API_BASE}/config/:customerCompanyId/csv-columns`, ({ request }) => {
    const url = new URL(request.url);
    const filename = url.searchParams.get('filename') ?? 'upload.csv';
    // Return realistic mock columns
    return HttpResponse.json({
      filename,
      columns: ['employee_id', 'first_name', 'last_name', 'email', 'department', 'hire_date', 'salary'],
    } satisfies CsvColumnsResponse);
  }),
];
