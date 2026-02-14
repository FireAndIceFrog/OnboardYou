import { useReducer, useEffect, useCallback, type ReactNode } from 'react';
import { AuthContext } from './AuthContext';
import { authReducer, initialAuthState } from './authReducer';
import {
  buildLoginUrl,
  buildLogoutUrl,
  exchangeCodeForTokens,
  userFromIdToken,
} from '@/features/auth/services/authService';
import {
  COGNITO_DOMAIN,
  COGNITO_CLIENT_ID,
  REDIRECT_URI,
} from '@/features/auth/domain/constants';
import { MOCK_MODE } from '@/shared/domain/constants';
import type { User } from '@/shared/domain/types';

interface AuthProviderProps {
  children: ReactNode;
}

const MOCK_USER: User = {
  id: 'user-001',
  email: 'demo@onboardyou.com',
  name: 'Demo User',
  organizationId: 'org-001',
  role: 'admin',
};

const MOCK_TOKEN = 'mock-jwt-token-for-development';

export function AuthProvider({ children }: AuthProviderProps) {
  const [state, dispatch] = useReducer(authReducer, initialAuthState);

  // In MOCK_MODE, auto-login on mount
  useEffect(() => {
    if (MOCK_MODE) {
      dispatch({
        type: 'AUTH_SUCCESS',
        payload: { user: MOCK_USER, token: MOCK_TOKEN, refreshToken: null },
      });
      return;
    }

    // In production, check for stored refresh token
    const storedRefreshToken = sessionStorage.getItem('oy_refresh_token');
    if (!storedRefreshToken) {
      dispatch({ type: 'AUTH_LOGOUT' });
      return;
    }

    // We don't do silent refresh on mount for simplicity — the callback flow
    // handles the initial auth, and the token is held in state.
    dispatch({ type: 'AUTH_LOGOUT' });
  }, []);

  const login = useCallback(() => {
    if (MOCK_MODE) {
      dispatch({
        type: 'AUTH_SUCCESS',
        payload: { user: MOCK_USER, token: MOCK_TOKEN, refreshToken: null },
      });
      return;
    }

    const url = buildLoginUrl(COGNITO_DOMAIN, COGNITO_CLIENT_ID, REDIRECT_URI);
    window.location.href = url;
  }, []);

  const logout = useCallback(() => {
    sessionStorage.removeItem('oy_refresh_token');
    dispatch({ type: 'AUTH_LOGOUT' });

    if (!MOCK_MODE && COGNITO_DOMAIN) {
      const url = buildLogoutUrl(COGNITO_DOMAIN, COGNITO_CLIENT_ID, REDIRECT_URI);
      window.location.href = url;
    }
  }, []);

  const getToken = useCallback(() => {
    return state.token;
  }, [state.token]);

  const exchangeCode = useCallback(async (code: string) => {
    dispatch({ type: 'AUTH_LOADING' });

    try {
      const tokens = await exchangeCodeForTokens(
        code,
        COGNITO_DOMAIN,
        COGNITO_CLIENT_ID,
        REDIRECT_URI,
      );

      const user = userFromIdToken(tokens.id_token);
      if (tokens.refresh_token) {
        sessionStorage.setItem('oy_refresh_token', tokens.refresh_token);
      }

      dispatch({
        type: 'AUTH_SUCCESS',
        payload: {
          user,
          token: tokens.access_token,
          refreshToken: tokens.refresh_token ?? null,
        },
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Authentication failed';
      dispatch({ type: 'AUTH_ERROR', payload: message });
    }
  }, []);

  return (
    <AuthContext.Provider value={{ state, login, logout, getToken, exchangeCode }}>
      {children}
    </AuthContext.Provider>
  );
}
