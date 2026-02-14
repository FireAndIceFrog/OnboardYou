// ============================================================================
// OnboardYou — Auth Provider
//
// Manages authentication state using Cognito Hosted UI (redirect-based OAuth).
// Tokens are stored **in memory only** — never in localStorage — for security.
// Uses plain `fetch` against the Cognito token endpoint instead of Amplify SDK.
// ============================================================================

import {
  useState,
  useEffect,
  useCallback,
  useMemo,
  type ReactNode,
} from 'react';
import { AuthContext, type AuthContextValue } from './AuthContext';
import type { User, UserRole } from '@/types';

// ---------------------------------------------------------------------------
// Cognito configuration (from environment)
// ---------------------------------------------------------------------------

const COGNITO_DOMAIN = import.meta.env.VITE_COGNITO_DOMAIN as string; // e.g. myapp.auth.us-east-1.amazoncognito.com
const CLIENT_ID = import.meta.env.VITE_COGNITO_CLIENT_ID as string;
const REDIRECT_URI = import.meta.env.VITE_REDIRECT_URI as string;

// ---------------------------------------------------------------------------
// In-memory token store
// ---------------------------------------------------------------------------

let _accessToken: string | null = null;
let _idToken: string | null = null;
let _refreshToken: string | null = null;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Decode a JWT payload (no verification — that's the backend's job). */
function decodeJwtPayload(token: string): Record<string, unknown> {
  const base64Url = token.split('.')[1];
  const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
  const json = decodeURIComponent(
    atob(base64)
      .split('')
      .map((c) => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
      .join(''),
  );
  return JSON.parse(json);
}

/** Map Cognito ID-token claims to our User type. */
function userFromIdToken(token: string): User {
  const claims = decodeJwtPayload(token);
  return {
    id: claims.sub as string,
    email: claims.email as string,
    name: (claims.name as string) || (claims.email as string),
    organizationId: (claims['custom:organizationId'] as string) || '',
    role: ((claims['custom:role'] as string) || 'member') as UserRole,
  };
}

/** Check whether a JWT is expired (with a 60-second margin). */
function isTokenExpired(token: string): boolean {
  try {
    const { exp } = decodeJwtPayload(token) as { exp: number };
    return Date.now() >= exp * 1000 - 60_000;
  } catch {
    return true;
  }
}

/** POST to the Cognito `/oauth2/token` endpoint. */
async function fetchTokens(code: string) {
  const tokenUrl = `https://${COGNITO_DOMAIN}/oauth2/token`;

  const body = new URLSearchParams({
    grant_type: 'authorization_code',
    client_id: CLIENT_ID,
    redirect_uri: REDIRECT_URI,
    code,
  });

  const res = await fetch(tokenUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body,
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`Token exchange failed (${res.status}): ${text}`);
  }

  return (await res.json()) as {
    access_token: string;
    id_token: string;
    refresh_token?: string;
  };
}

/** Attempt to refresh the session using the stored refresh token. */
async function refreshSession(): Promise<boolean> {
  if (!_refreshToken) return false;

  try {
    const tokenUrl = `https://${COGNITO_DOMAIN}/oauth2/token`;

    const body = new URLSearchParams({
      grant_type: 'refresh_token',
      client_id: CLIENT_ID,
      refresh_token: _refreshToken,
    });

    const res = await fetch(tokenUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body,
    });

    if (!res.ok) return false;

    const data = (await res.json()) as {
      access_token: string;
      id_token: string;
    };

    _accessToken = data.access_token;
    _idToken = data.id_token;
    return true;
  } catch {
    return false;
  }
}

function clearTokens() {
  _accessToken = null;
  _idToken = null;
  _refreshToken = null;
}

// ---------------------------------------------------------------------------
// Provider Component
// ---------------------------------------------------------------------------

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  // On mount: attempt to restore session from in-memory tokens.
  useEffect(() => {
    let cancelled = false;

    async function restore() {
      if (_idToken && !isTokenExpired(_idToken)) {
        try {
          const parsed = userFromIdToken(_idToken);
          if (!cancelled) {
            setUser(parsed);
            setIsAuthenticated(true);
          }
        } catch {
          clearTokens();
        }
      } else if (_refreshToken) {
        const ok = await refreshSession();
        if (ok && _idToken && !cancelled) {
          try {
            const parsed = userFromIdToken(_idToken);
            setUser(parsed);
            setIsAuthenticated(true);
          } catch {
            clearTokens();
          }
        }
      }
      if (!cancelled) setIsLoading(false);
    }

    restore();
    return () => {
      cancelled = true;
    };
  }, []);

  // ------ Actions ------

  const login = useCallback(() => {
    const authUrl = new URL(`https://${COGNITO_DOMAIN}/oauth2/authorize`);
    authUrl.searchParams.set('response_type', 'code');
    authUrl.searchParams.set('client_id', CLIENT_ID);
    authUrl.searchParams.set('redirect_uri', REDIRECT_URI);
    authUrl.searchParams.set('scope', 'openid email profile');
    window.location.href = authUrl.toString();
  }, []);

  const logout = useCallback(() => {
    clearTokens();
    setUser(null);
    setIsAuthenticated(false);

    const logoutUrl = new URL(`https://${COGNITO_DOMAIN}/logout`);
    logoutUrl.searchParams.set('client_id', CLIENT_ID);
    logoutUrl.searchParams.set('logout_uri', REDIRECT_URI);
    window.location.href = logoutUrl.toString();
  }, []);

  const getToken = useCallback((): string | null => _accessToken, []);

  const exchangeCode = useCallback(async (code: string) => {
    setIsLoading(true);
    try {
      const tokens = await fetchTokens(code);
      _accessToken = tokens.access_token;
      _idToken = tokens.id_token;
      _refreshToken = tokens.refresh_token ?? null;

      const parsed = userFromIdToken(tokens.id_token);
      setUser(parsed);
      setIsAuthenticated(true);
    } catch (err) {
      clearTokens();
      setUser(null);
      setIsAuthenticated(false);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  // ------ Context value ------

  const value = useMemo<AuthContextValue>(
    () => ({
      user,
      isAuthenticated,
      isLoading,
      login,
      logout,
      getToken,
      exchangeCode,
    }),
    [user, isAuthenticated, isLoading, login, logout, getToken, exchangeCode],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
