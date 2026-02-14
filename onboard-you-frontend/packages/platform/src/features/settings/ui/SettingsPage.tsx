import { Button } from '@/shared/ui/Button';
import { Badge } from '@/shared/ui/Badge';
import { Card } from '@/shared/ui/Card';
import { PLACEMENT_OPTIONS, GRANT_TYPE_OPTIONS } from '../domain/types';
import { useSettingsState } from '../state/useSettingsState';
import styles from './SettingsPage.module.scss';

export function SettingsPage() {
  const {
    settings,
    saved,
    dirty,
    updateBearer,
    updateOAuth2,
    updateRetry,
    handleAuthTypeChange,
    handleSave,
    handleTestConnection,
  } = useSettingsState();

  return (
    <div className={styles.page}>
      {/* Header */}
      <div className={styles.header}>
        <div>
          <h1 className={styles.title}>My Systems</h1>
          <p className={styles.subtitle}>
            Configure where processed employee data is dispatched after each
            pipeline run.
          </p>
        </div>
        <div className={styles.headerActions}>
          {saved && <Badge variant="active">Saved ✓</Badge>}
          {dirty && <Badge variant="draft">Unsaved</Badge>}
        </div>
      </div>

      {/* ── Auth type selector ───────────────────────────── */}
      <Card className={styles.section}>
        <h2 className={styles.sectionTitle}>Authentication Type</h2>
        <p className={styles.sectionDesc}>
          Select how the dispatcher authenticates with your destination API.
        </p>
        <div className={styles.authToggle}>
          <button
            type="button"
            className={`${styles.authOption} ${settings.authType === 'bearer' ? styles.authOptionActive : ''}`}
            onClick={() => handleAuthTypeChange('bearer')}
          >
            <span className={styles.authIcon}>🔑</span>
            <span className={styles.authLabel}>Bearer Token</span>
            <span className={styles.authDesc}>Static token, API key, or no auth</span>
          </button>
          <button
            type="button"
            className={`${styles.authOption} ${settings.authType === 'oauth2' ? styles.authOptionActive : ''}`}
            onClick={() => handleAuthTypeChange('oauth2')}
          >
            <span className={styles.authIcon}>🛡️</span>
            <span className={styles.authLabel}>OAuth 2.0</span>
            <span className={styles.authDesc}>Client credentials or authorization code</span>
          </button>
        </div>
      </Card>

      {/* ── Bearer config ────────────────────────────────── */}
      {settings.authType === 'bearer' && (
        <Card className={styles.section}>
          <h2 className={styles.sectionTitle}>Bearer / API Key Configuration</h2>

          <div className={styles.formGroup}>
            <label className={styles.formLabel}>Destination URL</label>
            <input
              className={styles.formInput}
              type="url"
              placeholder="https://api.example.com/employees"
              value={settings.bearer.destinationUrl}
              onChange={updateBearer('destinationUrl')}
            />
            <span className={styles.formHint}>
              The endpoint that receives the JSON payload after each pipeline run.
            </span>
          </div>

          <div className={styles.formGroup}>
            <label className={styles.formLabel}>Token</label>
            <input
              className={styles.formInput}
              type="password"
              placeholder="Bearer sk-…  (leave blank for unauthenticated)"
              value={settings.bearer.token}
              onChange={updateBearer('token')}
            />
          </div>

          <div className={styles.formRow}>
            <div className={styles.formGroup}>
              <label className={styles.formLabel}>Token Placement</label>
              <select
                className={styles.formSelect}
                value={settings.bearer.placement}
                onChange={updateBearer('placement')}
              >
                {PLACEMENT_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {opt.label}
                  </option>
                ))}
              </select>
            </div>
            <div className={styles.formGroup}>
              <label className={styles.formLabel}>Placement Key</label>
              <input
                className={styles.formInput}
                type="text"
                placeholder="Authorization"
                value={settings.bearer.placementKey}
                onChange={updateBearer('placementKey')}
              />
              <span className={styles.formHint}>
                Header name or query parameter key.
              </span>
            </div>
          </div>
        </Card>
      )}

      {/* ── OAuth 2.0 config ─────────────────────────────── */}
      {settings.authType === 'oauth2' && (
        <Card className={styles.section}>
          <h2 className={styles.sectionTitle}>OAuth 2.0 Configuration</h2>

          <div className={styles.formGroup}>
            <label className={styles.formLabel}>Destination URL</label>
            <input
              className={styles.formInput}
              type="url"
              placeholder="https://api.example.com/employees"
              value={settings.oauth2.destinationUrl}
              onChange={updateOAuth2('destinationUrl')}
            />
          </div>

          <div className={styles.formRow}>
            <div className={styles.formGroup}>
              <label className={styles.formLabel}>Client ID</label>
              <input
                className={styles.formInput}
                type="text"
                placeholder="client-id"
                value={settings.oauth2.clientId}
                onChange={updateOAuth2('clientId')}
              />
            </div>
            <div className={styles.formGroup}>
              <label className={styles.formLabel}>Client Secret</label>
              <input
                className={styles.formInput}
                type="password"
                placeholder="••••••••"
                value={settings.oauth2.clientSecret}
                onChange={updateOAuth2('clientSecret')}
              />
            </div>
          </div>

          <div className={styles.formGroup}>
            <label className={styles.formLabel}>Token URL</label>
            <input
              className={styles.formInput}
              type="url"
              placeholder="https://auth.example.com/oauth2/token"
              value={settings.oauth2.tokenUrl}
              onChange={updateOAuth2('tokenUrl')}
            />
          </div>

          <div className={styles.formRow}>
            <div className={styles.formGroup}>
              <label className={styles.formLabel}>Grant Type</label>
              <select
                className={styles.formSelect}
                value={settings.oauth2.grantType}
                onChange={updateOAuth2('grantType')}
              >
                {GRANT_TYPE_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {opt.label}
                  </option>
                ))}
              </select>
            </div>
            <div className={styles.formGroup}>
              <label className={styles.formLabel}>Scopes</label>
              <input
                className={styles.formInput}
                type="text"
                placeholder="read write (space-separated)"
                value={settings.oauth2.scopes}
                onChange={updateOAuth2('scopes')}
              />
            </div>
          </div>

          {settings.oauth2.grantType === 'authorization_code' && (
            <div className={styles.formGroup}>
              <label className={styles.formLabel}>Refresh Token</label>
              <input
                className={styles.formInput}
                type="password"
                placeholder="refresh-token"
                value={settings.oauth2.refreshToken}
                onChange={updateOAuth2('refreshToken')}
              />
            </div>
          )}
        </Card>
      )}

      {/* ── Retry policy ─────────────────────────────────── */}
      <Card className={styles.section}>
        <h2 className={styles.sectionTitle}>Retry Policy</h2>
        <p className={styles.sectionDesc}>
          Controls automatic retries on transient failures.
        </p>

        <div className={styles.formRow}>
          <div className={styles.formGroup}>
            <label className={styles.formLabel}>Max Attempts</label>
            <input
              className={styles.formInput}
              type="number"
              min={1}
              max={10}
              value={settings.retryPolicy.maxAttempts}
              onChange={updateRetry('maxAttempts')}
            />
          </div>
          <div className={styles.formGroup}>
            <label className={styles.formLabel}>Initial Back-off (ms)</label>
            <input
              className={styles.formInput}
              type="number"
              min={100}
              step={100}
              value={settings.retryPolicy.initialBackoffMs}
              onChange={updateRetry('initialBackoffMs')}
            />
          </div>
        </div>

        <div className={styles.formGroup}>
          <label className={styles.formLabel}>Retryable Status Codes</label>
          <input
            className={styles.formInput}
            type="text"
            placeholder="429, 502, 503, 504"
            value={settings.retryPolicy.retryableStatusCodes.join(', ')}
            onChange={updateRetry('retryableStatusCodes')}
          />
          <span className={styles.formHint}>
            Comma-separated HTTP status codes that trigger a retry.
          </span>
        </div>
      </Card>

      {/* ── Footer actions ───────────────────────────────── */}
      <div className={styles.footer}>
        <Button variant="secondary" size="md" onClick={handleTestConnection}>
          🔗 Test Connection
        </Button>
        <Button variant="primary" size="md" onClick={handleSave} disabled={!dirty}>
          Save Settings
        </Button>
      </div>
    </div>
  );
}
