import { useCallback, useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { useAppDispatch, useAppSelector } from '@/store';
import type { RootState } from '@/store';
import type { ActionConfigPayload, ActionType } from '@/generated/api';
import { businessLabel, actionCategory } from '@/shared/domain/types';
import { ACTION_FIELD_SCHEMAS, ACTION_CATALOG, type FieldSchema } from '../domain/actionCatalog';
import {
  selectSelectedNode,
  selectAvailableColumnsForAction,
  deselectNode,
  removeFlowAction,
  updateFlowActionConfig,
} from '../state/configDetailsSlice';
import styles from './ActionEditPanel.module.scss';

/* ── Category icons ────────────────────────────────────────── */
const CATEGORY_ICONS: Record<string, string> = {
  ingestion: '📥',
  logic: '⚙️',
  egress: '📤',
};

/* ── helpers ───────────────────────────────────────────────── */

/** Get a nested value from an object by a dotted key */
function getField(config: Record<string, unknown>, key: string): unknown {
  return config[key];
}

/** Set a top-level field and return the new config */
function setField(
  config: Record<string, unknown>,
  key: string,
  value: unknown,
): Record<string, unknown> {
  return { ...config, [key]: value };
}

/* ── Sub-components ────────────────────────────────────────── */

interface FieldProps {
  schema: FieldSchema;
  value: unknown;
  onChange: (key: string, value: unknown) => void;
  availableColumns: string[];
}

/** Render a single field based on its schema type */
function FieldEditor({ schema, value, onChange, availableColumns }: FieldProps) {
  const handleText = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => onChange(schema.key, e.target.value),
    [onChange, schema.key],
  );

  const handleNumber = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => onChange(schema.key, Number(e.target.value) || 0),
    [onChange, schema.key],
  );

  const handleSelect = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => onChange(schema.key, e.target.value),
    [onChange, schema.key],
  );

  switch (schema.type) {
    case 'readonly': {
      const display = Array.isArray(value)
        ? (value as string[]).join(', ')
        : typeof value === 'object' && value !== null
          ? JSON.stringify(value, null, 2)
          : String(value ?? '—');
      return <div className={styles.readonlyValue}>{display}</div>;
    }

    case 'text':
      return (
        <input
          type="text"
          className={styles.textInput}
          value={String(value ?? '')}
          onChange={handleText}
          placeholder={schema.placeholder}
        />
      );

    case 'number':
      return (
        <input
          type="number"
          className={styles.textInput}
          value={value === undefined || value === null ? '' : String(value)}
          onChange={handleNumber}
          placeholder={schema.placeholder}
        />
      );

    case 'select':
      return (
        <select
          className={styles.selectInput}
          value={String(value ?? '')}
          onChange={handleSelect}
        >
          {schema.options?.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      );

    /* ── Single column select ──────────────────────────────── */
    case 'column-select':
      return (
        <select
          className={styles.selectInput}
          value={String(value ?? '')}
          onChange={(e) => onChange(schema.key, e.target.value)}
        >
          <option value="">— Select a column —</option>
          {availableColumns.map((col) => (
            <option key={col} value={col}>{col}</option>
          ))}
        </select>
      );

    /* ── Multi column select (checkbox list) ───────────────── */
    case 'column-multi': {
      const selected = new Set<string>(
        Array.isArray(value) ? (value as string[]) : [],
      );
      const toggle = (col: string) => {
        const next = new Set(selected);
        if (next.has(col)) next.delete(col);
        else next.add(col);
        onChange(schema.key, [...next]);
      };
      return (
        <div className={styles.columnMulti}>
          {availableColumns.length === 0 && (
            <span className={styles.columnMultiEmpty}>
              No columns available yet — save or validate first
            </span>
          )}
          {availableColumns.map((col) => (
            <label key={col} className={styles.columnChip}>
              <input
                type="checkbox"
                checked={selected.has(col)}
                onChange={() => toggle(col)}
              />
              <span>{col}</span>
            </label>
          ))}
        </div>
      );
    }

    /* ── Legacy columns (comma-separated, kept as fallback) ── */
    case 'columns': {
      const arr = Array.isArray(value) ? (value as string[]) : [];
      return (
        <input
          type="text"
          className={styles.textInput}
          value={arr.join(', ')}
          onChange={(e) => onChange(schema.key, e.target.value.split(',').map(s => s.trim()).filter(Boolean))}
          placeholder={schema.placeholder}
        />
      );
    }

    case 'mapping':
      return <MappingEditor value={value} onChange={(v) => onChange(schema.key, v)} availableColumns={availableColumns} />;

    default:
      return <div className={styles.readonlyValue}>{String(value ?? '—')}</div>;
  }
}

/** Key-value editor for rename_column mapping */
function MappingEditor({
  value,
  onChange,
  availableColumns,
}: {
  value: unknown;
  onChange: (v: Record<string, string>) => void;
  availableColumns: string[];
}) {
  const { t } = useTranslation();
  const mapping = (value && typeof value === 'object' && !Array.isArray(value)
    ? value
    : {}) as Record<string, string>;

  const entries = Object.entries(mapping);

  const handleKeyChange = useCallback(
    (oldKey: string, newKey: string) => {
      const updated: Record<string, string> = {};
      for (const [k, v] of Object.entries(mapping)) {
        updated[k === oldKey ? newKey : k] = v;
      }
      onChange(updated);
    },
    [mapping, onChange],
  );

  const handleValueChange = useCallback(
    (key: string, newValue: string) => {
      onChange({ ...mapping, [key]: newValue });
    },
    [mapping, onChange],
  );

  const handleRemoveRow = useCallback(
    (key: string) => {
      const { [key]: _, ...rest } = mapping;
      onChange(rest);
    },
    [mapping, onChange],
  );

  const handleAddRow = useCallback(() => {
    onChange({ ...mapping, '': '' });
  }, [mapping, onChange]);

  return (
    <div className={styles.mappingEditor}>
      {entries.length > 0 && (
        <div className={styles.mappingHeader}>
          <span>{t('flow.edit.mappingFrom', 'Current Name')}</span>
          <span>{t('flow.edit.mappingTo', 'New Name')}</span>
          <span />
        </div>
      )}
      {entries.map(([key, val], idx) => (
        <div key={idx} className={styles.mappingRow}>
          <select
            className={styles.selectInput}
            value={key}
            onChange={(e) => handleKeyChange(key, e.target.value)}
          >
            <option value="">— Column —</option>
            {availableColumns.map((col) => (
              <option key={col} value={col}>{col}</option>
            ))}
            {/* Keep the current value visible even if not in available columns */}
            {key && !availableColumns.includes(key) && (
              <option value={key}>{key}</option>
            )}
          </select>
          <span className={styles.mappingArrow}>→</span>
          <input
            type="text"
            className={styles.textInput}
            value={val}
            onChange={(e) => handleValueChange(key, e.target.value)}
            placeholder="new_name"
          />
          <button
            type="button"
            className={styles.removeRowBtn}
            onClick={() => handleRemoveRow(key)}
            aria-label={t('flow.edit.removeMapping', 'Remove')}
          >
            ✕
          </button>
        </div>
      ))}
      <button type="button" className={styles.addRowBtn} onClick={handleAddRow}>
        + {t('flow.edit.addMapping', 'Add mapping')}
      </button>
    </div>
  );
}

/* ── PII Masking sub-editor ────────────────────────────────── */

const PII_STRATEGIES = [
  { value: 'Redact', label: 'Mask (show last 4)' },
  { value: 'Zero', label: 'Zero out' },
  { value: 'FullRedact', label: 'Fully redact' },
];

interface PiiColumn {
  name: string;
  strategy: unknown;
}

function PiiMaskingEditor({
  value,
  onChange,
  availableColumns,
}: {
  value: unknown;
  onChange: (key: string, v: unknown) => void;
  availableColumns: string[];
}) {
  const { t } = useTranslation();
  const columns: PiiColumn[] = Array.isArray(value)
    ? (value as PiiColumn[])
    : [];

  const getStrategyLabel = (strat: unknown): string => {
    if (typeof strat === 'string') return strat;
    if (typeof strat === 'object' && strat !== null) {
      return Object.keys(strat)[0] ?? 'Unknown';
    }
    return 'Unknown';
  };

  const buildStrategy = (type: string): unknown => {
    switch (type) {
      case 'Redact':
        return { Redact: { keep_last: 4, mask_prefix: '***-**-' } };
      case 'Zero':
        return 'Zero';
      case 'FullRedact':
        return { Redact: { keep_last: 0, mask_prefix: '****' } };
      default:
        return 'Zero';
    }
  };

  const handleNameChange = (idx: number, name: string) => {
    const updated = columns.map((c, i) => (i === idx ? { ...c, name } : c));
    onChange('columns', updated);
  };

  const handleStratChange = (idx: number, type: string) => {
    const updated = columns.map((c, i) =>
      i === idx ? { ...c, strategy: buildStrategy(type) } : c,
    );
    onChange('columns', updated);
  };

  const handleRemove = (idx: number) => {
    onChange('columns', columns.filter((_, i) => i !== idx));
  };

  const handleAdd = () => {
    onChange('columns', [
      ...columns,
      { name: '', strategy: { Redact: { keep_last: 4, mask_prefix: '***-**-' } } },
    ]);
  };

  return (
    <div className={styles.piiEditor}>
      <label className={styles.fieldLabel}>
        {t('flow.edit.piiColumns', 'Sensitive Columns')}
      </label>
      <p className={styles.fieldHint}>
        {t('flow.edit.piiHint', 'Choose which columns to mask and how')}
      </p>
      {columns.map((col, idx) => (
        <div key={idx} className={styles.piiRow}>
          <select
            className={styles.selectInput}
            value={col.name}
            onChange={(e) => handleNameChange(idx, e.target.value)}
          >
            <option value="">— Column —</option>
            {availableColumns.map((c) => (
              <option key={c} value={c}>{c}</option>
            ))}
            {col.name && !availableColumns.includes(col.name) && (
              <option value={col.name}>{col.name}</option>
            )}
          </select>
          <select
            className={styles.selectInput}
            value={getStrategyLabel(col.strategy)}
            onChange={(e) => handleStratChange(idx, e.target.value)}
          >
            {PII_STRATEGIES.map((s) => (
              <option key={s.value} value={s.value}>
                {s.label}
              </option>
            ))}
          </select>
          <button
            type="button"
            className={styles.removeRowBtn}
            onClick={() => handleRemove(idx)}
            aria-label="Remove"
          >
            ✕
          </button>
        </div>
      ))}
      <button type="button" className={styles.addRowBtn} onClick={handleAdd}>
        + {t('flow.edit.addColumn', 'Add column')}
      </button>
    </div>
  );
}

/* ── Main panel ────────────────────────────────────────────── */

export function ActionEditPanel() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const selectedNode = useAppSelector(selectSelectedNode);
  const [confirmRemove, setConfirmRemove] = useState(false);

  const handleClose = useCallback(() => {
    dispatch(deselectNode());
    setConfirmRemove(false);
  }, [dispatch]);

  const nodeData = selectedNode?.data as Record<string, unknown> | undefined;
  const actionId = (nodeData?.actionId as string) ?? selectedNode?.id;
  const actionType = (nodeData?.actionType as ActionType) ?? '';
  const category = (nodeData?.category as string) ?? 'logic';
  const config = nodeData?.config as ActionConfigPayload | undefined;
  const label = (nodeData?.label as string) ?? businessLabel(actionType);

  // Derive available columns from validation result (columns_after of preceding step)
  const availableColumns = useAppSelector(
    (state: RootState) => selectAvailableColumnsForAction(state, actionId ?? ''),
  );

  const catalogEntry = useMemo(
    () => ACTION_CATALOG.find((a) => a.actionType === actionType),
    [actionType],
  );

  const fieldSchemas = useMemo(
    () => ACTION_FIELD_SCHEMAS[actionType as ActionType] ?? [],
    [actionType],
  );

  const isIngestion = category === 'ingestion';

  const handleFieldChange = useCallback(
    (key: string, value: unknown) => {
      if (!actionId || !config) return;

      // config can be a string (api_dispatcher 'Default') — coerce to object
      const configObj =
        typeof config === 'object' && config !== null
          ? (config as Record<string, unknown>)
          : {};

      const updated = setField(configObj, key, value);
      dispatch(updateFlowActionConfig({ actionId, config: updated as ActionConfigPayload }));
    },
    [actionId, config, dispatch],
  );

  const handleRemove = useCallback(() => {
    if (!actionId) return;
    if (!confirmRemove) {
      setConfirmRemove(true);
      return;
    }
    dispatch(removeFlowAction(actionId));
    setConfirmRemove(false);
  }, [actionId, confirmRemove, dispatch]);

  if (!selectedNode) return null;

  const configObj =
    typeof config === 'object' && config !== null
      ? (config as Record<string, unknown>)
      : null;

  return (
    <div className={styles.panel}>
      {/* Header */}
      <div className={styles.panelHeader}>
        <div className={styles.headerInfo}>
          <span className={styles.headerIcon}>{CATEGORY_ICONS[category] ?? '🔧'}</span>
          <div>
            <h3 className={styles.headerTitle}>{label}</h3>
            <span className={styles.headerCategory}>
              {t(`configDetails.form.categoryLabels.${category}`, category)}
            </span>
          </div>
        </div>
        <button
          type="button"
          className={styles.closeBtn}
          onClick={handleClose}
          aria-label={t('common.close', 'Close')}
        >
          ✕
        </button>
      </div>

      {/* Description */}
      {catalogEntry?.description && (
        <div className={styles.description}>{catalogEntry.description}</div>
      )}

      {/* Fields */}
      <div className={styles.panelBody}>
        {/* Standard field-schema based fields */}
        {fieldSchemas.length > 0 && configObj ? (
          fieldSchemas.map((schema) => {
            // PII masking gets its own editor
            if (actionType === 'pii_masking' && schema.key === 'columns') {
              return (
                <PiiMaskingEditor
                  key="pii"
                  value={getField(configObj, 'columns')}
                  onChange={handleFieldChange}
                  availableColumns={availableColumns}
                />
              );
            }

            return (
              <div key={schema.key} className={styles.fieldGroup}>
                <label className={styles.fieldLabel}>{schema.label}</label>
                {schema.hint && (
                  <p className={styles.fieldHint}>{schema.hint}</p>
                )}
                <FieldEditor
                  schema={schema}
                  value={getField(configObj, schema.key)}
                  onChange={handleFieldChange}
                  availableColumns={availableColumns}
                />
              </div>
            );
          })
        ) : configObj ? (
          /* Fallback: render config keys as readonly if no schema defined */
          Object.entries(configObj).map(([key, value]) => (
            <div key={key} className={styles.fieldGroup}>
              <label className={styles.fieldLabel}>{key}</label>
              <div className={styles.readonlyValue}>
                {typeof value === 'string' || typeof value === 'number'
                  ? String(value)
                  : JSON.stringify(value, null, 2)}
              </div>
            </div>
          ))
        ) : (
          /* String config (e.g. api_dispatcher 'Default') */
          <div className={styles.fieldGroup}>
            <label className={styles.fieldLabel}>
              {t('flow.edit.configuration', 'Configuration')}
            </label>
            <div className={styles.readonlyValue}>{String(config ?? '—')}</div>
          </div>
        )}
      </div>

      {/* Footer with remove button */}
      {!isIngestion && (
        <div className={styles.panelFooter}>
          <button
            type="button"
            className={`${styles.removeBtn} ${confirmRemove ? styles.removeBtnConfirm : ''}`}
            onClick={handleRemove}
          >
            {confirmRemove
              ? t('flow.edit.confirmRemove', '⚠️ Click again to confirm removal')
              : t('flow.edit.removeStep', '🗑️ Remove this step')}
          </button>
        </div>
      )}
    </div>
  );
}
