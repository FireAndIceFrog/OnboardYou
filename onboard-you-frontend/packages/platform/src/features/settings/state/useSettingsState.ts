import { useState, useCallback } from 'react';
import type {
  EgressSettings,
  AuthType,
  BearerConfig,
  OAuth2Config,
  RetryPolicy,
} from '../domain/types';
import { DEFAULT_EGRESS_SETTINGS } from '../domain/types';

export function useSettingsState() {
  const [settings, setSettings] = useState<EgressSettings>(DEFAULT_EGRESS_SETTINGS);
  const [saved, setSaved] = useState(false);
  const [dirty, setDirty] = useState(false);

  /* ── Generic updaters ───────────────────────────────────── */
  const update = useCallback(<K extends keyof EgressSettings>(key: K, value: EgressSettings[K]) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
    setDirty(true);
    setSaved(false);
  }, []);

  const updateBearer = useCallback(
    (field: keyof BearerConfig) =>
      (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
        setSettings((prev) => ({
          ...prev,
          bearer: { ...prev.bearer, [field]: e.target.value },
        }));
        setDirty(true);
        setSaved(false);
      },
    [],
  );

  const updateOAuth2 = useCallback(
    (field: keyof OAuth2Config) =>
      (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
        setSettings((prev) => ({
          ...prev,
          oauth2: { ...prev.oauth2, [field]: e.target.value },
        }));
        setDirty(true);
        setSaved(false);
      },
    [],
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
        setSettings((prev) => ({
          ...prev,
          retryPolicy: { ...prev.retryPolicy, [field]: val },
        }));
        setDirty(true);
        setSaved(false);
      },
    [],
  );

  const handleAuthTypeChange = useCallback(
    (authType: AuthType) => {
      update('authType', authType);
    },
    [update],
  );

  const handleSave = useCallback(() => {
    // TODO: POST to API when backend endpoint is ready
    console.info('[Settings] Saving egress config:', JSON.stringify(settings, null, 2));
    setSaved(true);
    setDirty(false);
  }, [settings]);

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
