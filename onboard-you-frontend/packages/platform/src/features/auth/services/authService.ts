import type { User } from '@/shared/domain/types';

interface TokenResponse {
  access_token: string;
  id_token: string;
  refresh_token?: string;
  token_type: string;
  expires_in: number;
}

/**
 * Build the Cognito hosted UI login URL.
 */
export function buildLoginUrl(
  cognitoDomain: string,
  clientId: string,
  redirectUri: string,
): string {
  const params = new URLSearchParams({
    response_type: 'code',
    client_id: clientId,
    redirect_uri: redirectUri,
    scope: 'openid profile email',
  });
  return `${cognitoDomain}/oauth2/authorize?${params.toString()}`;
}

/**
 * Build the Cognito hosted UI logout URL.
 */
export function buildLogoutUrl(
  cognitoDomain: string,
  clientId: string,
  redirectUri: string,
): string {
  const params = new URLSearchParams({
    client_id: clientId,
    logout_uri: redirectUri,
  });
  return `${cognitoDomain}/logout?${params.toString()}`;
}

/**
 * Exchange an authorization code for tokens via the Cognito token endpoint.
 */
export async function exchangeCodeForTokens(
  code: string,
  cognitoDomain: string,
  clientId: string,
  redirectUri: string,
): Promise<TokenResponse> {
  const body = new URLSearchParams({
    grant_type: 'authorization_code',
    client_id: clientId,
    redirect_uri: redirectUri,
    code,
  });

  const res = await fetch(`${cognitoDomain}/oauth2/token`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: body.toString(),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`Token exchange failed: ${text}`);
  }

  return res.json();
}

/**
 * Refresh tokens using a refresh token.
 */
export async function refreshTokens(
  refreshToken: string,
  cognitoDomain: string,
  clientId: string,
): Promise<TokenResponse> {
  const body = new URLSearchParams({
    grant_type: 'refresh_token',
    client_id: clientId,
    refresh_token: refreshToken,
  });

  const res = await fetch(`${cognitoDomain}/oauth2/token`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: body.toString(),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`Token refresh failed: ${text}`);
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
