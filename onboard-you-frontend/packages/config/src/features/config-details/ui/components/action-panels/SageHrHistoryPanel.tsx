import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Text } from '@chakra-ui/react';
import { ACTION_FIELD_SCHEMAS } from '../../../domain/actionCatalog';
import { SAGE_HR_HISTORY_OPTIONS } from '../../../domain/types';
import { FieldEditor } from '../FieldEditor';
import type { ActionEditorProps } from './registry';

export function SageHrHistoryPanel({ config, onChange, availableColumns }: ActionEditorProps) {
  const { t } = useTranslation();
  const fieldSchemas = useMemo(
    () => ACTION_FIELD_SCHEMAS.sage_hr_connector ?? [],
    [],
  );

  const toggle = (configKey: string) => {
    onChange(configKey, !config[configKey]);
  };

  return (
    <Box data-testid="sage-hr-history-panel">
      {/* Render standard fields (subdomain, api_token) via generic FieldEditor */}
      {fieldSchemas.map((schema) => (
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

      {/* Custom history toggle chips */}
      <Box>
        <Text fontSize="sm" fontWeight="600" mb="1">
          {t('configDetails.panels.historyOptions.title')}
        </Text>
        <Text fontSize="xs" color="gray.500" mb="2">
          {t('configDetails.panels.historyOptions.hint')}
        </Text>
        <Flex wrap="wrap" gap="2">
          {SAGE_HR_HISTORY_OPTIONS.map((opt) => (
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
              borderColor={config[opt.configKey] ? 'blue.400' : 'gray.200'}
              bg={config[opt.configKey] ? 'blue.50' : 'white'}
              cursor="pointer"
              fontSize="xs"
              transition="all 0.15s"
              _hover={{ borderColor: 'blue.300' }}
            >
              <input
                type="checkbox"
                checked={!!config[opt.configKey]}
                onChange={() => toggle(opt.configKey)}
                style={{ display: 'none' }}
                data-testid={`history-${opt.configKey}`}
              />
              <Text>{t(opt.labelKey)}</Text>
            </Box>
          ))}
        </Flex>
      </Box>
    </Box>
  );
}
