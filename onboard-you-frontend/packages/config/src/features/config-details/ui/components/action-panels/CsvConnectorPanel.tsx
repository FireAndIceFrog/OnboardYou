import { useRef, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { useParams } from 'react-router-dom';
import { Box, Flex, Heading, Text, Button, chakra } from '@chakra-ui/react';
import { FolderOpenIcon } from '@/shared/ui';
import { validateCsvFile, uploadCsvAndDiscoverColumns } from '../../../services/csvUploadService';
import type { ActionEditorProps } from './registry';

const Label = chakra('label');

type UploadStatus = 'idle' | 'uploading' | 'done' | 'error';

export function CsvConnectorPanel({ config, onChange }: ActionEditorProps) {
  const { t } = useTranslation();
  const { customerCompanyId } = useParams<{ customerCompanyId: string }>();
  const inputRef = useRef<HTMLInputElement>(null);

  const [status, setStatus] = useState<UploadStatus>('idle');
  const [uploadError, setUploadError] = useState<string | null>(null);
  const [isDragOver, setIsDragOver] = useState(false);

  const currentFilename = typeof config.filename === 'string' ? config.filename : '';
  const currentColumns = Array.isArray(config.columns) ? (config.columns as string[]) : [];

  const handleFile = useCallback(
    async (file: File) => {
      const validationError = validateCsvFile(file);
      if (validationError) {
        setUploadError(validationError);
        setStatus('error');
        return;
      }

      setStatus('uploading');
      setUploadError(null);

      try {
        const { filename, columns } = await uploadCsvAndDiscoverColumns(
          customerCompanyId!,
          file,
        );
        onChange('filename', filename);
        onChange('columns', columns);
        setStatus('done');
      } catch (err) {
        const message = err instanceof Error ? err.message : t('configDetails.panels.csv.uploadFailed');
        setUploadError(message);
        setStatus('error');
      }
    },
    [customerCompanyId, onChange, t],
  );

  const handleInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) handleFile(file);
      // Reset so the same file can be re-selected
      e.target.value = '';
    },
    [handleFile],
  );

  const handleDrop = useCallback(
    (e: React.DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      setIsDragOver(false);
      const file = e.dataTransfer.files?.[0];
      if (file) handleFile(file);
    },
    [handleFile],
  );

  const isUploading = status === 'uploading';

  return (
    <Box as="section" data-testid="csv-connector-panel">
      {/* Current file */}
      {currentFilename && (
        <Box mb="4" p="3" bg="blue.50" borderRadius="md" border="1px solid" borderColor="blue.200">
          <Heading as="h3" fontSize="xs" fontWeight="600" color="blue.700" mb="1">
            {t('configDetails.panels.csv.currentFile')}
          </Heading>
          <Text fontSize="sm" color="blue.800" fontFamily="mono">
            {currentFilename}
          </Text>
          {currentColumns.length > 0 && (
            <Flex wrap="wrap" gap="1.5" mt="2">
              {currentColumns.map((col) => (
                <Box
                  key={col}
                  px="2"
                  py="0.5"
                  borderRadius="full"
                  bg="blue.100"
                  border="1px solid"
                  borderColor="blue.300"
                  fontSize="xs"
                  color="blue.700"
                >
                  {col}
                </Box>
              ))}
            </Flex>
          )}
        </Box>
      )}

      {/* Drop zone */}
      <Label htmlFor="csv-panel-input">
        <Box
          onDragOver={(e: React.DragEvent<HTMLDivElement>) => { e.preventDefault(); setIsDragOver(true); }}
          onDragLeave={() => setIsDragOver(false)}
          onDrop={handleDrop}
          border="2px dashed"
          borderColor={isDragOver ? 'blue.400' : isUploading ? 'gray.200' : 'gray.300'}
          borderRadius="md"
          p="5"
          textAlign="center"
          bg={isDragOver ? 'blue.50' : 'gray.50'}
          transition="all 0.15s"
          cursor={isUploading ? 'not-allowed' : 'pointer'}
          mb="3"
        >
          <FolderOpenIcon size="1.5em" />
          <Text fontSize="sm" fontWeight="500" color="gray.600">
            {isUploading
              ? t('configDetails.panels.csv.uploading')
              : t('configDetails.panels.csv.dropHint')}
          </Text>
          {!isUploading && (
            <Text fontSize="xs" color="gray.400" mt="1">
              {t('configDetails.panels.csv.orClick')}
            </Text>
          )}
        </Box>
      </Label>

      <input
        id="csv-panel-input"
        ref={inputRef}
        type="file"
        accept=".csv"
        style={{ position: 'absolute', width: 1, height: 1, overflow: 'hidden', clip: 'rect(0,0,0,0)' }}
        disabled={isUploading}
        onChange={handleInputChange}
      />

      <Button
        variant="outline"
        size="sm"
        width="100%"
        type="button"
        disabled={isUploading}
        onClick={() => inputRef.current?.click()}
      >
        {isUploading
          ? t('configDetails.panels.csv.uploading')
          : currentFilename
            ? t('configDetails.panels.csv.replaceFile')
            : t('configDetails.panels.csv.chooseFile')}
      </Button>

      {/* Status feedback */}
      {status === 'done' && (
        <Text fontSize="xs" color="green.600" mt="2" data-testid="csv-upload-success">
          ✓ {t('configDetails.panels.csv.uploadSuccess')}
        </Text>
      )}
      {status === 'error' && uploadError && (
        <Text fontSize="xs" color="red.600" mt="2" data-testid="csv-upload-error">
          {uploadError}
        </Text>
      )}
    </Box>
  );
}
