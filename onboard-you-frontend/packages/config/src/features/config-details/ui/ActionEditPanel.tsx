import { useCallback, useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Text, Heading, Input, chakra } from '@chakra-ui/react';
import { useAppDispatch, useAppSelector } from '@/store';
import type { RootState } from '@/store';
import type { ActionConfigPayload, ActionType } from '@/generated/api';
import { businessLabel, actionCategory } from '@/shared/domain/types';
import { ACTION_FIELD_SCHEMAS, ACTION_CATALOG, type FieldSchema } from '../domain/actionCatalog';
import { RESPONSE_GROUP_OPTIONS } from '../domain/types';
import {
  selectSelectedNode,
  selectAvailableColumnsForAction,
  deselectNode,
  removeFlowAction,
  updateFlowActionConfig,
} from '../state/configDetailsSlice';

const StyledSelect = chakra('select');
const StyledButton = chakra('button');

/* ── Category icons ────────────────────────────────────────── */
const CATEGORY_ICONS: Record<string, string> = {
  ingestion: '📥',
  logic: '⚙️',
  egress: '📤',
};

/* ── helpers ───────────────────────────────────────────────── */

function getField(config: Record<string, unknown>, key: string): unknown {
  return config[key];
}

function setField(
  config: Record<string, unknown>,
  key: string,
  value: unknown,
): Record<string, unknown> {
  return { ...config, [key]: value };
}

/* ── Shared input styles ───────────────────────────────────── */
const inputStyles = {
  fontSize: 'sm',
  borderColor: 'gray.200',
  bg: 'white',
  _focus: { borderColor: 'blue.500', boxShadow: '0 0 0 1px var(--chakra-colors-blue-500)' },
} as const;

const selectStyles = {
  ...inputStyles,
  cursor: 'pointer',
} as const;

/* ── Sub-components ────────────────────────────────────────── */

interface FieldProps {
  schema: FieldSchema;
  value: unknown;
  onChange: (key: string, value: unknown) => void;
  availableColumns: string[];
}

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
      return <Text fontSize="sm" color="gray.600" whiteSpace="pre-wrap">{display}</Text>;
    }

    case 'text':
      return (
        <Input
          type="text"
          value={String(value ?? '')}
          onChange={handleText}
          placeholder={schema.placeholder}
          {...inputStyles}
        />
      );

    case 'number':
      return (
        <Input
          type="number"
          value={value === undefined || value === null ? '' : String(value)}
          onChange={handleNumber}
          placeholder={schema.placeholder}
          {...inputStyles}
        />
      );

    case 'select':
      return (
        <StyledSelect w="full" p="2" borderRadius="md" border="1px solid" {...selectStyles} value={String(value ?? '')} onChange={handleSelect}>
          {schema.options?.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </StyledSelect>
      );

    case 'column-select':
      return (
        <StyledSelect w="full" p="2" borderRadius="md" border="1px solid" {...selectStyles} value={String(value ?? '')} onChange={(e: React.ChangeEvent<HTMLSelectElement>) => onChange(schema.key, e.target.value)}>
          <option value="">— Select a column —</option>
          {availableColumns.map((col) => (
            <option key={col} value={col}>{col}</option>
          ))}
        </StyledSelect>
      );

    case 'column-multi': {
      const selected = new Set<string>(Array.isArray(value) ? (value as string[]) : []);
      const toggle = (col: string) => {
        const next = new Set(selected);
        if (next.has(col)) next.delete(col);
        else next.add(col);
        onChange(schema.key, [...next]);
      };
      return (
        <Flex wrap="wrap" gap="2">
          {availableColumns.length === 0 && (
            <Text fontSize="xs" color="gray.400">No columns available yet — save or validate first</Text>
          )}
          {availableColumns.map((col) => (
            <Box as="label" key={col} display="flex" alignItems="center" gap="1.5" px="2.5" py="1" borderRadius="full" border="1px solid" borderColor={selected.has(col) ? 'blue.400' : 'gray.200'} bg={selected.has(col) ? 'blue.50' : 'white'} cursor="pointer" fontSize="xs" transition="all 0.15s" _hover={{ borderColor: 'blue.300' }}>
              <input type="checkbox" checked={selected.has(col)} onChange={() => toggle(col)} style={{ display: 'none' }} />
              <Text>{col}</Text>
            </Box>
          ))}
        </Flex>
      );
    }

    case 'columns': {
      const arr = Array.isArray(value) ? (value as string[]) : [];
      return (
        <Input
          type="text"
          value={arr.join(', ')}
          onChange={(e) => onChange(schema.key, e.target.value.split(',').map(s => s.trim()).filter(Boolean))}
          placeholder={schema.placeholder}
          {...inputStyles}
        />
      );
    }

    case 'mapping':
      return <MappingEditor value={value} onChange={(v) => onChange(schema.key, v)} availableColumns={availableColumns} />;

    default:
      return <Text fontSize="sm" color="gray.600">{String(value ?? '—')}</Text>;
  }
}

/* ── Mapping Editor ────────────────────────────────────────── */

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
    <Box>
      {entries.length > 0 && (
        <Flex gap="2" mb="2" px="1">
          <Text flex="1" fontSize="xs" fontWeight="600" color="gray.500">{t('flow.edit.mappingFrom', 'Current Name')}</Text>
          <Box w="4" />
          <Text flex="1" fontSize="xs" fontWeight="600" color="gray.500">{t('flow.edit.mappingTo', 'New Name')}</Text>
          <Box w="6" />
        </Flex>
      )}
      {entries.map(([key, val], idx) => (
        <Flex key={idx} align="center" gap="2" mb="2">
          <StyledSelect flex="1" p="2" borderRadius="md" border="1px solid" {...selectStyles} value={key} onChange={(e: React.ChangeEvent<HTMLSelectElement>) => handleKeyChange(key, e.target.value)}>
            <option value="">— Column —</option>
            {availableColumns.map((col) => (
              <option key={col} value={col}>{col}</option>
            ))}
            {key && !availableColumns.includes(key) && (
              <option value={key}>{key}</option>
            )}
          </StyledSelect>
          <Text color="gray.400">→</Text>
          <Input flex="1" type="text" value={val} onChange={(e) => handleValueChange(key, e.target.value)} placeholder="new_name" {...inputStyles} />
          <StyledButton
            bg="transparent"
            border="none"
            cursor="pointer"
            color="gray.400"
            _hover={{ color: 'red.500' }}
            fontSize="sm"
            p="1"
            onClick={() => handleRemoveRow(key)}
            aria-label={t('flow.edit.removeMapping', 'Remove')}
          >
            ✕
          </StyledButton>
        </Flex>
      ))}
      <StyledButton
        bg="transparent"
        border="none"
        cursor="pointer"
        color="blue.500"
        fontSize="sm"
        fontWeight="500"
        _hover={{ color: 'blue.600' }}
        p="0"
        onClick={handleAddRow}
      >
        + {t('flow.edit.addMapping', 'Add mapping')}
      </StyledButton>
    </Box>
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
  const columns: PiiColumn[] = Array.isArray(value) ? (value as PiiColumn[]) : [];

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
    <Box>
      <Text fontSize="sm" fontWeight="600" mb="1">{t('flow.edit.piiColumns', 'Sensitive Columns')}</Text>
      <Text fontSize="xs" color="gray.500" mb="3">{t('flow.edit.piiHint', 'Choose which columns to mask and how')}</Text>
      {columns.map((col, idx) => (
        <Flex key={idx} align="center" gap="2" mb="2">
          <StyledSelect flex="1" p="2" borderRadius="md" border="1px solid" {...selectStyles} value={col.name} onChange={(e: React.ChangeEvent<HTMLSelectElement>) => handleNameChange(idx, e.target.value)}>
            <option value="">— Column —</option>
            {availableColumns.map((c) => (
              <option key={c} value={c}>{c}</option>
            ))}
            {col.name && !availableColumns.includes(col.name) && (
              <option value={col.name}>{col.name}</option>
            )}
          </StyledSelect>
          <StyledSelect flex="1" p="2" borderRadius="md" border="1px solid" {...selectStyles} value={getStrategyLabel(col.strategy)} onChange={(e: React.ChangeEvent<HTMLSelectElement>) => handleStratChange(idx, e.target.value)}>
            {PII_STRATEGIES.map((s) => (
              <option key={s.value} value={s.value}>{s.label}</option>
            ))}
          </StyledSelect>
          <StyledButton bg="transparent" border="none" cursor="pointer" color="gray.400" _hover={{ color: 'red.500' }} fontSize="sm" p="1" onClick={() => handleRemove(idx)} aria-label="Remove">
            ✕
          </StyledButton>
        </Flex>
      ))}
      <StyledButton bg="transparent" border="none" cursor="pointer" color="blue.500" fontSize="sm" fontWeight="500" _hover={{ color: 'blue.600' }} p="0" onClick={handleAdd}>
        + {t('flow.edit.addColumn', 'Add column')}
      </StyledButton>
    </Box>
  );
}

/* ── Workday Response Group toggle editor ──────────────────── */

function WorkdayResponseGroupEditor({
  value,
  onChange,
}: {
  value: unknown;
  onChange: (key: string, v: unknown) => void;
}) {
  const group = (typeof value === 'object' && value !== null ? value : {}) as Record<string, boolean>;

  const toggle = (field: string) => {
    onChange('response_group', { ...group, [field]: !group[field] });
  };

  return (
    <Flex wrap="wrap" gap="2">
      {RESPONSE_GROUP_OPTIONS.map((opt) => (
        <Box as="label" key={opt.value} display="flex" alignItems="center" gap="1.5" px="2.5" py="1" borderRadius="full" border="1px solid" borderColor={group[opt.value] ? 'blue.400' : 'gray.200'} bg={group[opt.value] ? 'blue.50' : 'white'} cursor="pointer" fontSize="xs" transition="all 0.15s" _hover={{ borderColor: 'blue.300' }}>
          <input type="checkbox" checked={!!group[opt.value]} onChange={() => toggle(opt.value)} style={{ display: 'none' }} />
          <Text>{opt.label}</Text>
        </Box>
      ))}
    </Flex>
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
    <Box
      position="absolute"
      top="4"
      right="4"
      w="380px"
      bg="white"
      borderRadius="lg"
      border="1px solid"
      borderColor="gray.200"
      shadow="xl"
      zIndex="10"
      display="flex"
      flexDirection="column"
      maxH="calc(100vh - 200px)"
    >
      {/* Header */}
      <Flex align="center" justify="space-between" px="4" py="3" borderBottom="1px solid" borderColor="gray.100" bg="gray.50" borderTopRadius="lg">
        <Flex align="center" gap="2">
          <Text fontSize="lg">{CATEGORY_ICONS[category] ?? '🔧'}</Text>
          <Box>
            <Heading size="sm">{label}</Heading>
            <Text fontSize="xs" color="gray.500">
              {t(`configDetails.form.categoryLabels.${category}`, category)}
            </Text>
          </Box>
        </Flex>
        <StyledButton
          onClick={handleClose}
          aria-label={t('common.close', 'Close')}
          cursor="pointer"
          fontSize="lg"
          color="gray.400"
          _hover={{ color: 'gray.600' }}
          bg="transparent"
          border="none"
          p="0"
        >
          ✕
        </StyledButton>
      </Flex>

      {/* Description */}
      {catalogEntry?.description && (
        <Text fontSize="sm" color="gray.600" px="4" py="2" borderBottom="1px solid" borderColor="gray.50" bg="blue.50">
          {catalogEntry.description}
        </Text>
      )}

      {/* Fields */}
      <Box flex="1" overflowY="auto" p="4" display="flex" flexDirection="column" gap="4">
        {fieldSchemas.length > 0 && configObj ? (
          fieldSchemas.map((schema) => {
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

            if (actionType === 'workday_hris_connector' && schema.key === 'response_group') {
              return (
                <Box key={schema.key}>
                  <Text fontSize="sm" fontWeight="600" mb="1">{schema.label}</Text>
                  {schema.hint && <Text fontSize="xs" color="gray.500" mb="2">{schema.hint}</Text>}
                  <WorkdayResponseGroupEditor
                    value={getField(configObj, 'response_group')}
                    onChange={handleFieldChange}
                  />
                </Box>
              );
            }

            return (
              <Box key={schema.key}>
                <Text fontSize="sm" fontWeight="600" mb="1">{schema.label}</Text>
                {schema.hint && <Text fontSize="xs" color="gray.500" mb="2">{schema.hint}</Text>}
                <FieldEditor
                  schema={schema}
                  value={getField(configObj, schema.key)}
                  onChange={handleFieldChange}
                  availableColumns={availableColumns}
                />
              </Box>
            );
          })
        ) : configObj ? (
          Object.entries(configObj).map(([key, value]) => (
            <Box key={key}>
              <Text fontSize="sm" fontWeight="600" mb="1">{key}</Text>
              <Text fontSize="sm" color="gray.600" whiteSpace="pre-wrap">
                {typeof value === 'string' || typeof value === 'number'
                  ? String(value)
                  : JSON.stringify(value, null, 2)}
              </Text>
            </Box>
          ))
        ) : (
          <Box>
            <Text fontSize="sm" fontWeight="600" mb="1">
              {t('flow.edit.configuration', 'Configuration')}
            </Text>
            <Text fontSize="sm" color="gray.600">{String(config ?? '—')}</Text>
          </Box>
        )}
      </Box>

      {/* Footer */}
      {!isIngestion && (
        <Box px="4" py="3" borderTop="1px solid" borderColor="gray.100">
          <StyledButton
            w="full"
            py="2"
            borderRadius="md"
            border="1px solid"
            borderColor={confirmRemove ? 'red.300' : 'gray.200'}
            bg={confirmRemove ? 'red.50' : 'white'}
            color={confirmRemove ? 'red.600' : 'gray.600'}
            cursor="pointer"
            fontSize="sm"
            fontWeight="500"
            transition="all 0.15s"
            _hover={{ borderColor: 'red.300', bg: 'red.50', color: 'red.600' }}
            onClick={handleRemove}
          >
            {confirmRemove
              ? t('flow.edit.confirmRemove', '⚠️ Click again to confirm removal')
              : t('flow.edit.removeStep', '🗑️ Remove this step')}
          </StyledButton>
        </Box>
      )}
    </Box>
  );
}
