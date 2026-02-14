import { useConfigDetails } from '../state/ConfigDetailsContext';
import styles from './ConfigDetailsForm.module.scss';

const NODE_ICONS: Record<string, string> = {
  ingestion: '📥',
  transformation: '⚙️',
  egress: '📤',
};

const NODE_LABELS: Record<string, string> = {
  ingestion: 'Ingestion',
  transformation: 'Transformation',
  egress: 'Egress',
};

export function ConfigDetailsForm() {
  const { state, dispatch } = useConfigDetails();
  const { selectedNode } = state;

  if (!selectedNode) return null;

  const nodeType = (selectedNode.type ?? 'transformation') as string;
  const nodeData = selectedNode.data as Record<string, unknown>;
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
          <span>{NODE_ICONS[nodeType] ?? '🔧'}</span>
          <span>{nodeData.label as string ?? NODE_LABELS[nodeType] ?? 'Node Details'}</span>
        </div>
        <button type="button" className={styles.closeBtn} onClick={handleClose} aria-label="Close">
          ×
        </button>
      </div>

      {/* Body */}
      <div className={styles.formBody}>
        {/* Type field */}
        <div className={styles.configField}>
          <div className={styles.configLabel}>Type</div>
          <div className={styles.configValue}>{nodeData.stageType as string ?? nodeType}</div>
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
