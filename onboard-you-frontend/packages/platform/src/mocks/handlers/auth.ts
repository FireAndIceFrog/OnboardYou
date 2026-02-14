import { http, HttpResponse } from 'msw';

export const authHandlers = [
  // Mock Cognito token endpoint
  http.post('https://*/oauth2/token', () => {
    return HttpResponse.json({
      access_token: 'mock-access-token-' + Date.now(),
      id_token: btoa(JSON.stringify({ alg: 'RS256', typ: 'JWT' })) + '.' +
        btoa(JSON.stringify({
          sub: 'user-001',
          email: 'demo@onboardyou.com',
          name: 'Demo User',
          'custom:organizationId': 'org-001',
          'custom:role': 'admin',
          exp: Math.floor(Date.now() / 1000) + 3600,
        })) + '.mock-signature',
      refresh_token: 'mock-refresh-token',
      token_type: 'Bearer',
      expires_in: 3600,
    });
  }),

  // Mock user info
  http.get('https://*/oauth2/userInfo', () => {
    return HttpResponse.json({
      sub: 'user-001',
      email: 'demo@onboardyou.com',
      name: 'Demo User',
      'custom:organizationId': 'org-001',
      'custom:role': 'admin',
    });
  }),
];
