import type { Node } from '@xyflow/react';
import styles from './ConfigDetailsPage.module.scss';

interface ConfigDetailsFormProps {
  node: Node;
  onClose: () => void;
}

function formatValue(value: unknown, indent: number = 0): string {
  if (value === null || value === undefined) return 'null';
  if (typeof value === 'string') return value;
  if (typeof value === 'number' || typeof value === 'boolean') return String(value);
  if (Array.isArray(value)) {
    return value.map((v) => formatValue(v, indent + 1)).join(', ');
  }
  if (typeof value === 'object') {
    return JSON.stringify(value, null, 2);
  }
  return String(value);
}

export function ConfigDetailsForm({ node, onClose }: ConfigDetailsFormProps) {
  const nodeData = node.data as Record<string, unknown>;
  const label = String(nodeData.label ?? '');
  const stageType = nodeData.stageType ? String(nodeData.stageType) : '';
  const source = nodeData.source ? String(nodeData.source) : '';
  const destination = nodeData.destination ? String(nodeData.destination) : '';
  const status = nodeData.status ? String(nodeData.status) : '';
  const configRecord = (nodeData.config ?? {}) as Record<string, unknown>;

  return (
    <div className={styles.formOverlay} onClick={onClose}>
      <aside className={styles.formPanel} onClick={(e) => e.stopPropagation()}>
        <div className={styles.formHeader}>
          <h2 className={styles.formTitle}>{label}</h2>
          <button className={styles.formClose} onClick={onClose} aria-label="Close panel">
            ×
          </button>
        </div>

        <div className={styles.formBody}>
          {stageType && (
            <div className={styles.formField}>
              <label className={styles.formLabel}>Type</label>
              <span className={styles.formValue}>{stageType}</span>
            </div>
          )}

          {source && (
            <div className={styles.formField}>
              <label className={styles.formLabel}>Source</label>
              <span className={styles.formValue}>{source}</span>
            </div>
          )}

          {destination && (
            <div className={styles.formField}>
              <label className={styles.formLabel}>Destination</label>
              <span className={styles.formValue}>{destination}</span>
            </div>
          )}

          {status && (
            <div className={styles.formField}>
              <label className={styles.formLabel}>Status</label>
              <span className={styles.formValue}>{status}</span>
            </div>
          )}

          <div className={styles.formDivider} />

          <h3 className={styles.formSectionTitle}>Configuration</h3>
          {Object.keys(configRecord).length === 0 ? (
            <p className={styles.formEmpty}>No configuration parameters.</p>
          ) : (
            Object.entries(configRecord).map(([key, value]) => (
              <div key={key} className={styles.formField}>
                <label className={styles.formLabel}>{key}</label>
                {typeof value === 'object' && value !== null ? (
                  <pre className={styles.formCode}>{formatValue(value)}</pre>
                ) : (
                  <span className={styles.formValue}>{formatValue(value)}</span>
                )}
              </div>
            ))
          )}
        </div>
      </aside>
    </div>
  );
}
