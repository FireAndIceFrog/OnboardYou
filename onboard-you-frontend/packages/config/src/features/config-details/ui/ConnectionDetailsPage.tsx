import { Button } from '@/shared/ui/Button';
import { HR_SYSTEMS, RESPONSE_GROUP_OPTIONS } from '../domain/types';
import { useConnectionForm } from '../state/useConnectionForm';
import styles from './ConnectionDetailsPage.module.scss';

export function ConnectionDetailsPage() {
  const {
    form,
    isValid,
    activeGroups,
    handleSystemSelect,
    handleChange,
    handleWorkdayChange,
    handleCsvChange,
    handleResponseGroupToggle,
    handleNext,
    handleBack,
  } = useConnectionForm();

  return (
    <div className={styles.wizardPage}>
      {/* Step indicator */}
      <div className={styles.stepIndicator}>
        <div className={styles.step}>
          <span className={`${styles.stepCircle} ${styles.stepCircleActive}`}>1</span>
          <span className={`${styles.stepLabel} ${styles.stepLabelActive}`}>Connection Details</span>
        </div>
        <div className={styles.stepConnector} />
        <div className={styles.step}>
          <span className={styles.stepCircle}>2</span>
          <span className={styles.stepLabel}>Flow Customization</span>
        </div>
      </div>

      {/* Form card */}
      <div className={styles.card}>
        <h2 className={styles.cardTitle}>Connect Your HR System</h2>
        <p className={styles.cardSubtitle}>
          Choose your HR platform and provide the connection details.
        </p>

        {/* System selector */}
        <div className={styles.formGroup}>
          <label className={styles.formLabel}>HR System</label>
          <div className={styles.systemGrid}>
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
        </div>

        {/* Display name (always shown once a system is picked) */}
        {form.system && (
          <div className={styles.formGroup}>
            <label className={styles.formLabel}>Display Name</label>
            <input
              className={styles.formInput}
              type="text"
              placeholder="e.g. Acme Corp — Workday Sync"
              value={form.displayName}
              onChange={handleChange('displayName')}
            />
            <span className={styles.formHint}>A friendly name to identify this connection.</span>
          </div>
        )}

        {/* ── Workday-specific fields (WS-Security) ──────── */}
        {form.system === 'workday' && (
          <>
            <div className={styles.sectionHeader}>
              <span className={styles.sectionIcon}>🔐</span>
              <span>WS-Security Credentials</span>
            </div>

            <div className={styles.formGroup}>
              <label className={styles.formLabel}>Tenant URL</label>
              <input
                className={styles.formInput}
                type="url"
                placeholder="https://wd5-impl.workday.com/ccx/service/your_tenant"
                value={form.workday.tenantUrl}
                onChange={handleWorkdayChange('tenantUrl')}
              />
              <span className={styles.formHint}>
                Your Workday SOAP endpoint including the tenant path.
              </span>
            </div>

            <div className={styles.formGroup}>
              <label className={styles.formLabel}>Tenant ID</label>
              <input
                className={styles.formInput}
                type="text"
                placeholder="your_tenant"
                value={form.workday.tenantId}
                onChange={handleWorkdayChange('tenantId')}
              />
            </div>

            <div className={styles.formRow}>
              <div className={styles.formGroup}>
                <label className={styles.formLabel}>Username</label>
                <input
                  className={styles.formInput}
                  type="text"
                  placeholder="ISU_user@your_tenant"
                  value={form.workday.username}
                  onChange={handleWorkdayChange('username')}
                />
              </div>
              <div className={styles.formGroup}>
                <label className={styles.formLabel}>Password</label>
                <input
                  className={styles.formInput}
                  type="password"
                  placeholder="••••••••"
                  value={form.workday.password}
                  onChange={handleWorkdayChange('password')}
                />
              </div>
            </div>

            <div className={styles.sectionHeader}>
              <span className={styles.sectionIcon}>⚙️</span>
              <span>Query Options</span>
            </div>

            <div className={styles.formGroup}>
              <label className={styles.formLabel}>Worker Count Limit</label>
              <input
                className={styles.formInput}
                type="number"
                min={1}
                max={999}
                value={form.workday.workerCountLimit}
                onChange={handleWorkdayChange('workerCountLimit')}
              />
              <span className={styles.formHint}>
                Maximum workers per page (default 200).
              </span>
            </div>

            <div className={styles.formGroup}>
              <label className={styles.formLabel}>Response Groups</label>
              <div className={styles.chipGroup}>
                {RESPONSE_GROUP_OPTIONS.map((opt) => (
                  <button
                    key={opt.value}
                    type="button"
                    className={`${styles.chip} ${activeGroups.has(opt.value) ? styles.chipActive : ''}`}
                    onClick={() => handleResponseGroupToggle(opt.value)}
                  >
                    {opt.label}
                  </button>
                ))}
              </div>
              <span className={styles.formHint}>
                Select which data groups to pull from Workday.
              </span>
            </div>
          </>
        )}

        {/* ── CSV-specific fields ────────────────────────── */}
        {form.system === 'csv' && (
          <>
            <div className={styles.sectionHeader}>
              <span className={styles.sectionIcon}>📁</span>
              <span>File Location</span>
            </div>

            <div className={styles.formGroup}>
              <label className={styles.formLabel}>CSV File Path</label>
              <input
                className={styles.formInput}
                type="text"
                placeholder="/data/uploads/employees.csv"
                value={form.csv.csvPath}
                onChange={handleCsvChange('csvPath')}
              />
              <span className={styles.formHint}>
                Path to the CSV file that will be ingested by the pipeline.
              </span>
            </div>
          </>
        )}
      </div>

      {/* Actions */}
      <div className={styles.actions}>
        <Button variant="secondary" size="md" onClick={handleBack}>
          ← Back
        </Button>
        <Button variant="primary" size="md" disabled={!isValid} onClick={handleNext}>
          Next: Customize Flow →
        </Button>
      </div>
    </div>
  );
}
