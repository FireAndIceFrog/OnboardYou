import { useTranslation } from 'react-i18next';
import { useAppDispatch, useAppSelector } from '@/store';
import { selectSelectedNode, deselectNode } from '../state/configDetailsSlice';
import { businessLabel } from '@/shared/domain/types';
import type { ActionConfigPayload } from '@/generated/api';
import styles from './ConfigDetailsForm.module.scss';

const CATEGORY_ICONS: Record<string, string> = {
  ingestion: '📥',
  logic: '⚙️',
  egress: '📤',
};

export function ConfigDetailsForm() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const selectedNode = useAppSelector(selectSelectedNode);

  if (!selectedNode) return null;

  const nodeData = selectedNode.data as Record<string, unknown>;
  const category = (nodeData.category as string) ?? 'logic';
  const actionType = (nodeData.actionType as string) ?? '';
  const config = nodeData.config as ActionConfigPayload | undefined;

  const handleClose = () => {
    dispatch(deselectNode());
  };

  const configEntries = config && typeof config === 'object' ? Object.entries(config) : [];

  return (
    <div className={styles.formPanel}>
      {/* Header */}
      <div className={styles.formHeader}>
        <div className={styles.formTitle}>
          <span>{CATEGORY_ICONS[category] ?? '🔧'}</span>
          <span>{(nodeData.label as string) ?? t(`configDetails.form.categoryLabels.${category}`, t('configDetails.form.nodeDetails'))}</span>
        </div>
        <button type="button" className={styles.closeBtn} onClick={handleClose} aria-label={t('configDetails.form.close')}>
          ×
        </button>
      </div>

      {/* Body */}
      <dl className={styles.formBody}>
        {/* Action type field */}
        <div className={styles.configField}>
          <dt className={styles.configLabel}>{t('configDetails.form.stepType')}</dt>
          <dd className={styles.configValue}>{businessLabel(actionType)}</dd>
        </div>

        {/* Category field */}
        <div className={styles.configField}>
          <dt className={styles.configLabel}>{t('configDetails.form.category')}</dt>
          <dd className={styles.configValue}>{t(`configDetails.form.categoryLabels.${category}`, category)}</dd>
        </div>

        {/* Config key-value pairs */}
        {configEntries.length > 0 ? (
          configEntries.map(([key, value]) => (
            <div key={key} className={styles.configField}>
              <dt className={styles.configLabel}>{key}</dt>
              <dd className={styles.configValue}>
                {typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean'
                  ? String(value)
                  : JSON.stringify(value)}
              </dd>
            </div>
          ))
        ) : (
          <div className={styles.configField}>
            <dt className={styles.configLabel}>{t('configDetails.form.configuration')}</dt>
            <dd className={styles.configValue}>{t('configDetails.form.noConfigData')}</dd>
          </div>
        )}

        {/* JSON fallback block */}
        {config && Object.keys(config).length > 0 && (
          <div className={styles.jsonBlock}>
            <pre>{JSON.stringify(config, null, 2)}</pre>
          </div>
        )}
      </dl>
    </div>
  );
}
