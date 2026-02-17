// ── Base API client with built-in auth ──────────────────────
//
// Authenticates once via POST /auth/login, then attaches the
// Bearer token to all subsequent requests.

import type { LoginRequest, LoginResponse } from '../models/auth.js';

export class ApiClient {
  private readonly baseUrl: string;
  private token: string | null = null;
  private readonly email: string;
  private readonly password: string;

  constructor(opts: { baseUrl: string; email: string; password: string }) {
    // Strip trailing slash
    this.baseUrl = opts.baseUrl.replace(/\/+$/, '');
    this.email = opts.email;
    this.password = opts.password;
  }

  // ── Authentication ──────────────────────────────────────────

  /** Authenticate and cache the id_token for subsequent requests. */
  async login(): Promise<LoginResponse> {
    const body: LoginRequest = { email: this.email, password: this.password };

    const res = await fetch(`${this.baseUrl}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });

    if (!res.ok) {
      const text = await res.text();
      throw new Error(`Login failed (${res.status}): ${text}`);
    }

    const tokens: LoginResponse = await res.json();
    this.token = tokens.id_token;
    return tokens;
  }

  // ── Generic helpers ─────────────────────────────────────────

  /** Build headers, injecting the Bearer token when available. */
  private headers(extra: Record<string, string> = {}): Record<string, string> {
    const h: Record<string, string> = { 'Content-Type': 'application/json', ...extra };
    if (this.token) {
      h['Authorization'] = `Bearer ${this.token}`;
    }
    return h;
  }

  /** GET request to a relative path. */
  async get<T>(path: string): Promise<{ status: number; body: T }> {
    const res = await fetch(`${this.baseUrl}${path}`, {
      method: 'GET',
      headers: this.headers(),
    });
    const body = await res.json() as T;
    return { status: res.status, body };
  }

  /** POST request to a relative path. */
  async post<T>(path: string, data: unknown): Promise<{ status: number; body: T }> {
    const res = await fetch(`${this.baseUrl}${path}`, {
      method: 'POST',
      headers: this.headers(),
      body: JSON.stringify(data),
    });
    const body = await res.json() as T;
    return { status: res.status, body };
  }

  /** PUT request to a relative path. */
  async put<T>(path: string, data: unknown): Promise<{ status: number; body: T }> {
    const res = await fetch(`${this.baseUrl}${path}`, {
      method: 'PUT',
      headers: this.headers(),
      body: JSON.stringify(data),
    });
    const body = await res.json() as T;
    return { status: res.status, body };
  }

  /** DELETE request to a relative path. */
  async delete<T>(path: string): Promise<{ status: number; body: T }> {
    const res = await fetch(`${this.baseUrl}${path}`, {
      method: 'DELETE',
      headers: this.headers(),
    });
    const body = await res.json() as T;
    return { status: res.status, body };
  }
}
