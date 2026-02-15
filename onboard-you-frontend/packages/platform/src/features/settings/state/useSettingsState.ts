import { useCallback, useEffect, useRef } from 'react';
import { useAppSelector, useAppDispatch } from '@/store';
import { useGlobal } from '@/shared/hooks';
import {
  setAuthType,
  updateBearerField,
  updateOAuth2Field,
  updateRetryField,
  fetchSettingsThunk,
  saveSettingsThunk,
  clearSettingsError,
} from './settingsSlice';
import type {
  AuthType,
  BearerConfig,
  OAuth2Config,
  RetryPolicy,
} from '../domain/types';

export function useSettingsState() {
  const dispatch = useAppDispatch();
  const { settings, saved, dirty, isLoading, isSaving, error } = useAppSelector(
    (state) => state.settings,
  );
  const { apiClient, showNotification } = useGlobal();

  /* ── Load settings on mount ─────────────────────────────── */
  const fetchedRef = useRef(false);

  useEffect(() => {
    if (fetchedRef.current) return;
    fetchedRef.current = true;
    dispatch(fetchSettingsThunk(apiClient));
  }, [dispatch, apiClient]);

  /* ── Generic updaters ───────────────────────────────────── */
  const updateBearer = useCallback(
    (field: keyof BearerConfig) =>
      (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
        dispatch(updateBearerField({ field, value: e.target.value }));
      },
    [dispatch],
  );

  const updateOAuth2 = useCallback(
    (field: keyof OAuth2Config) =>
      (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
        dispatch(updateOAuth2Field({ field, value: e.target.value }));
      },
    [dispatch],
  );

  const updateRetry = useCallback(
    (field: keyof RetryPolicy) =>
      (e: React.ChangeEvent<HTMLInputElement>) => {
        const val =
          field === 'retryableStatusCodes'
            ? e.target.value
                .split(',')
                .map((s) => parseInt(s.trim(), 10))
                .filter(Boolean)
            : parseInt(e.target.value, 10) || 0;
        dispatch(updateRetryField({ field, value: val }));
      },
    [dispatch],
  );

  const handleAuthTypeChange = useCallback(
    (authType: AuthType) => {
      dispatch(setAuthType(authType));
    },
    [dispatch],
  );

  const handleSave = useCallback(async () => {
    const result = await dispatch(saveSettingsThunk({ apiClient, settings }));
    if (saveSettingsThunk.fulfilled.match(result)) {
      showNotification('Settings saved successfully', 'success');
    } else {
      showNotification(
        (result.payload as string) ?? 'Failed to save settings',
        'error',
      );
    }
  }, [dispatch, apiClient, settings, showNotification]);

  const handleTestConnection = useCallback(() => {
    const url =
      settings.authType === 'bearer'
        ? settings.bearer.destinationUrl
        : settings.oauth2.destinationUrl;
    console.info(`[Settings] Testing connection to ${url}…`);
    alert(`Testing connection to:\n${url || '(no URL configured)'}`);
  }, [settings]);

  const handleClearError = useCallback(() => {
    dispatch(clearSettingsError());
  }, [dispatch]);

  return {
    settings,
    saved,
    dirty,
    isLoading,
    isSaving,
    error,
    updateBearer,
    updateOAuth2,
    updateRetry,
    handleAuthTypeChange,
    handleSave,
    handleTestConnection,
    handleClearError,
  } as const;
}
