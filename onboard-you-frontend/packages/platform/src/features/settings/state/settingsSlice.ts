import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit';
import type {
  EgressSettings,
  AuthType,
  BearerConfig,
  OAuth2Config,
  RetryPolicy,
} from '../domain/types';
import { DEFAULT_EGRESS_SETTINGS } from '../domain/types';
import {
  fetchSettings as fetchSettingsApi,
  saveSettings as saveSettingsApi,
} from '../services/settingsService';
import type { RootState } from '@/store';

/* ── State type ───────────────────────────────────────────── */

export interface SettingsState {
  settings: EgressSettings;
  saved: boolean;
  dirty: boolean;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
}

/* ── Initial state ────────────────────────────────────────── */

const initialState: SettingsState = {
  settings: DEFAULT_EGRESS_SETTINGS,
  saved: false,
  dirty: false,
  isLoading: false,
  isSaving: false,
  error: null,
};

/* ── Async thunks ─────────────────────────────────────────── */

export const fetchSettingsThunk = createAsyncThunk(
  'settings/fetchSettings',
  async (_: void, { rejectWithValue }) => {
    try {
      return await fetchSettingsApi();
    } catch (err: unknown) {
      /* 404 means no settings saved yet — use defaults */
      if (typeof err === 'object' && err !== null && 'statusCode' in err) {
        const apiErr = err as { statusCode: number; message: string };
        if (apiErr.statusCode === 404) return null;
        return rejectWithValue(apiErr.message);
      }
      const message =
        err instanceof Error ? err.message : 'Failed to load settings';
      return rejectWithValue(message);
    }
  },
);

export const saveSettingsThunk = createAsyncThunk(
  'settings/saveSettings',
  async (
    { settings }: { settings: EgressSettings },
    { rejectWithValue },
  ) => {
    try {
      return await saveSettingsApi(settings);
    } catch (err: unknown) {
      if (typeof err === 'object' && err !== null && 'statusCode' in err && 'message' in err) {
        return rejectWithValue(err.message);
      }
      const message =
        err instanceof Error ? err.message : 'Failed to save settings';
      return rejectWithValue(message);
    }
  },
);

/* ── Slice ────────────────────────────────────────────────── */

const settingsSlice = createSlice({
  name: 'settings',
  initialState,
  reducers: {
    setAuthType(state, action: PayloadAction<AuthType>) {
      state.settings.authType = action.payload;
      state.dirty = true;
      state.saved = false;
    },
    updateBearerField(
      state,
      action: PayloadAction<{ field: keyof BearerConfig; value: string }>,
    ) {
      (state.settings.bearer[action.payload.field] as string) = action.payload.value;
      state.dirty = true;
      state.saved = false;
    },
    updateOAuth2Field(
      state,
      action: PayloadAction<{ field: keyof OAuth2Config; value: string }>,
    ) {
      (state.settings.oauth2[action.payload.field] as string) = action.payload.value;
      state.dirty = true;
      state.saved = false;
    },
    updateRetryField(
      state,
      action: PayloadAction<{ field: keyof RetryPolicy; value: number | number[] }>,
    ) {
      (state.settings.retryPolicy[action.payload.field] as number | number[]) =
        action.payload.value;
      state.dirty = true;
      state.saved = false;
    },
    clearSettingsError(state) {
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    builder
      /* ── fetch ──────────────────────────────────────────── */
      .addCase(fetchSettingsThunk.pending, (state) => {
        state.isLoading = true;
        state.error = null;
      })
      .addCase(fetchSettingsThunk.fulfilled, (state, action) => {
        state.isLoading = false;
        if (action.payload) {
          state.settings = action.payload;
        }
        state.dirty = false;
        state.saved = false;
      })
      .addCase(fetchSettingsThunk.rejected, (state, action) => {
        state.isLoading = false;
        state.error = (action.payload as string) ?? 'Failed to load settings';
      })
      /* ── save ───────────────────────────────────────────── */
      .addCase(saveSettingsThunk.pending, (state) => {
        state.isSaving = true;
        state.error = null;
      })
      .addCase(saveSettingsThunk.fulfilled, (state, action) => {
        state.isSaving = false;
        state.settings = action.payload;
        state.saved = true;
        state.dirty = false;
      })
      .addCase(saveSettingsThunk.rejected, (state, action) => {
        state.isSaving = false;
        state.error = (action.payload as string) ?? 'Failed to save settings';
      });
  },
});

export const {
  setAuthType,
  updateBearerField,
  updateOAuth2Field,
  updateRetryField,
  clearSettingsError,
} = settingsSlice.actions;

/* ── Selectors ────────────────────────────────────────────── */

export const selectSettings = (state: RootState) => state.settings;
export const selectSettingsLoading = (state: RootState) => state.settings.isLoading;
export const selectSettingsSaving = (state: RootState) => state.settings.isSaving;
export const selectSettingsError = (state: RootState) => state.settings.error;

export default settingsSlice.reducer;
