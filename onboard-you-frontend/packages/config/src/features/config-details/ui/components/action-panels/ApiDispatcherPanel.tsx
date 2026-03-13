import { Box, Heading, Text, Table } from '@chakra-ui/react';
import { useTranslation } from 'react-i18next';
import { useAppSelector } from '@/store';
import { selectSettingsSchema } from '../../../state/configDetailsSlice';
import type { ActionEditorProps } from './registry';

export function ApiDispatcherPanel({ config }: ActionEditorProps) {
  const { t } = useTranslation();
  const settingsSchema = useAppSelector(selectSettingsSchema);

  // When auth_type is 'default', show the columns from org settings
  if (config.auth_type === 'default' && settingsSchema && Object.keys(settingsSchema).length > 0) {
    return (
      <Box as="section" data-testid="api-dispatcher-panel">
        <Heading as="h3" fontSize="md" fontWeight="600" mb="2">
          {t('flow.edit.columns', 'Columns')}
        </Heading>
        <Text fontSize="xs" color="gray.500" mb="3">
          {t('flow.edit.columnsDescription', 'These are the required columns from your organisation\'s settings.')}
        </Text>
        <Table.Root size="sm" variant="outline" borderWidth="0" borderRadius="10px" px="5px" overflow="hidden">
          <Table.Header>
            <Table.Row>
              <Table.ColumnHeader fontSize="xs" fontWeight="600" color="gray.500">
                {t('flow.edit.columnName', 'Name')}
              </Table.ColumnHeader>
              <Table.ColumnHeader fontSize="xs" fontWeight="600" color="gray.500" ps="0">
                {t('flow.edit.columnType', 'Type')}
              </Table.ColumnHeader>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {Object.entries(settingsSchema).map(([name, type]) => (
              <Table.Row key={name}>
                <Table.Cell fontSize="sm" fontWeight="500">{name}</Table.Cell>
                <Table.Cell fontSize="sm" color="gray.500" ps="0">{type}</Table.Cell>
              </Table.Row>
            ))}
          </Table.Body>
        </Table.Root>
      </Box>
    );
  }

  // Fallback: show raw config entries
  return (
    <Box data-testid="api-dispatcher-panel">
      {Object.entries(config).map(([key, value]) => (
        <Box key={key} mb="2">
          <Text fontSize="sm" fontWeight="600" mb="1">
            {key}
          </Text>
          <Text fontSize="sm" color="gray.600" whiteSpace="pre-wrap">
            {typeof value === 'string' || typeof value === 'number'
              ? String(value)
              : JSON.stringify(value, null, 2)}
          </Text>
        </Box>
      ))}
    </Box>
  );
}
