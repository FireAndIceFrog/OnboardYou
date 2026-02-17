import { client } from '@/generated/api/client.gen';
import { API_BASE_URL } from '@/shared/domain/constants';

/**
 * One-time setup: configure the generated API client singleton
 * with the base URL and an auth interceptor that attaches the
 * Bearer token from sessionStorage to every outgoing request.
 */
export function configureApiClient(): void {
  client.setConfig({ baseUrl: API_BASE_URL });

  client.interceptors.request.use((request) => {
    const token = sessionStorage.getItem('oy_access_token');
    if (token) {
      request.headers.set('Authorization', `Bearer ${token}`);
    }
    return request;
  });
}
