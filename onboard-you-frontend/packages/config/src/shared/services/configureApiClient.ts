import { client } from '@/generated/api/client.gen';

/**
 * Called once from setGlobalValue() when platform injects globals.
 * Sets the base URL and wires an auth interceptor that reads the
 * Bearer token from sessionStorage (shared with the platform).
 */
export function configureApiClient(baseUrl: string): void {
  client.setConfig({ baseUrl });

  client.interceptors.request.use((request) => {
    const token = sessionStorage.getItem('oy_id_token');
    if (token) {
      request.headers.set('Authorization', `Bearer ${token}`);
    }
    return request;
  });
}
