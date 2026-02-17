import { http, HttpResponse } from 'msw';

const API_BASE = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:3000';

export const authHandlers = [
  // POST /auth/login — return a fake token set
  http.post(`${API_BASE}/auth/login`, async ({ request }) => {
    const body = (await request.json()) as { email?: string; password?: string };

    if (!body.email || !body.password) {
      return HttpResponse.json(
        { error: 'email and password are required' },
        { status: 400 },
      );
    }

    const idToken =
      btoa(JSON.stringify({ alg: 'RS256', typ: 'JWT' })) +
      '.' +
      btoa(
        JSON.stringify({
          sub: 'user-001',
          email: body.email,
          name: 'Demo User',
          'custom:organization_id': 'org-001',
          'custom:role': 'admin',
          exp: Math.floor(Date.now() / 1000) + 3600,
        }),
      ) +
      '.mock-signature';

    return HttpResponse.json({
      id_token: idToken,
      access_token: 'mock-access-token-' + Date.now(),
      refresh_token: 'mock-refresh-token',
      token_type: 'Bearer',
      expires_in: 3600,
    });
  }),
];
