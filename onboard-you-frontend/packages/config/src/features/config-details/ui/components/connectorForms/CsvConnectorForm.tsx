import { useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Text, Button, chakra } from '@chakra-ui/react';
import { FolderIcon } from '@/shared/ui';
import { FieldError } from '../FieldError';
import type { ConnectorFormProps } from './types';

const Label = chakra('label');

export function CsvConnectorForm({ form, errors, onChange }: ConnectorFormProps) {
  const { t } = useTranslation();
  const csvInputRef = useRef<HTMLInputElement>(null);

  return (
    <Box as="fieldset" border="none" p="0" m="0" mb="5">
      <Flex as="legend" align="center" gap="2" fontSize="sm" fontWeight="700" color="gray.700" mb="4" pb="2" borderBottom="1px solid" borderColor="gray.100">
        <FolderIcon size="1em" />
        <Text>{t('configDetails.connection.csv.uploadTitle')}</Text>
      </Flex>

      <Box mb="4">
        <Label htmlFor="conn-csv-file" fontSize="sm" fontWeight="600" display="block" mb="2">
          {t('configDetails.connection.csv.uploadLabel')}
        </Label>
        <input
          id="conn-csv-file"
          ref={csvInputRef}
          type="file"
          accept=".csv"
          style={{ position: 'absolute', width: 1, height: 1, overflow: 'hidden', clip: 'rect(0,0,0,0)' }}
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) onChange({ type: 'file', file });
          }}
        />
        <Flex align="center" gap="3">
          <Button
            variant="outline"
            size="sm"
            type="button"
            disabled={form.csv.uploadStatus === 'uploading' || form.csv.uploadStatus === 'discovering'}
            onClick={() => csvInputRef.current?.click()}
          >
            {form.csv.uploadStatus === 'uploading' || form.csv.uploadStatus === 'discovering'
              ? t('configDetails.connection.csv.uploading')
              : t('configDetails.connection.csv.chooseFile')}
          </Button>
          {form.csv.filename && (
            <Text fontSize="sm" color="gray.600">{form.csv.filename}</Text>
          )}
        </Flex>
        <FieldError id="csv-file-error" error={errors['csv.filename']} />
        <Text fontSize="xs" color="gray.400" mt="1">{t('configDetails.connection.csv.uploadHint')}</Text>
      </Box>

      {form.csv.columns.length > 0 && (
        <Box mb="4">
          <Text fontSize="sm" fontWeight="600" mb="2">{t('configDetails.connection.csv.discoveredColumns')}</Text>
          <Flex wrap="wrap" gap="2">
            {form.csv.columns.map((col) => (
              <Box key={col} px="2.5" py="1" borderRadius="full" bg="blue.50" border="1px solid" borderColor="blue.200" fontSize="xs" color="blue.700">
                {col}
              </Box>
            ))}
          </Flex>
          <Text fontSize="xs" color="gray.400" mt="1">
            {t('configDetails.connection.csv.columnsHint', { count: form.csv.columns.length })}
          </Text>
        </Box>
      )}
    </Box>
  );
}
