import { useConfigDetails } from '../state/ConfigDetailsContext';
import { businessLabel } from '@/shared/domain/types';
import styles from './ConfigDetailsForm.module.scss';

const CATEGORY_ICONS: Record<string, string> = {
  ingestion: '📥',
  logic: '⚙️',
  egress: '📤',
};

const CATEGORY_LABELS: Record<string, string> = {
  ingestion: 'Data Source',
  logic: 'Business Rule',
  egress: 'Destination',
};

export function ConfigDetailsForm() {
  const { state, dispatch } = useConfigDetails();
  const { selectedNode } = state;

  if (!selectedNode) return null;

  const nodeData = selectedNode.data as Record<string, unknown>;
  const category = (nodeData.category as string) ?? 'logic';
  const actionType = (nodeData.actionType as string) ?? '';
  const config = nodeData.config as Record<string, unknown> | undefined;

  const handleClose = () => {
    dispatch({ type: 'DESELECT_NODE' });
  };

  const configEntries = config ? Object.entries(config) : [];

  return (
    <div className={styles.formPanel}>
      {/* Header */}
      <div className={styles.formHeader}>
        <div className={styles.formTitle}>
          <span>{CATEGORY_ICONS[category] ?? '🔧'}</span>
          <span>{(nodeData.label as string) ?? CATEGORY_LABELS[category] ?? 'Node Details'}</span>
        </div>
        <button type="button" className={styles.closeBtn} onClick={handleClose} aria-label="Close">
          ×
        </button>
      </div>

      {/* Body */}
      <div className={styles.formBody}>
        {/* Action type field */}
        <div className={styles.configField}>
          <div className={styles.configLabel}>Step Type</div>
          <div className={styles.configValue}>{businessLabel(actionType)}</div>
        </div>

        {/* Category field */}
        <div className={styles.configField}>
          <div className={styles.configLabel}>Category</div>
          <div className={styles.configValue}>{CATEGORY_LABELS[category] ?? category}</div>
        </div>

        {/* Config key-value pairs */}
        {configEntries.length > 0 ? (
          configEntries.map(([key, value]) => (
            <div key={key} className={styles.configField}>
              <div className={styles.configLabel}>{key}</div>
              <div className={styles.configValue}>
                {typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean'
                  ? String(value)
                  : JSON.stringify(value)}
              </div>
            </div>
          ))
        ) : (
          <div className={styles.configField}>
            <div className={styles.configLabel}>Configuration</div>
            <div className={styles.configValue}>No configuration data</div>
          </div>
        )}

        {/* JSON fallback block */}
        {config && Object.keys(config).length > 0 && (
          <div className={styles.jsonBlock}>
            <pre>{JSON.stringify(config, null, 2)}</pre>
          </div>
        )}
      </div>
    </div>
  );
}
