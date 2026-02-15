import { useTranslation } from 'react-i18next';
import { Button } from '@/shared/ui/Button';
import { Badge } from '@/shared/ui/Badge';
import { Card } from '@/shared/ui/Card';
import { Spinner } from '@/shared/ui/Spinner';
import { PLACEMENT_OPTIONS, GRANT_TYPE_OPTIONS } from '../domain/types';
import { useSettingsState } from '../state/useSettingsState';
import { useSettingsValidation } from '../state/useSettingsValidation';
import { FieldError } from './FieldError';
import styles from './SettingsPage.module.scss';

/** Return the combined className for an input, adding the invalid class when an error exists. */
function inputClass(error?: string) {
  return error
    ? `${styles.formInput} ${styles.formInputInvalid}`
    : styles.formInput;
}

export function SettingsPage() {
  const { t } = useTranslation();
  const {
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
  } = useSettingsState();

  const { errors, isValid, validateAll } = useSettingsValidation(settings);

  const onSave = () => {
    if (!validateAll()) return;
    handleSave();
  };

  if (isLoading) {
    return (
      <div className={styles.page} role="status" aria-label={t('settings.loading')}>
        <Spinner />
      </div>
    );
  }

  return (
    <form className={styles.page} onSubmit={(e) => e.preventDefault()}>
      {/* Header */}
      <div className={styles.header}>
        <div>
          <h1 className={styles.title}>{t('settings.title')}</h1>
          <p className={styles.subtitle}>
            {t('settings.subtitle')}
          </p>
        </div>
        <div className={styles.headerActions}>
          {saved && <Badge variant="active">{t('settings.saved')}</Badge>}
          {dirty && <Badge variant="draft">{t('settings.unsaved')}</Badge>}
          {isSaving && <Badge variant="draft">{t('settings.saving')}</Badge>}
        </div>
      </div>

      {/* ── Error banner ─────────────────────────────────── */}
      {error && (
        <Card className={styles.section} role="alert">
          <div className={styles.errorBanner}>
            <p>{error}</p>
            <button type="button" onClick={handleClearError} aria-label={t('settings.dismissError')}>
              ✕
            </button>
          </div>
        </Card>
      )}

      {/* ── Auth type selector ───────────────────────────── */}
      <Card className={styles.section}>
        <fieldset>
        <legend className={styles.sectionTitle}>{t('settings.authType.title')}</legend>
        <p className={styles.sectionDesc}>
          {t('settings.authType.description')}
        </p>
        <div className={styles.authToggle}>
          <button
            type="button"
            className={`${styles.authOption} ${settings.authType === 'bearer' ? styles.authOptionActive : ''}`}
            onClick={() => handleAuthTypeChange('bearer')}
          >
            <span className={styles.authIcon}>🔑</span>
            <span className={styles.authLabel}>{t('settings.authType.bearer')}</span>
            <span className={styles.authDesc}>{t('settings.authType.bearerDesc')}</span>
          </button>
          <button
            type="button"
            className={`${styles.authOption} ${settings.authType === 'oauth2' ? styles.authOptionActive : ''}`}
            onClick={() => handleAuthTypeChange('oauth2')}
          >
            <span className={styles.authIcon}>🛡️</span>
            <span className={styles.authLabel}>{t('settings.authType.oauth2')}</span>
            <span className={styles.authDesc}>{t('settings.authType.oauth2Desc')}</span>
          </button>
        </div>
        </fieldset>
      </Card>

      {/* ── Bearer config ────────────────────────────────── */}
      {settings.authType === 'bearer' && (
        <Card className={styles.section}>
          <fieldset>
          <legend className={styles.sectionTitle}>{t('settings.bearer.title')}</legend>

          <div className={styles.formGroup}>
            <label className={styles.formLabel} htmlFor="bearer-destination-url">{t('settings.bearer.destinationUrl')}</label>
            <input
              id="bearer-destination-url"
              className={styles.formInput}
              type="url"
              placeholder={t('settings.bearer.destinationUrlPlaceholder')}
              value={settings.bearer.destinationUrl}
              onChange={updateBearer('destinationUrl')}
            />
            <span className={styles.formHint}>
              {t('settings.bearer.destinationUrlHint')}
            </span>
          </div>

          <div className={styles.formGroup}>
            <label className={styles.formLabel} htmlFor="bearer-token">{t('settings.bearer.token')}</label>
            <input
              id="bearer-token"
              className={inputClass(errors['bearer.token'])}
              type="password"
              placeholder={t('settings.bearer.tokenPlaceholder')}
              value={settings.bearer.token}
              onChange={updateBearer('token')}
              aria-invalid={!!errors['bearer.token']}
              aria-describedby={errors['bearer.token'] ? 'bearer-token-error' : undefined}
            />
            <FieldError id="bearer-token-error" error={errors['bearer.token']} />
          </div>

          <div className={styles.formRow}>
            <div className={styles.formGroup}>
              <label className={styles.formLabel} htmlFor="bearer-placement">{t('settings.bearer.tokenPlacement')}</label>
              <select
                id="bearer-placement"
                className={styles.formSelect}
                value={settings.bearer.placement}
                onChange={updateBearer('placement')}
              >
                {PLACEMENT_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {t(`settings.placementOptions.${opt.value}`)}
                  </option>
                ))}
              </select>
            </div>
            <div className={styles.formGroup}>
              <label className={styles.formLabel} htmlFor="bearer-placement-key">{t('settings.bearer.placementKey')}</label>
              <input
                id="bearer-placement-key"
                className={styles.formInput}
                type="text"
                placeholder={t('settings.bearer.placementKeyPlaceholder')}
                value={settings.bearer.placementKey}
                onChange={updateBearer('placementKey')}
              />
              <span className={styles.formHint}>
                {t('settings.bearer.placementKeyHint')}
              </span>
            </div>
          </div>
          </fieldset>
        </Card>
      )}

      {/* ── OAuth 2.0 config ─────────────────────────────── */}
      {settings.authType === 'oauth2' && (
        <Card className={styles.section}>
          <fieldset>
          <legend className={styles.sectionTitle}>{t('settings.oauth2.title')}</legend>

          <div className={styles.formGroup}>
            <label className={styles.formLabel} htmlFor="oauth2-destination-url">{t('settings.oauth2.destinationUrl')}</label>
            <input
              id="oauth2-destination-url"
              className={styles.formInput}
              type="url"
              placeholder={t('settings.oauth2.destinationUrlPlaceholder')}
              value={settings.oauth2.destinationUrl}
              onChange={updateOAuth2('destinationUrl')}
            />
          </div>

          <div className={styles.formRow}>
            <div className={styles.formGroup}>
              <label className={styles.formLabel} htmlFor="oauth2-client-id">{t('settings.oauth2.clientId')}</label>
              <input
                id="oauth2-client-id"
                className={inputClass(errors['oauth2.clientId'])}
                type="text"
                placeholder={t('settings.oauth2.clientIdPlaceholder')}
                value={settings.oauth2.clientId}
                onChange={updateOAuth2('clientId')}
                aria-invalid={!!errors['oauth2.clientId']}
                aria-describedby={errors['oauth2.clientId'] ? 'oauth2-client-id-error' : undefined}
              />
              <FieldError id="oauth2-client-id-error" error={errors['oauth2.clientId']} />
            </div>
            <div className={styles.formGroup}>
              <label className={styles.formLabel} htmlFor="oauth2-client-secret">{t('settings.oauth2.clientSecret')}</label>
              <input
                id="oauth2-client-secret"
                className={inputClass(errors['oauth2.clientSecret'])}
                type="password"
                placeholder={t('settings.oauth2.clientSecretPlaceholder')}
                value={settings.oauth2.clientSecret}
                onChange={updateOAuth2('clientSecret')}
                aria-invalid={!!errors['oauth2.clientSecret']}
                aria-describedby={errors['oauth2.clientSecret'] ? 'oauth2-client-secret-error' : undefined}
              />
              <FieldError id="oauth2-client-secret-error" error={errors['oauth2.clientSecret']} />
            </div>
          </div>

          <div className={styles.formGroup}>
            <label className={styles.formLabel} htmlFor="oauth2-token-url">{t('settings.oauth2.tokenUrl')}</label>
            <input
              id="oauth2-token-url"
              className={inputClass(errors['oauth2.tokenUrl'])}
              type="url"
              placeholder={t('settings.oauth2.tokenUrlPlaceholder')}
              value={settings.oauth2.tokenUrl}
              onChange={updateOAuth2('tokenUrl')}
              aria-invalid={!!errors['oauth2.tokenUrl']}
              aria-describedby={errors['oauth2.tokenUrl'] ? 'oauth2-token-url-error' : undefined}
            />
            <FieldError id="oauth2-token-url-error" error={errors['oauth2.tokenUrl']} />
          </div>

          <div className={styles.formRow}>
            <div className={styles.formGroup}>
              <label className={styles.formLabel} htmlFor="oauth2-grant-type">{t('settings.oauth2.grantType')}</label>
              <select
                id="oauth2-grant-type"
                className={styles.formSelect}
                value={settings.oauth2.grantType}
                onChange={updateOAuth2('grantType')}
              >
                {GRANT_TYPE_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {t(`settings.grantTypeOptions.${opt.value}`)}
                  </option>
                ))}
              </select>
            </div>
            <div className={styles.formGroup}>
              <label className={styles.formLabel} htmlFor="oauth2-scopes">{t('settings.oauth2.scopes')}</label>
              <input
                id="oauth2-scopes"
                className={styles.formInput}
                type="text"
                placeholder={t('settings.oauth2.scopesPlaceholder')}
                value={settings.oauth2.scopes}
                onChange={updateOAuth2('scopes')}
              />
            </div>
          </div>

          {settings.oauth2.grantType === 'authorization_code' && (
            <div className={styles.formGroup}>
              <label className={styles.formLabel} htmlFor="oauth2-refresh-token">{t('settings.oauth2.refreshToken')}</label>
              <input
                id="oauth2-refresh-token"
                className={styles.formInput}
                type="password"
                placeholder={t('settings.oauth2.refreshTokenPlaceholder')}
                value={settings.oauth2.refreshToken}
                onChange={updateOAuth2('refreshToken')}
              />
            </div>
          )}
          </fieldset>
        </Card>
      )}

      {/* ── Retry policy ─────────────────────────────────── */}
      <Card className={styles.section}>
        <fieldset>
        <legend className={styles.sectionTitle}>{t('settings.retry.title')}</legend>
        <p className={styles.sectionDesc}>
          {t('settings.retry.description')}
        </p>

        <div className={styles.formRow}>
          <div className={styles.formGroup}>
            <label className={styles.formLabel} htmlFor="retry-max-attempts">{t('settings.retry.maxAttempts')}</label>
            <input
              id="retry-max-attempts"
              className={inputClass(errors['retry.maxAttempts'])}
              type="number"
              min={1}
              max={10}
              value={settings.retryPolicy.maxAttempts}
              onChange={updateRetry('maxAttempts')}
              aria-invalid={!!errors['retry.maxAttempts']}
              aria-describedby={errors['retry.maxAttempts'] ? 'retry-max-attempts-error' : undefined}
            />
            <FieldError id="retry-max-attempts-error" error={errors['retry.maxAttempts']} />
          </div>
          <div className={styles.formGroup}>
            <label className={styles.formLabel} htmlFor="retry-initial-backoff">{t('settings.retry.initialBackoff')}</label>
            <input
              id="retry-initial-backoff"
              className={inputClass(errors['retry.initialBackoffMs'])}
              type="number"
              min={100}
              step={100}
              value={settings.retryPolicy.initialBackoffMs}
              onChange={updateRetry('initialBackoffMs')}
              aria-invalid={!!errors['retry.initialBackoffMs']}
              aria-describedby={errors['retry.initialBackoffMs'] ? 'retry-initial-backoff-error' : undefined}
            />
            <FieldError id="retry-initial-backoff-error" error={errors['retry.initialBackoffMs']} />
          </div>
        </div>

        <div className={styles.formGroup}>
          <label className={styles.formLabel} htmlFor="retry-status-codes">{t('settings.retry.retryableStatusCodes')}</label>
          <input
            id="retry-status-codes"
            className={styles.formInput}
            type="text"
            placeholder={t('settings.retry.retryableStatusCodesPlaceholder')}
            value={settings.retryPolicy.retryableStatusCodes.join(', ')}
            onChange={updateRetry('retryableStatusCodes')}
          />
          <span className={styles.formHint}>
            {t('settings.retry.retryableStatusCodesHint')}
          </span>
        </div>
        </fieldset>
      </Card>

      {/* ── Footer actions ───────────────────────────────── */}
      <div className={styles.footer}>
        <Button variant="secondary" size="md" onClick={handleTestConnection}>
          {t('settings.testConnection')}
        </Button>
        <Button variant="primary" size="md" onClick={onSave} disabled={(!dirty && !isSaving) || !isValid || isSaving}>
          {isSaving ? t('settings.saving') : t('settings.saveSettings')}
        </Button>
      </div>
    </form>
  );
}
