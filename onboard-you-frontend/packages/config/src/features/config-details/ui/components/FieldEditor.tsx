import { useCallback } from 'react';
import { Box, Flex, Text, Input, chakra } from '@chakra-ui/react';
import type { FieldSchema } from '../../domain/actionCatalog';
import { inputStyles, selectStyles } from './styles';
import { MappingEditor } from './MappingEditor';

const StyledSelect = chakra('select');

export interface FieldEditorProps {
  schema: FieldSchema;
  value: unknown;
  onChange: (key: string, value: unknown) => void;
  availableColumns: string[];
}

export function FieldEditor({ schema, value, onChange, availableColumns }: FieldEditorProps) {
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
      return (
        <Text fontSize="sm" color="gray.600" whiteSpace="pre-wrap" data-testid={`field-readonly-${schema.key}`}>
          {display}
        </Text>
      );
    }

    case 'text':
      return (
        <Input
          type="text"
          value={String(value ?? '')}
          onChange={handleText}
          placeholder={schema.placeholder}
          data-testid={`field-text-${schema.key}`}
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
          data-testid={`field-number-${schema.key}`}
          {...inputStyles}
        />
      );

    case 'select':
      return (
        <StyledSelect
          w="full"
          p="2"
          borderRadius="md"
          border="1px solid"
          {...selectStyles}
          value={String(value ?? '')}
          onChange={handleSelect}
          data-testid={`field-select-${schema.key}`}
        >
          {schema.options?.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </StyledSelect>
      );

    case 'column-select':
      return (
        <StyledSelect
          w="full"
          p="2"
          borderRadius="md"
          border="1px solid"
          {...selectStyles}
          value={String(value ?? '')}
          onChange={(e: React.ChangeEvent<HTMLSelectElement>) => onChange(schema.key, e.target.value)}
          data-testid={`field-column-select-${schema.key}`}
        >
          <option value="">— Select a column —</option>
          {availableColumns.map((col) => (
            <option key={col} value={col}>
              {col}
            </option>
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
        <Flex wrap="wrap" gap="2" data-testid={`field-column-multi-${schema.key}`}>
          {availableColumns.length === 0 && (
            <Text fontSize="xs" color="gray.400">
              No columns available yet — save or validate first
            </Text>
          )}
          {availableColumns.map((col) => (
            <Box
              as="label"
              key={col}
              display="flex"
              alignItems="center"
              gap="1.5"
              px="2.5"
              py="1"
              borderRadius="full"
              border="1px solid"
              borderColor={selected.has(col) ? 'blue.400' : 'gray.200'}
              bg={selected.has(col) ? 'blue.50' : 'white'}
              cursor="pointer"
              fontSize="xs"
              transition="all 0.15s"
              _hover={{ borderColor: 'blue.300' }}
            >
              <input
                type="checkbox"
                checked={selected.has(col)}
                onChange={() => toggle(col)}
                style={{ display: 'none' }}
              />
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
          onChange={(e) =>
            onChange(
              schema.key,
              e.target.value
                .split(',')
                .map((s) => s.trim())
                .filter(Boolean),
            )
          }
          placeholder={schema.placeholder}
          data-testid={`field-columns-${schema.key}`}
          {...inputStyles}
        />
      );
    }

    case 'mapping':
      return (
        <MappingEditor
          value={value}
          onChange={(v) => onChange(schema.key, v)}
          availableColumns={availableColumns}
        />
      );

    default:
      return (
        <Text fontSize="sm" color="gray.600">
          {String(value ?? '—')}
        </Text>
      );
  }
}
