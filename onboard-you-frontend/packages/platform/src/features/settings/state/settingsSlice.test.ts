import { describe, it, expect } from 'vitest';
import reducer, {
  setAuthType,
  updateBearerField,
  updateOAuth2Field,
  updateRetryField,
  save,
} from './settingsSlice';
import { DEFAULT_EGRESS_SETTINGS } from '../domain/types';

const initialState = {
  settings: DEFAULT_EGRESS_SETTINGS,
  saved: false,
  dirty: false,
};

describe('settingsSlice', () => {
  it('should return the initial state with sensible defaults', () => {
    const state = reducer(undefined, { type: '@@INIT' });
    expect(state.settings.authType).toBe('bearer');
    expect(state.settings.retryPolicy.maxAttempts).toBe(3);
    expect(state.saved).toBe(false);
    expect(state.dirty).toBe(false);
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

  it('updateOAuth2Field updates a specific OAuth2 field', () => {
    const state = reducer(
      initialState,
      updateOAuth2Field({ field: 'clientId', value: 'my-client' }),
    );
    expect(state.settings.oauth2.clientId).toBe('my-client');
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

  it('save marks as saved and clears dirty', () => {
    const dirtyState = { ...initialState, dirty: true };
    const state = reducer(dirtyState, save());
    expect(state.saved).toBe(true);
    expect(state.dirty).toBe(false);
  });
});
