import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit';
import type { AuthState } from '@/features/auth/domain/types';
import type { User } from '@/shared/domain/types';
import { login as loginService, userFromIdToken } from '@/features/auth/services/authService';
import { MOCK_MODE, API_BASE_URL } from '@/shared/domain/constants';
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

/** Check for an existing session on mount. Auto-login in mock mode. */
export const initAuth = createAsyncThunk(
  'auth/initAuth',
  async (_, { dispatch }) => {
    if (MOCK_MODE) {
      dispatch(setUser({ user: MOCK_USER, token: MOCK_TOKEN, refreshToken: null }));
      return;
    }

    // Check for a stored token from a previous login.
    const storedToken = sessionStorage.getItem('oy_access_token');
    const storedIdToken = sessionStorage.getItem('oy_id_token');
    if (storedToken && storedIdToken) {
      try {
        const user = userFromIdToken(storedIdToken, storedToken);
        const refreshToken = sessionStorage.getItem('oy_refresh_token');
        dispatch(setUser({ user, token: storedIdToken, refreshToken }));
        return;
      } catch {
        // Token is corrupt — fall through to logout.
      }
    }

    dispatch(logout());
  },
);

/** Authenticate with email + password via the backend /auth/login route. */
export const performLogin = createAsyncThunk(
  'auth/performLogin',
  async ({ email, password }: { email: string; password: string }, { dispatch }) => {
    if (MOCK_MODE) {
      dispatch(setUser({ user: MOCK_USER, token: MOCK_TOKEN, refreshToken: null }));
      return;
    }

    dispatch(setLoading());

    try {
      const tokens = await loginService(API_BASE_URL, email, password);
      const user = userFromIdToken(tokens.id_token, tokens.access_token);

      // Persist tokens so we can restore the session on refresh.
      sessionStorage.setItem('oy_access_token', tokens.access_token);
      sessionStorage.setItem('oy_id_token', tokens.id_token);
      if (tokens.refresh_token) {
        sessionStorage.setItem('oy_refresh_token', tokens.refresh_token);
      }

      dispatch(
        setUser({
          user,
          token: tokens.id_token,
          refreshToken: tokens.refresh_token ?? null,
        }),
      );
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Authentication failed';
      dispatch(setError(message));
    }
  },
);

/** Clear the session. */
export const performLogout = createAsyncThunk(
  'auth/performLogout',
  async (_, { dispatch }) => {
    sessionStorage.removeItem('oy_access_token');
    sessionStorage.removeItem('oy_id_token');
    sessionStorage.removeItem('oy_refresh_token');
    dispatch(logout());
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
