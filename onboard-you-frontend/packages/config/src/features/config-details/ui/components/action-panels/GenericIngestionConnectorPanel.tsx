import { useRef, useState, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { Box, Flex, Heading, Text, Button, chakra, Badge } from '@chakra-ui/react';
import { FolderOpenIcon } from '@/shared/ui';
import { validateGenericFile, uploadFileAndStartConversion } from '../../../services/genericUploadService';
import type { ActionEditorProps } from './registry';

const Label = chakra('label');

type UploadStatus = 'idle' | 'uploading' | 'done' | 'converting' | 'error';

const ACCEPTED = '.csv,.pdf,.xml,.json,.xlsx,.xls,.png,.jpg,.jpeg,.tiff,.tif';

/** Accepted label shown in the drop zone */
const ACCEPTED_LABEL = 'CSV, PDF, XML, Excel, JSON, or image';

export function GenericIngestionConnectorPanel({ config, onChange }: ActionEditorProps) {
  const { customerCompanyId } = useParams<{ customerCompanyId: string }>();
  const inputRef = useRef<HTMLInputElement>(null);

  const [status, setStatus] = useState<UploadStatus>('idle');
  const [uploadError, setUploadError] = useState<string | null>(null);
  const [isDragOver, setIsDragOver] = useState(false);

  const currentFilename = typeof config.filename === 'string' ? config.filename : '';
  const currentColumns = Array.isArray(config.columns) ? (config.columns as string[]) : [];
  const conversionStatus = typeof config.conversionStatus === 'string' ? config.conversionStatus : '';

  const handleFile = useCallback(
    async (file: File) => {
      const validationError = validateGenericFile(file);
      if (validationError) {
        setUploadError(validationError);
        setStatus('error');
        return;
      }

      setStatus('uploading');
      setUploadError(null);

      try {
        const { filename, columns, conversionStatus: cs } = await uploadFileAndStartConversion(
          customerCompanyId!,
          file,
        );
        onChange('filename', filename);
        onChange('columns', columns);
        onChange('conversionStatus', cs);
        setStatus(cs === 'converted' ? 'done' : 'done');
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Upload failed';
        setUploadError(message);
        setStatus('error');
      }
    },
    [customerCompanyId, onChange],
  );

  const handleInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) handleFile(file);
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
    <Box as="section" data-testid="generic-ingestion-connector-panel">
      {/* Current file info */}
      {currentFilename && (
        <Box mb="4" p="3" bg="blue.50" borderRadius="md" border="1px solid" borderColor="blue.200">
          <Flex align="center" justify="space-between" mb="1">
            <Heading as="h3" fontSize="xs" fontWeight="600" color="blue.700">
              Current file
            </Heading>
            {conversionStatus === 'not_needed' || conversionStatus === 'converted' ? (
              <Badge colorScheme="green" fontSize="xs">Ready</Badge>
            ) : null}
          </Flex>
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
          {conversionStatus === 'queued' && (
            <Text fontSize="xs" color="orange.600" mt="2">
              This file will be converted to CSV using AWS Textract before the pipeline runs.
              Column names will be auto-detected from the document.
            </Text>
          )}
        </Box>
      )}

      {/* Drop zone */}
      <Label htmlFor="generic-ingestion-input">
        <Box
          onDragOver={(e: React.DragEvent<HTMLDivElement>) => { e.preventDefault(); setIsDragOver(true); }}
          onDragLeave={() => setIsDragOver(false)}
          onDrop={handleDrop}
          border="2px dashed"
          borderColor={isDragOver ? 'purple.400' : isUploading ? 'gray.200' : 'gray.300'}
          borderRadius="md"
          p="5"
          textAlign="center"
          bg={isDragOver ? 'purple.50' : 'gray.50'}
          transition="all 0.15s"
          cursor={isUploading ? 'not-allowed' : 'pointer'}
          mb="3"
        >
          <FolderOpenIcon size="1.5em" />
          <Text fontSize="sm" fontWeight="500" color="gray.600" mt="1">
            {isUploading ? 'Uploading…' : `Drop any file here`}
          </Text>
          {!isUploading && (
            <Text fontSize="xs" color="gray.400" mt="1">
              {ACCEPTED_LABEL} · max 50 MB
            </Text>
          )}
        </Box>
      </Label>

      <input
        id="generic-ingestion-input"
        ref={inputRef}
        type="file"
        accept={ACCEPTED}
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
          ? 'Uploading…'
          : currentFilename
            ? 'Replace file'
            : 'Choose file'}
      </Button>

      {/* Status feedback */}
      {status === 'done' && (
        <Text fontSize="xs" color="green.600" mt="2" data-testid="generic-upload-success">
          ✓ File uploaded and columns detected
        </Text>
      )}
      {status === 'converting' && (
        <Text fontSize="xs" color="orange.600" mt="2" data-testid="generic-upload-converting">
          ⟳ File uploaded. Conversion queued — columns will be auto-detected before the pipeline runs.
        </Text>
      )}
      {status === 'error' && uploadError && (
        <Text fontSize="xs" color="red.600" mt="2" data-testid="generic-upload-error">
          {uploadError}
        </Text>
      )}
    </Box>
  );
}
