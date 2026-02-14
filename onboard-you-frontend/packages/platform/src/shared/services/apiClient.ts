import type { ApiErrorResponse } from '@/shared/domain/types';
import { API_BASE_URL } from '@/shared/domain/constants';

export class ApiClient {
  private baseUrl: string;
  private getToken: () => string | null;

  constructor(getToken: () => string | null, baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
    this.getToken = getToken;
  }

  private async request<T>(method: string, path: string, body?: unknown): Promise<T> {
    const token = this.getToken();
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const res = await fetch(`${this.baseUrl}${path}`, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    });

    if (!res.ok) {
      const error: ApiErrorResponse = await res.json().catch(() => ({
        statusCode: res.status,
        message: res.statusText,
      }));
      throw error;
    }

    return res.json();
  }

  get<T = unknown>(path: string): Promise<T> {
    return this.request<T>('GET', path);
  }

  post<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('POST', path, body);
  }

  put<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('PUT', path, body);
  }

  patch<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.request<T>('PATCH', path, body);
  }

  del<T = unknown>(path: string): Promise<T> {
    return this.request<T>('DELETE', path);
  }
}
