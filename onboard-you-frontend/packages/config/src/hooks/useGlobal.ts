import type { PipelineConfig } from '@/types';

// TODO: Replace this local mock with the real platform import:
// import { useGlobal } from '@onboard-you/platform/hooks';

const MOCK_CONFIGS: PipelineConfig[] = [
  {
    id: 'cfg-001',
    organizationId: 'org-1',
    name: 'Workday Employee Sync',
    description:
      'Synchronises new-hire records from Workday HCM into the OnboardYou identity graph. Runs daily at 02:00 UTC, pulling employee profiles, job details, and manager hierarchy. Transforms are applied for field normalisation, deduplication, and org-unit mapping before loading into DynamoDB.',
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
          config: {
            mappings: {
              'Worker.Worker_ID': 'employeeId',
              'Worker.Personal_Data.Name.Legal_Name': 'legalName',
              'Worker.Personal_Data.Contact_Data.Email_Address': 'email',
              'Worker.Employment_Data.Job_Data.Position_Title': 'jobTitle',
              'Worker.Employment_Data.Job_Data.Department': 'department',
            },
          },
          dependsOn: [],
        },
        {
          id: 'tx-002',
          type: 'deduplication',
          name: 'Deduplication',
          config: {
            strategy: 'composite_key',
            keys: ['employeeId', 'email'],
            conflictResolution: 'latest_timestamp',
          },
          dependsOn: ['tx-001'],
        },
        {
          id: 'tx-003',
          type: 'enrichment',
          name: 'Org-Unit Mapping',
          config: {
            lookupTable: 'org_unit_xref',
            sourceField: 'department',
            targetField: 'orgUnitId',
            fallback: 'UNKNOWN',
          },
          dependsOn: ['tx-002'],
        },
      ],
      egress: {
        type: 'dynamodb',
        destination: 'onboardyou-identities',
        config: {
          tableName: 'onboardyou-identities',
          partitionKey: 'PK',
          sortKey: 'SK',
          writeMode: 'upsert',
        },
      },
    },
    createdAt: '2025-11-15T10:30:00Z',
    updatedAt: '2026-02-10T14:22:00Z',
  },
  {
    id: 'cfg-002',
    organizationId: 'org-1',
    name: 'BambooHR Onboarding Import',
    description:
      'Imports onboarding task-completion data from BambooHR. Captures checklist progress, document signatures, and training completions, then maps them to the OnboardYou task model for real-time dashboard reporting.',
    sourceSystem: 'BambooHR',
    status: 'draft',
    version: 1,
    pipeline: {
      ingestion: {
        type: 'rest_api',
        source: 'BambooHR',
        config: {
          endpoint: 'https://api.bamboohr.com/api/gateway.php/acme/v1',
          auth: 'api_key',
          schedule: '0 */4 * * *',
          pageSize: 100,
        },
      },
      transformations: [
        {
          id: 'tx-010',
          type: 'field_mapping',
          name: 'Task Field Mapping',
          config: {
            mappings: {
              id: 'taskId',
              employeeId: 'employeeId',
              category: 'taskCategory',
              status: 'completionStatus',
              completedDate: 'completedAt',
            },
          },
          dependsOn: [],
        },
        {
          id: 'tx-011',
          type: 'filter',
          name: 'Active Employees Only',
          config: {
            condition: 'record.status !== "terminated"',
            dropUnmatched: true,
          },
          dependsOn: ['tx-010'],
        },
      ],
      egress: {
        type: 'dynamodb',
        destination: 'onboardyou-tasks',
        config: {
          tableName: 'onboardyou-tasks',
          partitionKey: 'PK',
          sortKey: 'SK',
          writeMode: 'upsert',
        },
      },
    },
    createdAt: '2026-01-20T09:00:00Z',
    updatedAt: '2026-02-12T11:45:00Z',
  },
  {
    id: 'cfg-003',
    organizationId: 'org-1',
    name: 'ADP Payroll Verification',
    description:
      'Pulls payroll-enrolment confirmation from ADP Workforce Now to verify that new hires have been correctly set up in payroll. Flags discrepancies between OnboardYou records and ADP for HR review.',
    sourceSystem: 'ADP',
    status: 'paused',
    version: 2,
    pipeline: {
      ingestion: {
        type: 'sftp',
        source: 'ADP Workforce Now',
        config: {
          host: 'sftp.adp.com',
          path: '/outbound/payroll_enrolment/',
          filePattern: 'enrolment_*.csv',
          schedule: '0 6 * * 1',
        },
      },
      transformations: [
        {
          id: 'tx-020',
          type: 'csv_parse',
          name: 'CSV Parsing',
          config: {
            delimiter: ',',
            hasHeader: true,
            encoding: 'utf-8',
          },
          dependsOn: [],
        },
        {
          id: 'tx-021',
          type: 'field_mapping',
          name: 'Payroll Field Mapping',
          config: {
            mappings: {
              EMP_ID: 'employeeId',
              PAY_STATUS: 'payrollStatus',
              ENROL_DATE: 'enrolmentDate',
              PAY_FREQ: 'payFrequency',
            },
          },
          dependsOn: ['tx-020'],
        },
        {
          id: 'tx-022',
          type: 'validation',
          name: 'Discrepancy Check',
          config: {
            rules: [
              { field: 'employeeId', rule: 'exists_in', target: 'onboardyou-identities' },
              { field: 'payrollStatus', rule: 'equals', value: 'active' },
            ],
            onFailure: 'flag_for_review',
          },
          dependsOn: ['tx-021'],
        },
      ],
      egress: {
        type: 'dynamodb',
        destination: 'onboardyou-payroll-verifications',
        config: {
          tableName: 'onboardyou-payroll-verifications',
          partitionKey: 'PK',
          sortKey: 'SK',
          writeMode: 'put',
        },
      },
    },
    createdAt: '2025-12-01T16:00:00Z',
    updatedAt: '2026-01-28T08:30:00Z',
  },
];

interface MockUser {
  id: string;
  email: string;
  name: string;
  organizationId: string;
  role: string;
}

interface MockAuth {
  user: MockUser;
  isAuthenticated: boolean;
  token: string;
}

interface MockOrganization {
  id: string;
  name: string;
  plan: string;
}

interface ApiClient {
  get: <T = unknown>(path: string) => Promise<T>;
  post: <T = unknown>(path: string, body?: unknown) => Promise<T>;
  put: <T = unknown>(path: string, body?: unknown) => Promise<T>;
  delete: <T = unknown>(path: string) => Promise<T>;
}

interface GlobalState {
  auth: MockAuth;
  organization: MockOrganization;
  apiClient: ApiClient;
  theme: 'light' | 'dark';
  showNotification: (message: string, type?: 'success' | 'error' | 'info' | 'warning') => void;
}

function createMockApiClient(): ApiClient {
  const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

  return {
    get: async <T = unknown>(path: string): Promise<T> => {
      await delay(400);

      if (path === '/configs') {
        return [...MOCK_CONFIGS] as T;
      }

      const match = path.match(/^\/configs\/(.+)$/);
      if (match) {
        const config = MOCK_CONFIGS.find((c) => c.id === match[1]);
        if (config) return { ...config } as T;
        throw new Error(`Config not found: ${match[1]}`);
      }

      throw new Error(`Mock API: unhandled GET ${path}`);
    },

    post: async <T = unknown>(path: string, body?: unknown): Promise<T> => {
      await delay(300);
      console.log('[mock] POST', path, body);
      return { success: true } as T;
    },

    put: async <T = unknown>(path: string, body?: unknown): Promise<T> => {
      await delay(300);
      console.log('[mock] PUT', path, body);
      return { success: true } as T;
    },

    delete: async <T = unknown>(path: string): Promise<T> => {
      await delay(200);
      console.log('[mock] DELETE', path);
      return { success: true } as T;
    },
  };
}

const mockApiClient = createMockApiClient();

export function useGlobal(): GlobalState {
  return {
    auth: {
      user: {
        id: '1',
        email: 'demo@onboardyou.com',
        name: 'Demo User',
        organizationId: 'org-1',
        role: 'admin',
      },
      isAuthenticated: true,
      token: 'mock-token',
    },
    organization: {
      id: 'org-1',
      name: 'Acme Corp',
      plan: 'enterprise',
    },
    apiClient: mockApiClient,
    theme: 'light',
    showNotification: (message: string, type: string = 'info') => {
      console.log(`[notification:${type}] ${message}`);
    },
  };
}
