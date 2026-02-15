import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import type {
  EgressSettings,
  AuthType,
  BearerConfig,
  OAuth2Config,
  RetryPolicy,
} from '../domain/types';
import { DEFAULT_EGRESS_SETTINGS } from '../domain/types';
import type { RootState } from '@/store';

/* ── State type ───────────────────────────────────────────── */

export interface SettingsState {
  settings: EgressSettings;
  saved: boolean;
  dirty: boolean;
}

/* ── Initial state ────────────────────────────────────────── */

const initialState: SettingsState = {
  settings: DEFAULT_EGRESS_SETTINGS,
  saved: false,
  dirty: false,
};

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
    save(state) {
      state.saved = true;
      state.dirty = false;
    },
  },
});

export const {
  setAuthType,
  updateBearerField,
  updateOAuth2Field,
  updateRetryField,
  save,
} = settingsSlice.actions;

/* ── Selectors ────────────────────────────────────────────── */

export const selectSettings = (state: RootState) => state.settings;

export default settingsSlice.reducer;
