import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Heading, Text, chakra } from '@chakra-ui/react';
import { selectStyles } from '../styles';
import type { ActionEditorProps } from './registry';

const StyledSelect = chakra('select');
const StyledButton = chakra('button');

const PII_STRATEGIES = [
  { value: 'Redact', label: 'Mask (show last 4)' },
  { value: 'Zero', label: 'Zero out' },
  { value: 'FullRedact', label: 'Fully redact' },
];

interface PiiColumn {
  name: string;
  strategy: unknown;
}

function getStrategyLabel(strat: unknown): string {
  if (typeof strat === 'string') return strat;
  if (typeof strat === 'object' && strat !== null) {
    return Object.keys(strat)[0] ?? 'Unknown';
  }
  return 'Unknown';
}

function buildStrategy(type: string): unknown {
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
}

export function PiiMaskingPanel({ config, onChange, availableColumns }: ActionEditorProps) {
  const { t } = useTranslation();
  const columns: PiiColumn[] = Array.isArray(config.columns)
    ? (config.columns as PiiColumn[])
    : [];

  const handleNameChange = useCallback(
    (idx: number, name: string) => {
      const updated = columns.map((c, i) => (i === idx ? { ...c, name } : c));
      onChange('columns', updated);
    },
    [columns, onChange],
  );

  const handleStratChange = useCallback(
    (idx: number, type: string) => {
      const updated = columns.map((c, i) =>
        i === idx ? { ...c, strategy: buildStrategy(type) } : c,
      );
      onChange('columns', updated);
    },
    [columns, onChange],
  );

  const handleRemove = useCallback(
    (idx: number) => {
      onChange(
        'columns',
        columns.filter((_, i) => i !== idx),
      );
    },
    [columns, onChange],
  );

  const handleAdd = useCallback(() => {
    onChange('columns', [
      ...columns,
      { name: '', strategy: { Redact: { keep_last: 4, mask_prefix: '***-**-' } } },
    ]);
  }, [columns, onChange]);

  return (
    <Box as="section" data-testid="pii-masking-panel">
      <Heading as="h3" fontSize="sm" fontWeight="600" mb="1">
        {t('flow.edit.piiColumns', 'Sensitive Columns')}
      </Heading>
      <Text fontSize="xs" color="gray.500" mb="3">
        {t('flow.edit.piiHint', 'Choose which columns to mask and how')}
      </Text>
      {columns.map((col, idx) => (
        <Flex key={idx} align="center" gap="2" mb="2" data-testid={`pii-row-${idx}`}>
          <StyledSelect
            flex="1"
            p="2"
            borderRadius="md"
            border="1px solid"
            {...selectStyles}
            value={col.name}
            onChange={(e: React.ChangeEvent<HTMLSelectElement>) =>
              handleNameChange(idx, e.target.value)
            }
          >
            <option value="">— Column —</option>
            {availableColumns.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
            {col.name && !availableColumns.includes(col.name) && (
              <option value={col.name}>{col.name}</option>
            )}
          </StyledSelect>
          <StyledSelect
            flex="1"
            p="2"
            borderRadius="md"
            border="1px solid"
            {...selectStyles}
            value={getStrategyLabel(col.strategy)}
            onChange={(e: React.ChangeEvent<HTMLSelectElement>) =>
              handleStratChange(idx, e.target.value)
            }
          >
            {PII_STRATEGIES.map((s) => (
              <option key={s.value} value={s.value}>
                {s.label}
              </option>
            ))}
          </StyledSelect>
          <StyledButton
            bg="transparent"
            border="none"
            cursor="pointer"
            color="gray.400"
            _hover={{ color: 'red.500' }}
            fontSize="sm"
            p="1"
            onClick={() => handleRemove(idx)}
            aria-label="Remove"
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
        onClick={handleAdd}
        data-testid="pii-add-column"
      >
        + {t('flow.edit.addColumn', 'Add column')}
      </StyledButton>
    </Box>
  );
}
