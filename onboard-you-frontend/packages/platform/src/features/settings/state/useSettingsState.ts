import { useCallback } from 'react';
import { useAppSelector, useAppDispatch } from '@/store';
import {
  setAuthType,
  updateBearerField,
  updateOAuth2Field,
  updateRetryField,
  save,
} from './settingsSlice';
import type {
  AuthType,
  BearerConfig,
  OAuth2Config,
  RetryPolicy,
} from '../domain/types';

export function useSettingsState() {
  const dispatch = useAppDispatch();
  const { settings, saved, dirty } = useAppSelector((state) => state.settings);

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

  const handleSave = useCallback(() => {
    // TODO: POST to API when backend endpoint is ready
    console.info('[Settings] Saving egress config:', JSON.stringify(settings, null, 2));
    dispatch(save());
  }, [dispatch, settings]);

  const handleTestConnection = useCallback(() => {
    // TODO: ping destination URL when backend endpoint is ready
    const url =
      settings.authType === 'bearer'
        ? settings.bearer.destinationUrl
        : settings.oauth2.destinationUrl;
    console.info(`[Settings] Testing connection to ${url}…`);
    alert(`Testing connection to:\n${url || '(no URL configured)'}`);
  }, [settings]);

  return {
    settings,
    saved,
    dirty,
    updateBearer,
    updateOAuth2,
    updateRetry,
    handleAuthTypeChange,
    handleSave,
    handleTestConnection,
  } as const;
}
