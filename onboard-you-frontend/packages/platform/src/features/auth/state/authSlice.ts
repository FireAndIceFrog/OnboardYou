import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit';
import type { AuthState } from '@/features/auth/domain/types';
import type { User } from '@/shared/domain/types';
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
import type { RootState } from '@/store';

/* ── Mock data (development only) ─────────────────────────── */

const MOCK_USER: User = {
  id: 'user-001',
  email: 'demo@onboardyou.com',
  name: 'Demo User',
  organizationId: 'org-001',
  role: 'admin',
};

const MOCK_TOKEN = 'mock-jwt-token-for-development';

/* ── Initial state ────────────────────────────────────────── */

const initialState: AuthState = {
  user: null,
  isAuthenticated: false,
  isLoading: true,
  token: null,
  refreshToken: null,
  error: null,
};

/* ── Async thunks ─────────────────────────────────────────── */

/** Auto-login in mock mode, otherwise check for a stored session. */
export const initAuth = createAsyncThunk(
  'auth/initAuth',
  async (_, { dispatch }) => {
    if (MOCK_MODE) {
      dispatch(setUser({ user: MOCK_USER, token: MOCK_TOKEN, refreshToken: null }));
      return;
    }

    // In production, check for stored refresh token
    const storedRefreshToken = sessionStorage.getItem('oy_refresh_token');
    if (!storedRefreshToken) {
      dispatch(logout());
      return;
    }

    // We don't do silent refresh on mount for simplicity — the callback flow
    // handles the initial auth, and the token is held in state.
    dispatch(logout());
  },
);

/** Exchange an authorization code for tokens (Cognito callback). */
export const exchangeCode = createAsyncThunk(
  'auth/exchangeCode',
  async (code: string, { dispatch }) => {
    dispatch(setLoading());

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

      dispatch(
        setUser({
          user,
          token: tokens.access_token,
          refreshToken: tokens.refresh_token ?? null,
        }),
      );
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Authentication failed';
      dispatch(setError(message));
    }
  },
);

/** Redirect to Cognito hosted UI (or auto-login in mock mode). */
export const performLogin = createAsyncThunk(
  'auth/performLogin',
  async (_, { dispatch }) => {
    if (MOCK_MODE) {
      dispatch(setUser({ user: MOCK_USER, token: MOCK_TOKEN, refreshToken: null }));
      return;
    }

    const url = buildLoginUrl(COGNITO_DOMAIN, COGNITO_CLIENT_ID, REDIRECT_URI);
    window.location.href = url;
  },
);

/** Clear session and redirect to Cognito logout URL. */
export const performLogout = createAsyncThunk(
  'auth/performLogout',
  async (_, { dispatch }) => {
    sessionStorage.removeItem('oy_refresh_token');
    dispatch(logout());

    if (!MOCK_MODE && COGNITO_DOMAIN) {
      const url = buildLogoutUrl(COGNITO_DOMAIN, COGNITO_CLIENT_ID, REDIRECT_URI);
      window.location.href = url;
    }
  },
);

/* ── Slice ────────────────────────────────────────────────── */

const authSlice = createSlice({
  name: 'auth',
  initialState,
  reducers: {
    setLoading(state) {
      state.isLoading = true;
      state.error = null;
    },
    setUser(
      state,
      action: PayloadAction<{ user: User; token: string; refreshToken: string | null }>,
    ) {
      state.user = action.payload.user;
      state.token = action.payload.token;
      state.refreshToken = action.payload.refreshToken;
      state.isAuthenticated = true;
      state.isLoading = false;
      state.error = null;
    },
    setError(state, action: PayloadAction<string>) {
      state.user = null;
      state.token = null;
      state.refreshToken = null;
      state.isAuthenticated = false;
      state.isLoading = false;
      state.error = action.payload;
    },
    logout(state) {
      state.user = null;
      state.token = null;
      state.refreshToken = null;
      state.isAuthenticated = false;
      state.isLoading = false;
      state.error = null;
    },
  },
});

export const { setLoading, setUser, setError, logout } = authSlice.actions;

/* ── Selectors ────────────────────────────────────────────── */

export const selectAuth = (state: RootState) => state.auth;
export const selectUser = (state: RootState) => state.auth.user;
export const selectIsAuthenticated = (state: RootState) => state.auth.isAuthenticated;
export const selectIsLoading = (state: RootState) => state.auth.isLoading;

export default authSlice.reducer;
