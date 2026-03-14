import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Text, Input, chakra } from '@chakra-ui/react';
import { CloseIcon, ArrowLeftIcon } from '@/shared/ui';
import { inputStyles, selectStyles } from './styles';

const StyledSelect = chakra('select');
const StyledButton = chakra('button');

interface MappingEditorProps {
  value: unknown;
  onChange: (v: Record<string, string>) => void;
  availableColumns: string[];
}

export function MappingEditor({ value, onChange, availableColumns }: MappingEditorProps) {
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
    <Box data-testid="mapping-editor">
      {entries.length > 0 && (
        <Flex gap="2" mb="2" px="1">
          <Text flex="1" fontSize="xs" fontWeight="600" color="gray.500">
            {t('flow.edit.mappingFrom', 'Current Name')}
          </Text>
          <Box w="4" />
          <Text flex="1" fontSize="xs" fontWeight="600" color="gray.500">
            {t('flow.edit.mappingTo', 'New Name')}
          </Text>
          <Box w="6" />
        </Flex>
      )}
      {entries.map(([key, val], idx) => (
        <Flex key={idx} align="center" gap="2" mb="2" data-testid={`mapping-row-${idx}`}>
          <StyledSelect
            flex="1"
            p="2"
            borderRadius="md"
            border="1px solid"
            {...selectStyles}
            value={key}
            onChange={(e: React.ChangeEvent<HTMLSelectElement>) =>
              handleKeyChange(key, e.target.value)
            }
          >
            <option value="">— Column —</option>
            {availableColumns.map((col) => (
              <option key={col} value={col}>
                {col}
              </option>
            ))}
            {key && !availableColumns.includes(key) && <option value={key}>{key}</option>}
          </StyledSelect>
          <ArrowLeftIcon size="0.85em" style={{ transform: 'rotate(180deg)' }} color="gray.400" />
          <Input
            flex="1"
            type="text"
            value={val}
            onChange={(e) => handleValueChange(key, e.target.value)}
            placeholder="new_name"
            {...inputStyles}
          />
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
            <CloseIcon size="0.75em" />
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
        data-testid="mapping-add-row"
      >
        + {t('flow.edit.addMapping', 'Add mapping')}
      </StyledButton>
    </Box>
  );
}
