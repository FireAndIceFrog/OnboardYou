import { useTranslation } from 'react-i18next';
import { Button } from '@/shared/ui/Button';
import { HR_SYSTEMS, RESPONSE_GROUP_OPTIONS } from '../domain/types';
import { useConnectionForm } from '../state/useConnectionForm';
import { FieldError } from './FieldError';
import styles from './ConnectionDetailsPage.module.scss';

/** Return the combined className for an input, adding the invalid class when an error exists. */
function inputClass(error?: string) {
  return error
    ? `${styles.formInput} ${styles.formInputInvalid}`
    : styles.formInput;
}

export function ConnectionDetailsPage() {
  const {
    form,
    errors,
    isValid,
    activeGroups,
    handleSystemSelect,
    handleChange,
    handleWorkdayChange,
    handleCsvChange,
    handleResponseGroupToggle,
    handleNext,
    handleBack,
    validateField,
  } = useConnectionForm();
  const { t } = useTranslation();

  return (
    <div className={styles.wizardPage}>
      {/* Step indicator */}
      <nav className={styles.stepIndicator} aria-label="Configuration steps">
        <div className={styles.step} aria-current="step">
          <span className={`${styles.stepCircle} ${styles.stepCircleActive}`}>1</span>
          <span className={`${styles.stepLabel} ${styles.stepLabelActive}`}>{t('configDetails.steps.connectionDetails')}</span>
        </div>
        <div className={styles.stepConnector} />
        <div className={styles.step}>
          <span className={styles.stepCircle}>2</span>
          <span className={styles.stepLabel}>{t('configDetails.steps.flowCustomization')}</span>
        </div>
      </nav>

      {/* Form card */}
      <form className={styles.card} onSubmit={(e) => e.preventDefault()}>
        <h2 className={styles.cardTitle}>{t('configDetails.connection.title')}</h2>
        <p className={styles.cardSubtitle}>
          {t('configDetails.connection.subtitle')}
        </p>

        {/* System selector */}
        <div className={styles.formGroup}>
          <label className={styles.formLabel}>{t('configDetails.connection.hrSystem')}</label>
          <div className={`${styles.systemGrid} ${errors.system ? styles.systemGridInvalid : ''}`}>
            {HR_SYSTEMS.map((sys) => (
              <button
                key={sys.id}
                type="button"
                className={`${styles.systemCard} ${form.system === sys.id ? styles.systemCardSelected : ''}`}
                onClick={() => handleSystemSelect(sys.id)}
              >
                <span className={styles.systemIcon}>{sys.icon}</span>
                <span className={styles.systemName}>{sys.name}</span>
              </button>
            ))}
          </div>
          <FieldError id="system-error" error={errors.system} />
        </div>

        {/* Display name (always shown once a system is picked) */}
        {form.system && (
          <div className={styles.formGroup}>
            <label className={styles.formLabel} htmlFor="conn-display-name">{t('configDetails.connection.displayName')}</label>
            <input
              id="conn-display-name"
              className={styles.formInput}
              type="text"
              placeholder={t('configDetails.connection.displayNamePlaceholder')}
              value={form.displayName}
              onChange={handleChange('displayName')}
            />
            <span className={styles.formHint}>{t('configDetails.connection.displayNameHint')}</span>
          </div>
        )}

        {/* ── Workday-specific fields (WS-Security) ──────── */}
        {form.system === 'workday' && (
          <>
            <fieldset>
            <legend className={styles.sectionHeader}>
              <span className={styles.sectionIcon}>🔐</span>
              <span>{t('configDetails.connection.workday.credentialsTitle')}</span>
            </legend>

            <div className={styles.formGroup}>
              <label className={styles.formLabel} htmlFor="conn-tenant-url">{t('configDetails.connection.workday.tenantUrl')}</label>
              <input
                id="conn-tenant-url"
                className={inputClass(errors['workday.tenantUrl'])}
                type="url"
                placeholder={t('configDetails.connection.workday.tenantUrlPlaceholder')}
                value={form.workday.tenantUrl}
                onChange={handleWorkdayChange('tenantUrl')}
                onBlur={() => validateField('workday.tenantUrl')}
                aria-invalid={!!errors['workday.tenantUrl']}
                aria-describedby={errors['workday.tenantUrl'] ? 'conn-tenant-url-error' : undefined}
              />
              <FieldError id="conn-tenant-url-error" error={errors['workday.tenantUrl']} />
              <span className={styles.formHint}>
                {t('configDetails.connection.workday.tenantUrlHint')}
              </span>
            </div>

            <div className={styles.formGroup}>
              <label className={styles.formLabel} htmlFor="conn-tenant-id">{t('configDetails.connection.workday.tenantId')}</label>
              <input
                id="conn-tenant-id"
                className={inputClass(errors['workday.tenantId'])}
                type="text"
                placeholder={t('configDetails.connection.workday.tenantIdPlaceholder')}
                value={form.workday.tenantId}
                onChange={handleWorkdayChange('tenantId')}
                onBlur={() => validateField('workday.tenantId')}
                aria-invalid={!!errors['workday.tenantId']}
                aria-describedby={errors['workday.tenantId'] ? 'conn-tenant-id-error' : undefined}
              />
              <FieldError id="conn-tenant-id-error" error={errors['workday.tenantId']} />
            </div>

            <div className={styles.formRow}>
              <div className={styles.formGroup}>
                <label className={styles.formLabel} htmlFor="conn-username">{t('configDetails.connection.workday.username')}</label>
                <input
                  id="conn-username"
                  className={inputClass(errors['workday.username'])}
                  type="text"
                  placeholder={t('configDetails.connection.workday.usernamePlaceholder')}
                  value={form.workday.username}
                  onChange={handleWorkdayChange('username')}
                  onBlur={() => validateField('workday.username')}
                  aria-invalid={!!errors['workday.username']}
                  aria-describedby={errors['workday.username'] ? 'conn-username-error' : undefined}
                />
                <FieldError id="conn-username-error" error={errors['workday.username']} />
              </div>
              <div className={styles.formGroup}>
                <label className={styles.formLabel} htmlFor="conn-password">{t('configDetails.connection.workday.password')}</label>
                <input
                  id="conn-password"
                  className={inputClass(errors['workday.password'])}
                  type="password"
                  placeholder={t('configDetails.connection.workday.passwordPlaceholder')}
                  value={form.workday.password}
                  onChange={handleWorkdayChange('password')}
                  onBlur={() => validateField('workday.password')}
                  aria-invalid={!!errors['workday.password']}
                  aria-describedby={errors['workday.password'] ? 'conn-password-error' : undefined}
                />
                <FieldError id="conn-password-error" error={errors['workday.password']} />
              </div>
            </div>

            </fieldset>

            <fieldset>
            <legend className={styles.sectionHeader}>
              <span className={styles.sectionIcon}>⚙️</span>
              <span>{t('configDetails.connection.workday.queryOptionsTitle')}</span>
            </legend>

            <div className={styles.formGroup}>
              <label className={styles.formLabel} htmlFor="conn-worker-count">{t('configDetails.connection.workday.workerCountLimit')}</label>
              <input
                id="conn-worker-count"
                className={styles.formInput}
                type="number"
                min={1}
                max={999}
                value={form.workday.workerCountLimit}
                onChange={handleWorkdayChange('workerCountLimit')}
              />
              <span className={styles.formHint}>
                {t('configDetails.connection.workday.workerCountLimitHint')}
              </span>
            </div>

            <div className={styles.formGroup}>
              <label className={styles.formLabel}>{t('configDetails.connection.workday.responseGroups')}</label>
              <div className={`${styles.chipGroup} ${errors['workday.responseGroup'] ? styles.chipGroupInvalid : ''}`}>
                {RESPONSE_GROUP_OPTIONS.map((opt) => (
                  <button
                    key={opt.value}
                    type="button"
                    className={`${styles.chip} ${activeGroups.has(opt.value) ? styles.chipActive : ''}`}
                    aria-pressed={activeGroups.has(opt.value)}
                    onClick={() => handleResponseGroupToggle(opt.value)}
                  >
                    {opt.label}
                  </button>
                ))}
              </div>
              <FieldError id="response-group-error" error={errors['workday.responseGroup']} />
              <span className={styles.formHint}>
                {t('configDetails.connection.workday.responseGroupsHint')}
              </span>
            </div>
            </fieldset>
          </>
        )}

        {/* ── CSV-specific fields ────────────────────────── */}
        {form.system === 'csv' && (
          <>
            <fieldset>
            <legend className={styles.sectionHeader}>
              <span className={styles.sectionIcon}>📁</span>
              <span>{t('configDetails.connection.csv.fileLocationTitle')}</span>
            </legend>

            <div className={styles.formGroup}>
              <label className={styles.formLabel} htmlFor="conn-csv-path">{t('configDetails.connection.csv.csvFilePath')}</label>
              <input
                id="conn-csv-path"
                className={inputClass(errors['csv.csvPath'])}
                type="text"
                placeholder={t('configDetails.connection.csv.csvFilePathPlaceholder')}
                value={form.csv.csvPath}
                onChange={handleCsvChange('csvPath')}
                onBlur={() => validateField('csv.csvPath')}
                aria-invalid={!!errors['csv.csvPath']}
                aria-describedby={errors['csv.csvPath'] ? 'csv-path-error' : undefined}
              />
              <FieldError id="csv-path-error" error={errors['csv.csvPath']} />
              <span className={styles.formHint}>
                {t('configDetails.connection.csv.csvFilePathHint')}
              </span>
            </div>
            </fieldset>
          </>
        )}
      </form>

      {/* Actions */}
      <div className={styles.actions}>
        <Button variant="secondary" size="md" onClick={handleBack}>
          {t('configDetails.connection.backButton')}
        </Button>
        <Button variant="primary" size="md" disabled={!isValid} onClick={handleNext}>
          {t('configDetails.connection.nextButton')}
        </Button>
      </div>
    </div>
  );
}
