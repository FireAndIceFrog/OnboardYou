import { describe, it, expect } from 'vitest';
import reducer, {
  setAuthType,
  updateBearerField,
  updateBearerSchema,
  updateBearerBodyPath,
  updateOAuth2Field,
  updateOAuth2Schema,
  updateOAuth2BodyPath,
  updateRetryField,
  clearSettingsError,
  fetchSettingsThunk,
  saveSettingsThunk,
  toggleShowAdvanced,
  setWizardStep,
  LoadingStatus,
} from './settingsSlice';
import { DEFAULT_EGRESS_SETTINGS } from '../domain/types';

const initialState = {
  settings: DEFAULT_EGRESS_SETTINGS,
  saved: false,
  dirty: false,
  loadingStatus: LoadingStatus.Idle,
  isSaving: false,
  error: null,
  showAdvanced: false,
  wizardStep: 0,
};

describe('settingsSlice', () => {
  it('should return the initial state with sensible defaults', () => {
    const state = reducer(undefined, { type: '@@INIT' });
    expect(state.settings.authType).toBe('bearer');
    expect(state.settings.retryPolicy.maxAttempts).toBe(3);
    expect(state.saved).toBe(false);
    expect(state.dirty).toBe(false);
    expect(state.loadingStatus).toBe(LoadingStatus.Idle);
    expect(state.isSaving).toBe(false);
    expect(state.error).toBeNull();
  });

  it('setAuthType changes auth type and marks dirty', () => {
    const state = reducer(initialState, setAuthType('oauth2'));
    expect(state.settings.authType).toBe('oauth2');
    expect(state.dirty).toBe(true);
    expect(state.saved).toBe(false);
  });

  it('updateBearerField updates a specific bearer field', () => {
    const state = reducer(
      initialState,
      updateBearerField({ field: 'destinationUrl', value: 'https://api.test.com' }),
    );
    expect(state.settings.bearer.destinationUrl).toBe('https://api.test.com');
    expect(state.dirty).toBe(true);
  });

  it('updateBearerSchema replaces the schema object', () => {
    const newSchema = { id: 'string', count: 'number' };
    const state = reducer(initialState, updateBearerSchema(newSchema));
    expect(state.settings.bearer.schema).toEqual(newSchema);
    expect(state.dirty).toBe(true);
  });

  it('updateBearerBodyPath sets the bodyPath string', () => {
    const state = reducer(initialState, updateBearerBodyPath('data.items'));
    expect(state.settings.bearer.bodyPath).toBe('data.items');
    expect(state.dirty).toBe(true);
  });

  it('updateOAuth2Field updates a specific OAuth2 field', () => {
    const state = reducer(
      initialState,
      updateOAuth2Field({ field: 'clientId', value: 'my-client' }),
    );
    expect(state.settings.oauth2.clientId).toBe('my-client');
    expect(state.dirty).toBe(true);
  });

  it('updateOAuth2Schema replaces the schema object', () => {
    const newSchema = { foo: 'string' };
    const state = reducer(initialState, updateOAuth2Schema(newSchema));
    expect(state.settings.oauth2.schema).toEqual(newSchema);
    expect(state.dirty).toBe(true);
  });

  it('updateOAuth2BodyPath sets the bodyPath string', () => {
    const state = reducer(initialState, updateOAuth2BodyPath('payload')); 
    expect(state.settings.oauth2.bodyPath).toBe('payload');
    expect(state.dirty).toBe(true);
  });

  it('updateRetryField updates retry policy', () => {
    const state = reducer(
      initialState,
      updateRetryField({ field: 'maxAttempts', value: 5 }),
    );
    expect(state.settings.retryPolicy.maxAttempts).toBe(5);
    expect(state.dirty).toBe(true);
  });

  it('clearSettingsError clears the error', () => {
    const errorState = { ...initialState, error: 'Something failed' };
    const state = reducer(errorState, clearSettingsError());
    expect(state.error).toBeNull();
  });

  /* ── Show‑advanced & wizard step ───────────────────────── */

  it('toggleShowAdvanced flips the flag', () => {
    const state = reducer(initialState, toggleShowAdvanced());
    expect(state.showAdvanced).toBe(true);
    const state2 = reducer(state, toggleShowAdvanced());
    expect(state2.showAdvanced).toBe(false);
  });

  it('toggling showAdvanced off clamps wizardStep to 1', () => {
    const advanced = { ...initialState, showAdvanced: true, wizardStep: 2 };
    const state = reducer(advanced, toggleShowAdvanced());
    expect(state.showAdvanced).toBe(false);
    expect(state.wizardStep).toBe(1);
  });

  it('setWizardStep sets the step', () => {
    const state = reducer(initialState, setWizardStep(1));
    expect(state.wizardStep).toBe(1);
  });

  /* ── Fetch thunk reducers ──────────────────────────────── */

  it('fetchSettingsThunk.pending sets loading', () => {
    const state = reducer(initialState, fetchSettingsThunk.pending('', {} as never));
    expect(state.loadingStatus).toBe(LoadingStatus.Loading);
    expect(state.error).toBeNull();
  });

  it('fetchSettingsThunk.fulfilled updates settings', () => {
    const loaded = { ...DEFAULT_EGRESS_SETTINGS, authType: 'oauth2' as const };
    const state = reducer(
      { ...initialState, loadingStatus: LoadingStatus.Loading },
      fetchSettingsThunk.fulfilled(loaded, '', {} as never),
    );
    expect(state.loadingStatus).toBe(LoadingStatus.Succeeded);
    expect(state.settings.authType).toBe('oauth2');
    expect(state.dirty).toBe(false);
  });

  it('fetchSettingsThunk.fulfilled with null keeps defaults', () => {
    const state = reducer(
      { ...initialState, loadingStatus: LoadingStatus.Loading },
      fetchSettingsThunk.fulfilled(null, '', {} as never),
    );
    expect(state.loadingStatus).toBe(LoadingStatus.Succeeded);
    expect(state.settings).toEqual(DEFAULT_EGRESS_SETTINGS);
  });

  it('fetchSettingsThunk.rejected sets error', () => {
    const state = reducer(
      { ...initialState, loadingStatus: LoadingStatus.Loading },
      fetchSettingsThunk.rejected(null, '', {} as never, 'Network error'),
    );
    expect(state.loadingStatus).toBe(LoadingStatus.Failed);
    expect(state.error).toBe('Network error');
  });

  /* ── Save thunk reducers ───────────────────────────────── */

  it('saveSettingsThunk.pending sets saving', () => {
    const state = reducer(
      initialState,
      saveSettingsThunk.pending('', {} as never),
    );
    expect(state.isSaving).toBe(true);
    expect(state.error).toBeNull();
  });

  it('saveSettingsThunk.fulfilled marks saved', () => {
    const saved = { ...DEFAULT_EGRESS_SETTINGS, authType: 'oauth2' as const };
    const state = reducer(
      { ...initialState, dirty: true, isSaving: true },
      saveSettingsThunk.fulfilled(saved, '', {} as never),
    );
    expect(state.isSaving).toBe(false);
    expect(state.saved).toBe(true);
    expect(state.dirty).toBe(false);
    expect(state.settings.authType).toBe('oauth2');
  });

  it('saveSettingsThunk.rejected sets error', () => {
    const state = reducer(
      { ...initialState, isSaving: true },
      saveSettingsThunk.rejected(null, '', {} as never, 'Validation error'),
    );
    expect(state.isSaving).toBe(false);
    expect(state.error).toBe('Validation error');
  });
});
