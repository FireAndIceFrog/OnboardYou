import { useCallback, useEffect } from 'react';
import { useAppSelector, useAppDispatch } from '@/store';
import { useGlobal } from '@/shared/hooks';
import {
  setAuthType,
  updateBearerField,
  updateBearerSchema as updateBearerSchemaAction,
  updateBearerBodyPath as updateBearerBodyPathAction,
  updateOAuth2Field,
  updateOAuth2Schema as updateOAuth2SchemaAction,
  updateOAuth2BodyPath as updateOAuth2BodyPathAction,
  updateRetryField,
  fetchSettingsThunk,
  saveSettingsThunk,
  clearSettingsError,
  toggleShowAdvanced,
} from './settingsSlice';
import type {
  AuthType,
  BearerConfig,
  OAuth2Config,
  RetryPolicy,
} from '../domain/types';

export function useSettingsState() {
  const dispatch = useAppDispatch();
  const { settings, saved, dirty, loadingStatus, isSaving, error, showAdvanced } = useAppSelector(
    (state) => state.settings,
  );
  const { showNotification } = useGlobal();

  /* ── Generic updaters ───────────────────────────────────── */
  const updateBearer = useCallback(
    (field: keyof BearerConfig) =>
      (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
        dispatch(updateBearerField({ field, value: e.target.value }));
      },
    [dispatch],
  );

  const updateBearerSchema = useCallback(
    (schema: Record<string, string>) => {
      dispatch(updateBearerSchemaAction(schema));
    },
    [dispatch],
  );

  const updateBearerBodyPath = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      dispatch(updateBearerBodyPathAction(e.target.value));
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

  const updateOAuth2Schema = useCallback(
    (schema: Record<string, string>) => {
      dispatch(updateOAuth2SchemaAction(schema));
    },
    [dispatch],
  );

  const updateOAuth2BodyPath = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      dispatch(updateOAuth2BodyPathAction(e.target.value));
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
    const result = await dispatch(saveSettingsThunk({ settings }));
    if (saveSettingsThunk.fulfilled.match(result)) {
      showNotification('Settings saved successfully', 'success');
    } else {
      showNotification(
        (result.payload as string) ?? 'Failed to save settings',
        'error',
      );
    }
  }, [dispatch, settings, showNotification]);

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

  const handleToggleShowAdvanced = useCallback(() => {
    dispatch(toggleShowAdvanced());
  }, [dispatch]);

  return {
    showAdvanced,
    settings,
    saved,
    dirty,
    loadingStatus,
    isSaving,
    error,
    updateBearer,
    updateBearerSchema,
    updateBearerBodyPath,
    updateOAuth2,
    updateOAuth2Schema,
    updateOAuth2BodyPath,
    updateRetry,
    handleAuthTypeChange,
    handleSave,
    handleTestConnection,
    handleClearError,
    handleToggleShowAdvanced,
  } as const;
}
