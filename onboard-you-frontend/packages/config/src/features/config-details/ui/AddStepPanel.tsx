import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { useAppDispatch } from '@/store';
import type { ActionConfig } from '@/generated/api';
import { ACTION_CATALOG, type ActionCatalogEntry } from '../domain/actionCatalog';
import { addFlowAction, setAddStepPanelOpen } from '../state/configDetailsSlice';
import styles from './AddStepPanel.module.scss';

const LOGIC_STEPS = ACTION_CATALOG.filter((a) => a.category === 'logic');
const EGRESS_STEPS = ACTION_CATALOG.filter((a) => a.category === 'egress');

interface AddStepPanelProps {
  onClose: () => void;
}

export function AddStepPanel({ onClose }: AddStepPanelProps) {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();

  const handleAdd = useCallback(
    (entry: ActionCatalogEntry) => {
      const action: ActionConfig = {
        id: `step-${Date.now()}`,
        action_type: entry.actionType,
        config: structuredClone(entry.defaultConfig),
      };
      dispatch(addFlowAction(action));
      dispatch(setAddStepPanelOpen(false));
    },
    [dispatch],
  );

  return (
    <aside className={styles.panel} aria-label={t('flow.addStep.title')}>
      <div className={styles.header}>
        <h3 className={styles.title}>{t('flow.addStep.title')}</h3>
        <button type="button" className={styles.closeBtn} onClick={onClose} aria-label={t('common.close')}>
          ✕
        </button>
      </div>

      <p className={styles.subtitle}>{t('flow.addStep.subtitle')}</p>

      <div className={styles.body}>
        {/* Logic / Transform */}
        <section>
          <h4 className={styles.sectionLabel}>
            <span className={styles.sectionIcon}>⚙️</span>
            {t('flow.addStep.sections.transform')}
          </h4>
          <ul className={styles.catalog} role="list">
            {LOGIC_STEPS.map((entry) => (
              <li key={entry.actionType}>
                <button
                  type="button"
                  className={styles.catalogItem}
                  onClick={() => handleAdd(entry)}
                >
                  <span className={styles.itemIcon}>{entry.icon}</span>
                  <div className={styles.itemText}>
                    <span className={styles.itemLabel}>{entry.label}</span>
                    <span className={styles.itemDesc}>{entry.description}</span>
                  </div>
                </button>
              </li>
            ))}
          </ul>
        </section>

        {/* Egress / Destinations */}
        <section>
          <h4 className={styles.sectionLabel}>
            <span className={styles.sectionIcon}>📤</span>
            {t('flow.addStep.sections.destination')}
          </h4>
          <ul className={styles.catalog} role="list">
            {EGRESS_STEPS.map((entry) => (
              <li key={entry.actionType}>
                <button
                  type="button"
                  className={styles.catalogItem}
                  onClick={() => handleAdd(entry)}
                >
                  <span className={styles.itemIcon}>{entry.icon}</span>
                  <div className={styles.itemText}>
                    <span className={styles.itemLabel}>{entry.label}</span>
                    <span className={styles.itemDesc}>{entry.description}</span>
                  </div>
                </button>
              </li>
            ))}
          </ul>
        </section>
      </div>
    </aside>
  );
}
