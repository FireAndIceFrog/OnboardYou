import { useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Text, Button, Badge, chakra } from '@chakra-ui/react';
import { FolderOpenIcon } from '@/shared/ui';
import { FieldError } from '../FieldError';
import type { ConnectorFormProps } from './types';

const ACCEPTED = '.csv,.pdf,.xml,.json,.xlsx,.xls,.png,.jpg,.jpeg,.tiff,.tif';

const Label = chakra('label');

export function GenericIngestionConnectorForm({ form, errors, onChange }: ConnectorFormProps) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);

  const { filename, uploadStatus, columns, conversionStatus } = form.genericIngestion;
  const isUploading = uploadStatus === 'uploading';

  return (
    <Box as="fieldset" border="none" p="0" m="0" mb="5">
      <Flex
        as="legend"
        align="center"
        gap="2"
        fontSize="sm"
        fontWeight="700"
        color="gray.700"
        mb="4"
        pb="2"
        borderBottom="1px solid"
        borderColor="gray.100"
      >
        <FolderOpenIcon size="1em" />
        <Text>{t('configDetails.connection.genericIngestion.uploadTitle')}</Text>
      </Flex>

      <Box mb="4">
        <Label htmlFor="conn-generic-file" fontSize="sm" fontWeight="600" display="block" mb="2">
          {t('configDetails.connection.genericIngestion.uploadLabel')}
        </Label>
        <input
          id="conn-generic-file"
          ref={inputRef}
          type="file"
          accept={ACCEPTED}
          style={{ position: 'absolute', width: 1, height: 1, overflow: 'hidden', clip: 'rect(0,0,0,0)' }}
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) onChange({ type: 'file', file });
            e.target.value = '';
          }}
        />
        <Flex align="center" gap="3" wrap="wrap">
          <Button
            variant="outline"
            size="sm"
            type="button"
            disabled={isUploading}
            onClick={() => inputRef.current?.click()}
          >
            {isUploading
              ? t('configDetails.connection.genericIngestion.uploadTitle') + '…'
              : filename
                ? 'Replace file'
                : 'Choose file'}
          </Button>
          {filename && (
            <Flex align="center" gap="2">
              <Text fontSize="sm" color="gray.600">{filename}</Text>
              {conversionStatus === 'queued' && (
                <Badge colorScheme="orange" fontSize="xs">Converting…</Badge>
              )}
              {conversionStatus === 'not_needed' && (
                <Badge colorScheme="green" fontSize="xs">Ready</Badge>
              )}
            </Flex>
          )}
        </Flex>
        <FieldError id="generic-file-error" error={errors['genericIngestion.filename']} />
        <Text fontSize="xs" color="gray.400" mt="1">
          {t('configDetails.connection.genericIngestion.uploadHint')}
        </Text>
        {conversionStatus === 'queued' && (
          <Text fontSize="xs" color="orange.600" mt="1">
            {t('configDetails.connection.genericIngestion.convertingHint')}
          </Text>
        )}
      </Box>

      {columns.length > 0 && (
        <Box mb="4">
          <Text fontSize="sm" fontWeight="600" mb="2">
            {t('configDetails.connection.genericIngestion.discoveredColumns')}
          </Text>
          <Flex wrap="wrap" gap="2">
            {columns.map((col) => (
              <Box
                key={col}
                px="2.5"
                py="1"
                borderRadius="full"
                bg="blue.50"
                border="1px solid"
                borderColor="blue.200"
                fontSize="xs"
                color="blue.700"
              >
                {col}
              </Box>
            ))}
          </Flex>
          <Text fontSize="xs" color="gray.400" mt="1">
            {t('configDetails.connection.genericIngestion.columnsHint', { count: columns.length })}
          </Text>
        </Box>
      )}
    </Box>
  );
}
