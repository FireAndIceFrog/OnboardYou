import { http, HttpResponse } from 'msw';

const MOCK_CONFIGS = [
  {
    id: 'cfg-001',
    organizationId: 'org-001',
    name: 'Workday Employee Sync',
    description: 'Synchronises new-hire records from Workday HCM into the OnboardYou identity graph. Runs daily at 02:00 UTC.',
    sourceSystem: 'Workday',
    status: 'active',
    version: 3,
    pipeline: {
      ingestion: {
        type: 'rest_api',
        source: 'Workday HCM',
        config: {
          endpoint: 'https://wd5-impl.workday.com/ccx/service/acme/Human_Resources/v40.1',
          auth: 'oauth2_client_credentials',
          schedule: '0 2 * * *',
          pageSize: 200,
        },
      },
      transformations: [
        {
          id: 'tx-001',
          type: 'field_mapping',
          name: 'Field Normalisation',
          config: { mappings: { 'Worker.Worker_ID': 'employeeId', 'Worker.Personal_Data.Name': 'legalName', 'Worker.Email': 'email' } },
          dependsOn: [],
        },
        {
          id: 'tx-002',
          type: 'deduplication',
          name: 'Deduplication',
          config: { strategy: 'composite_key', keys: ['employeeId', 'email'] },
          dependsOn: ['tx-001'],
        },
        {
          id: 'tx-003',
          type: 'enrichment',
          name: 'Org-Unit Mapping',
          config: { lookupTable: 'org_unit_xref', sourceField: 'department', targetField: 'orgUnitId' },
          dependsOn: ['tx-002'],
        },
      ],
      egress: {
        type: 'dynamodb',
        destination: 'onboardyou-identities',
        config: { tableName: 'onboardyou-identities', partitionKey: 'PK', sortKey: 'SK', writeMode: 'upsert' },
      },
    },
    createdAt: '2025-11-15T10:30:00Z',
    updatedAt: '2026-02-10T14:22:00Z',
  },
  {
    id: 'cfg-002',
    organizationId: 'org-001',
    name: 'BambooHR Onboarding Import',
    description: 'Imports onboarding task-completion data from BambooHR for real-time dashboard reporting.',
    sourceSystem: 'BambooHR',
    status: 'draft',
    version: 1,
    pipeline: {
      ingestion: {
        type: 'rest_api',
        source: 'BambooHR',
        config: { endpoint: 'https://api.bamboohr.com/api/gateway.php/acme/v1', auth: 'api_key', schedule: '0 */4 * * *' },
      },
      transformations: [
        {
          id: 'tx-010',
          type: 'field_mapping',
          name: 'Task Field Mapping',
          config: { mappings: { id: 'taskId', employeeId: 'employeeId', status: 'completionStatus' } },
          dependsOn: [],
        },
        {
          id: 'tx-011',
          type: 'filter',
          name: 'Active Employees Only',
          config: { field: 'employmentStatus', operator: 'eq', value: 'active' },
          dependsOn: ['tx-010'],
        },
      ],
      egress: {
        type: 'dynamodb',
        destination: 'onboardyou-tasks',
        config: { tableName: 'onboardyou-tasks', partitionKey: 'PK', sortKey: 'SK', writeMode: 'upsert' },
      },
    },
    createdAt: '2026-01-20T09:00:00Z',
    updatedAt: '2026-02-12T11:45:00Z',
  },
  {
    id: 'cfg-003',
    organizationId: 'org-001',
    name: 'SAP SuccessFactors Benefits Sync',
    description: 'Pulls benefits enrollment data from SAP SuccessFactors and maps to internal benefits model.',
    sourceSystem: 'SAP SuccessFactors',
    status: 'paused',
    version: 2,
    pipeline: {
      ingestion: {
        type: 'odata',
        source: 'SAP SuccessFactors',
        config: { endpoint: 'https://api.successfactors.com/odata/v2', auth: 'oauth2', schedule: '0 6 * * 1' },
      },
      transformations: [
        {
          id: 'tx-020',
          type: 'field_mapping',
          name: 'Benefits Field Mapping',
          config: { mappings: { userId: 'employeeId', benefitPlan: 'planName', effectiveDate: 'startDate' } },
          dependsOn: [],
        },
      ],
      egress: {
        type: 'dynamodb',
        destination: 'onboardyou-benefits',
        config: { tableName: 'onboardyou-benefits', partitionKey: 'PK', sortKey: 'SK', writeMode: 'upsert' },
      },
    },
    createdAt: '2025-12-01T14:00:00Z',
    updatedAt: '2026-01-30T16:30:00Z',
  },
];

const API_BASE = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:3000';

export const configHandlers = [
  // List all configs
  http.get(`${API_BASE}/configs`, () => {
    return HttpResponse.json({ data: MOCK_CONFIGS, total: MOCK_CONFIGS.length });
  }),

  // Get single config
  http.get(`${API_BASE}/configs/:configId`, ({ params }) => {
    const config = MOCK_CONFIGS.find((c) => c.id === params.configId);
    if (!config) {
      return HttpResponse.json({ error: 'Config not found' }, { status: 404 });
    }
    return HttpResponse.json({ data: config });
  }),

  // Delete config
  http.delete(`${API_BASE}/configs/:configId`, ({ params }) => {
    const idx = MOCK_CONFIGS.findIndex((c) => c.id === params.configId);
    if (idx === -1) {
      return HttpResponse.json({ error: 'Config not found' }, { status: 404 });
    }
    return HttpResponse.json({ success: true });
  }),

  // Create config
  http.post(`${API_BASE}/configs`, async ({ request }) => {
    const body = await request.json() as Record<string, unknown>;
    const newConfig = {
      id: `cfg-${Date.now()}`,
      organizationId: 'org-001',
      status: 'draft',
      version: 1,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      ...body,
    };
    return HttpResponse.json({ data: newConfig }, { status: 201 });
  }),

  // Update config
  http.put(`${API_BASE}/configs/:configId`, async ({ params, request }) => {
    const config = MOCK_CONFIGS.find((c) => c.id === params.configId);
    if (!config) {
      return HttpResponse.json({ error: 'Config not found' }, { status: 404 });
    }
    const body = await request.json() as Record<string, unknown>;
    const updated = { ...config, ...body, updatedAt: new Date().toISOString() };
    return HttpResponse.json({ data: updated });
  }),
];
