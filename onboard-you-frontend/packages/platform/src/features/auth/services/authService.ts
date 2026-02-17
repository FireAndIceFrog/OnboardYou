import type { User } from '@/shared/domain/types';

export interface LoginTokens {
  id_token: string;
  access_token: string;
  refresh_token?: string;
  token_type: string;
  expires_in: number;
}

/**
 * Authenticate with the backend `/auth/login` endpoint.
 *
 * The backend proxies credentials through to Cognito and returns
 * a token set — the frontend never talks to Cognito directly.
 */
export async function login(
  apiBaseUrl: string,
  email: string,
  password: string,
): Promise<LoginTokens> {
  const res = await fetch(`${apiBaseUrl}/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
  });

  if (!res.ok) {
    const body = await res.json().catch(() => null);
    const message =
      (body as { error?: string } | null)?.error ?? `Login failed (${res.status})`;
    throw new Error(message);
  }

  return res.json();
}

/**
 * Decode a JWT payload without verification (client-side only).
 */
export function decodeJwtPayload(token: string): Record<string, unknown> {
  const parts = token.split('.');
  if (parts.length !== 3) {
    throw new Error('Invalid JWT format');
  }
  const payload = parts[1];
  const decoded = atob(payload.replace(/-/g, '+').replace(/_/g, '/'));
  return JSON.parse(decoded);
}

/**
 * Extract a User object from a Cognito ID token.
 */
export function userFromIdToken(idToken: string): User {
  const claims = decodeJwtPayload(idToken);
  return {
    id: (claims['sub'] as string) ?? '',
    email: (claims['email'] as string) ?? '',
    name: (claims['name'] as string) ?? (claims['cognito:username'] as string) ?? '',
    organizationId: (claims['custom:organization_id'] as string) ?? '',
    role: ((claims['custom:role'] as string) ?? 'viewer') as User['role'],
  };
}
