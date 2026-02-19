import { useMemo } from 'react';
import { Box, Flex, Text } from '@chakra-ui/react';
import { ACTION_FIELD_SCHEMAS } from '../../../domain/actionCatalog';
import { RESPONSE_GROUP_OPTIONS } from '../../../domain/types';
import { FieldEditor } from '../FieldEditor';
import type { ActionEditorProps } from './registry';

export function WorkdayResponseGroupPanel({ config, onChange, availableColumns }: ActionEditorProps) {
  const fieldSchemas = useMemo(
    () => ACTION_FIELD_SCHEMAS.workday_hris_connector ?? [],
    [],
  );

  const group = (typeof config.response_group === 'object' && config.response_group !== null
    ? config.response_group
    : {}) as Record<string, boolean>;

  const toggle = (field: string) => {
    onChange('response_group', { ...group, [field]: !group[field] });
  };

  return (
    <Box data-testid="workday-response-group-panel">
      {/* Render all standard fields except response_group via generic FieldEditor */}
      {fieldSchemas
        .filter((schema) => schema.key !== 'response_group')
        .map((schema) => (
          <Box key={schema.key} mb="4">
            <Text fontSize="sm" fontWeight="600" mb="1">
              {schema.label}
            </Text>
            {schema.hint && (
              <Text fontSize="xs" color="gray.500" mb="2">
                {schema.hint}
              </Text>
            )}
            <FieldEditor
              schema={schema}
              value={config[schema.key]}
              onChange={onChange}
              availableColumns={availableColumns}
            />
          </Box>
        ))}

      {/* Custom response_group toggle chips */}
      <Box>
        <Text fontSize="sm" fontWeight="600" mb="1">
          Response Groups
        </Text>
        <Text fontSize="xs" color="gray.500" mb="2">
          Data sections to include in the Workday response
        </Text>
        <Flex wrap="wrap" gap="2">
          {RESPONSE_GROUP_OPTIONS.map((opt) => (
            <Box
              as="label"
              key={opt.value}
              display="flex"
              alignItems="center"
              gap="1.5"
              px="2.5"
              py="1"
              borderRadius="full"
              border="1px solid"
              borderColor={group[opt.value] ? 'blue.400' : 'gray.200'}
              bg={group[opt.value] ? 'blue.50' : 'white'}
              cursor="pointer"
              fontSize="xs"
              transition="all 0.15s"
              _hover={{ borderColor: 'blue.300' }}
            >
              <input
                type="checkbox"
                checked={!!group[opt.value]}
                onChange={() => toggle(opt.value)}
                style={{ display: 'none' }}
                data-testid={`response-group-${opt.value}`}
              />
              <Text>{opt.label}</Text>
            </Box>
          ))}
        </Flex>
      </Box>
    </Box>
  );
}
