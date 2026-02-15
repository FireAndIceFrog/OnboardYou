import { http, HttpResponse } from 'msw';

/** In-memory store for mock settings — survives across requests within a session. */
let storedSettings: Record<string, unknown> | null = null;

const MOCK_SETTINGS = {
  organizationId: 'org-001',
  defaultAuth: {
    auth_type: 'bearer',
    destination_url: 'https://api.example.com/employees',
    token: 'sk-mock-token-abc123',
    placement: 'authorization_header',
    placement_key: 'Authorization',
    extra_headers: {},
    retry_policy: {
      max_attempts: 3,
      initial_backoff_ms: 1000,
      retryable_status_codes: [429, 502, 503, 504],
    },
  },
};

export const settingsHandlers = [
  /* GET /settings */
  http.get('*/settings', () => {
    const data = storedSettings ?? MOCK_SETTINGS;
    return HttpResponse.json(data);
  }),

  /* PUT /settings */
  http.put('*/settings', async ({ request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    storedSettings = {
      organizationId: 'org-001',
      ...body,
    };
    return HttpResponse.json(storedSettings);
  }),
];
