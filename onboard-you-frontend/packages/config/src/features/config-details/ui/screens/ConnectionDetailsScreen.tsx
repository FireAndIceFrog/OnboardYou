import { useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Heading, Text, Button, chakra } from '@chakra-ui/react';
import { HR_SYSTEMS, RESPONSE_GROUP_OPTIONS, SAGE_HR_HISTORY_OPTIONS } from '../../domain/types';
import type { WorkdayFields, SageHrFields } from '../../domain/types';
import { useConnectionForm } from '../../state/useConnectionForm';
import { FieldError } from '../components';
import { FormField } from '@/shared/ui';

const Label = chakra('label');
const StyledButton = chakra('button');

/* ── Field descriptors ──────────────────────────────────── */

interface FieldDef {
  id: string;
  fieldKey: string;
  validationKey?: string;
  type?: string;
  min?: number;
  max?: number;
}

const WORKDAY_CREDENTIAL_FIELDS: FieldDef[] = [
  { id: 'conn-tenant-url', fieldKey: 'tenantUrl', validationKey: 'workday.tenantUrl', type: 'url' },
  { id: 'conn-tenant-id', fieldKey: 'tenantId', validationKey: 'workday.tenantId' },
];

const WORKDAY_INLINE_CREDENTIALS: FieldDef[] = [
  { id: 'conn-username', fieldKey: 'username', validationKey: 'workday.username' },
  { id: 'conn-password', fieldKey: 'password', validationKey: 'workday.password', type: 'password' },
];

const WORKDAY_QUERY_FIELDS: FieldDef[] = [
  { id: 'conn-worker-count', fieldKey: 'workerCountLimit', type: 'number', min: 1, max: 999 },
];

const SAGE_HR_CREDENTIAL_FIELDS: FieldDef[] = [
  { id: 'conn-sage-subdomain', fieldKey: 'subdomain', validationKey: 'sageHr.subdomain' },
  { id: 'conn-sage-api-token', fieldKey: 'apiToken', validationKey: 'sageHr.apiToken', type: 'password' },
];

/* ── Component ──────────────────────────────────────────── */

export function ConnectionDetailsScreen() {
  const {
    form,
    errors,
    isValid,
    activeGroups,
    handleSystemSelect,
    handleChange,
    handleWorkdayChange,
    handleSageHrChange,
    handleSageHrHistoryToggle,
    handleCsvFileSelect,
    handleResponseGroupToggle,
    handleNext,
    handleBack,
    validateField,
  } = useConnectionForm();
  const { t } = useTranslation();
  const csvInputRef = useRef<HTMLInputElement>(null);

  const renderFields = (
    fields: FieldDef[],
    i18nPrefix: string,
    getValue: (key: string) => string | number,
    getHandler: (key: string) => (e: React.ChangeEvent<HTMLInputElement>) => void,
  ) =>
    fields.map((def) => (
      <FormField
        key={def.id}
        id={def.id}
        label={t(`${i18nPrefix}.${def.fieldKey}`)}
        placeholder={t(`${i18nPrefix}.${def.fieldKey}Placeholder`, { defaultValue: '' }) || undefined}
        helperText={t(`${i18nPrefix}.${def.fieldKey}Hint`, { defaultValue: '' }) || undefined}
        type={def.type}
        min={def.min}
        max={def.max}
        value={getValue(def.fieldKey)}
        onChange={getHandler(def.fieldKey)}
        onBlur={def.validationKey ? () => validateField(def.validationKey!) : undefined}
        error={def.validationKey ? errors[def.validationKey] : undefined}
      />
    ));

  const workdayValue = (key: string) => form.workday[key as keyof WorkdayFields] as string | number;
  const workdayHandler = (key: string) => handleWorkdayChange(key as keyof WorkdayFields);
  const sageHrValue = (key: string) => form.sageHr[key as keyof SageHrFields] as string;
  const sageHrHandler = (key: string) => handleSageHrChange(key as keyof SageHrFields);

  return (
    <Box maxW="680px" mx="auto" py="8" px="6">
      <Box as="form" bg="white" borderRadius="lg" border="1px solid" borderColor="gray.200" p="6" shadow="sm" onSubmit={(e: React.FormEvent) => e.preventDefault()}>
        <Heading size="lg" mb="1">{t('configDetails.connection.title')}</Heading>
        <Text fontSize="sm" color="gray.500" mb="6">{t('configDetails.connection.subtitle')}</Text>

        {/* System selector */}
        <Box mb="5">
          <Text fontSize="sm" fontWeight="600" mb="2">{t('configDetails.connection.hrSystem')}</Text>
          <Box display="grid" gridTemplateColumns="repeat(auto-fill, minmax(140px, 1fr))" gap="3" border={errors.system ? '2px solid' : 'none'} borderColor="red.300" borderRadius="lg" p={errors.system ? '2' : '0'}>
            {HR_SYSTEMS.map((sys) => (
              <StyledButton
                key={sys.id}
                type="button"
                display="flex"
                flexDirection="column"
                alignItems="center"
                gap="2"
                p="4"
                borderRadius="lg"
                border="2px solid"
                borderColor={form.system === sys.id ? 'blue.500' : 'gray.200'}
                bg={form.system === sys.id ? 'blue.50' : 'white'}
                cursor="pointer"
                transition="all 0.15s"
                _hover={{ borderColor: 'blue.300' }}
                onClick={() => handleSystemSelect(sys.id)}
              >
                <Text fontSize="2xl">{sys.icon}</Text>
                <Text fontSize="sm" fontWeight="500">{t(sys.nameKey)}</Text>
              </StyledButton>
            ))}
          </Box>
          <FieldError id="system-error" error={errors.system} />
        </Box>

        {/* Display name */}
        {form.system && (
          <Box mb="5">
            <FormField
              id="conn-display-name"
              label={t('configDetails.connection.displayName')}
              placeholder={t('configDetails.connection.displayNamePlaceholder')}
              helperText={t('configDetails.connection.displayNameHint')}
              value={form.displayName}
              onChange={handleChange('displayName')}
            />
          </Box>
        )}

        {/* Workday-specific fields */}
        {form.system === 'workday' && (
          <>
            <Box as="fieldset" border="none" p="0" m="0" mb="5">
              <Flex as="legend" align="center" gap="2" fontSize="sm" fontWeight="700" color="gray.700" mb="4" pb="2" borderBottom="1px solid" borderColor="gray.100">
                <Text>🔐</Text>
                <Text>{t('configDetails.connection.workday.credentialsTitle')}</Text>
              </Flex>

              <Flex direction="column" gap="4">
                {renderFields(WORKDAY_CREDENTIAL_FIELDS, 'configDetails.connection.workday', workdayValue, workdayHandler)}
              </Flex>

              <Flex gap="4" mt="4">
                {WORKDAY_INLINE_CREDENTIALS.map((def) => (
                  <Box flex="1" key={def.id}>
                    <FormField
                      id={def.id}
                      label={t(`configDetails.connection.workday.${def.fieldKey}`)}
                      placeholder={t(`configDetails.connection.workday.${def.fieldKey}Placeholder`, { defaultValue: '' }) || undefined}
                      type={def.type}
                      value={workdayValue(def.fieldKey)}
                      onChange={workdayHandler(def.fieldKey)}
                      onBlur={def.validationKey ? () => validateField(def.validationKey!) : undefined}
                      error={def.validationKey ? errors[def.validationKey] : undefined}
                    />
                  </Box>
                ))}
              </Flex>
            </Box>

            <Box as="fieldset" border="none" p="0" m="0" mb="5">
              <Flex as="legend" align="center" gap="2" fontSize="sm" fontWeight="700" color="gray.700" mb="4" pb="2" borderBottom="1px solid" borderColor="gray.100">
                <Text>⚙️</Text>
                <Text>{t('configDetails.connection.workday.queryOptionsTitle')}</Text>
              </Flex>

              <Flex direction="column" gap="4">
                {renderFields(WORKDAY_QUERY_FIELDS, 'configDetails.connection.workday', workdayValue, workdayHandler)}
              </Flex>

              <Box mt="4">
                <Text fontSize="sm" fontWeight="600" mb="2">{t('configDetails.connection.workday.responseGroups')}</Text>
                <Flex wrap="wrap" gap="2" border={errors['workday.responseGroup'] ? '2px solid' : 'none'} borderColor="red.300" borderRadius="lg" p={errors['workday.responseGroup'] ? '2' : '0'}>
                  {RESPONSE_GROUP_OPTIONS.map((opt) => (
                    <StyledButton
                      key={opt.value}
                      type="button"
                      px="3"
                      py="1.5"
                      borderRadius="full"
                      border="1px solid"
                      borderColor={activeGroups.has(opt.value) ? 'blue.400' : 'gray.200'}
                      bg={activeGroups.has(opt.value) ? 'blue.50' : 'white'}
                      color={activeGroups.has(opt.value) ? 'blue.700' : 'gray.600'}
                      fontSize="sm"
                      cursor="pointer"
                      transition="all 0.15s"
                      _hover={{ borderColor: 'blue.300' }}
                      aria-pressed={activeGroups.has(opt.value)}
                      onClick={() => handleResponseGroupToggle(opt.value)}
                    >
                      {t(opt.labelKey)}
                    </StyledButton>
                  ))}
                </Flex>
                <FieldError id="response-group-error" error={errors['workday.responseGroup']} />
                <Text fontSize="xs" color="gray.400" mt="1">{t('configDetails.connection.workday.responseGroupsHint')}</Text>
              </Box>
            </Box>
          </>
        )}

        {/* Sage HR-specific fields */}
        {form.system === 'sage_hr' && (
          <>
            <Box as="fieldset" border="none" p="0" m="0" mb="5">
              <Flex as="legend" align="center" gap="2" fontSize="sm" fontWeight="700" color="gray.700" mb="4" pb="2" borderBottom="1px solid" borderColor="gray.100">
                <Text>🔐</Text>
                <Text>{t('configDetails.connection.sageHr.credentialsTitle')}</Text>
              </Flex>

              <Flex direction="column" gap="4">
                {renderFields(SAGE_HR_CREDENTIAL_FIELDS, 'configDetails.connection.sageHr', sageHrValue, sageHrHandler)}
              </Flex>
            </Box>

            <Box as="fieldset" border="none" p="0" m="0" mb="5">
              <Flex as="legend" align="center" gap="2" fontSize="sm" fontWeight="700" color="gray.700" mb="4" pb="2" borderBottom="1px solid" borderColor="gray.100">
                <Text>⚙️</Text>
                <Text>{t('configDetails.connection.sageHr.historyOptionsTitle')}</Text>
              </Flex>

              <Box mb="4">
                <Text fontSize="sm" fontWeight="600" mb="2">{t('configDetails.connection.sageHr.historyOptions')}</Text>
                <Flex wrap="wrap" gap="2">
                  {SAGE_HR_HISTORY_OPTIONS.map((opt) => (
                    <StyledButton
                      key={opt.value}
                      type="button"
                      px="3"
                      py="1.5"
                      borderRadius="full"
                      border="1px solid"
                      borderColor={form.sageHr[opt.value] ? 'blue.400' : 'gray.200'}
                      bg={form.sageHr[opt.value] ? 'blue.50' : 'white'}
                      color={form.sageHr[opt.value] ? 'blue.700' : 'gray.600'}
                      fontSize="sm"
                      cursor="pointer"
                      transition="all 0.15s"
                      _hover={{ borderColor: 'blue.300' }}
                      aria-pressed={!!form.sageHr[opt.value]}
                      onClick={() => handleSageHrHistoryToggle(opt.value)}
                    >
                      {t(opt.labelKey)}
                    </StyledButton>
                  ))}
                </Flex>
                <Text fontSize="xs" color="gray.400" mt="1">{t('configDetails.connection.sageHr.historyOptionsHint')}</Text>
              </Box>
            </Box>
          </>
        )}

        {/* CSV-specific fields */}
        {form.system === 'csv' && (
          <Box as="fieldset" border="none" p="0" m="0" mb="5">
            <Flex as="legend" align="center" gap="2" fontSize="sm" fontWeight="700" color="gray.700" mb="4" pb="2" borderBottom="1px solid" borderColor="gray.100">
              <Text>📁</Text>
              <Text>{t('configDetails.connection.csv.uploadTitle')}</Text>
            </Flex>

            <Box mb="4">
              <Label htmlFor="conn-csv-file" fontSize="sm" fontWeight="600" display="block" mb="2">{t('configDetails.connection.csv.uploadLabel')}</Label>
              <input
                id="conn-csv-file"
                ref={csvInputRef}
                type="file"
                accept=".csv"
                style={{ position: 'absolute', width: 1, height: 1, overflow: 'hidden', clip: 'rect(0,0,0,0)' }}
                onChange={(e) => {
                  const file = e.target.files?.[0];
                  if (file) handleCsvFileSelect(file);
                }}
              />
              <Flex align="center" gap="3">
                <Button variant="outline" size="sm" type="button" disabled={form.csv.uploadStatus === 'uploading' || form.csv.uploadStatus === 'discovering'} onClick={() => csvInputRef.current?.click()}>
                  {form.csv.uploadStatus === 'uploading' || form.csv.uploadStatus === 'discovering' ? t('configDetails.connection.csv.uploading') : t('configDetails.connection.csv.chooseFile')}
                </Button>
                {form.csv.filename && <Text fontSize="sm" color="gray.600">{form.csv.filename}</Text>}
              </Flex>
              <FieldError id="csv-file-error" error={errors['csv.filename']} />
              <Text fontSize="xs" color="gray.400" mt="1">{t('configDetails.connection.csv.uploadHint')}</Text>
            </Box>

            {form.csv.columns.length > 0 && (
              <Box mb="4">
                <Text fontSize="sm" fontWeight="600" mb="2">{t('configDetails.connection.csv.discoveredColumns')}</Text>
                <Flex wrap="wrap" gap="2">
                  {form.csv.columns.map((col) => (
                    <Box key={col} px="2.5" py="1" borderRadius="full" bg="blue.50" border="1px solid" borderColor="blue.200" fontSize="xs" color="blue.700">{col}</Box>
                  ))}
                </Flex>
                <Text fontSize="xs" color="gray.400" mt="1">{t('configDetails.connection.csv.columnsHint', { count: form.csv.columns.length })}</Text>
              </Box>
            )}
          </Box>
        )}
      </Box>

      {/* Actions */}
      <Flex justify="space-between" mt="6">
        <Button variant="outline" size="md" onClick={handleBack}>
          {t('configDetails.connection.backButton')}
        </Button>
        <Button colorPalette="blue" size="md" disabled={!isValid} onClick={handleNext}>
          {t('configDetails.connection.nextButton')}
        </Button>
      </Flex>
    </Box>
  );
}

/** @deprecated Use `ConnectionDetailsScreen` instead. Kept for backward compatibility during migration. */
export const ConnectionDetailsPage = ConnectionDetailsScreen;
