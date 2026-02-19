import { useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Heading, Text, Input, Button, chakra } from '@chakra-ui/react';
import { HR_SYSTEMS, RESPONSE_GROUP_OPTIONS } from '../../domain/types';
import { useConnectionForm } from '../../state/useConnectionForm';
import { FieldError } from '../components';
import { inputStyles } from '../components/styles';

const Label = chakra('label');
const StyledButton = chakra('button');

const invalidInputProps = {
  ...inputStyles,
  borderColor: 'red.400' as const,
  _focus: { borderColor: 'red.500', boxShadow: '0 0 0 1px var(--chakra-colors-red-500)' },
};

function getInputProps(errorKey?: string) {
  return errorKey ? invalidInputProps : inputStyles;
}

export function ConnectionDetailsScreen() {
  const {
    form,
    errors,
    isValid,
    activeGroups,
    handleSystemSelect,
    handleChange,
    handleWorkdayChange,
    handleCsvFileSelect,
    handleResponseGroupToggle,
    handleNext,
    handleBack,
    validateField,
  } = useConnectionForm();
  const { t } = useTranslation();
  const csvInputRef = useRef<HTMLInputElement>(null);

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
                <Text fontSize="sm" fontWeight="500">{sys.name}</Text>
              </StyledButton>
            ))}
          </Box>
          <FieldError id="system-error" error={errors.system} />
        </Box>

        {/* Display name */}
        {form.system && (
          <Box mb="5">
            <Label htmlFor="conn-display-name" fontSize="sm" fontWeight="600" display="block" mb="1">{t('configDetails.connection.displayName')}</Label>
            <Input id="conn-display-name" type="text" placeholder={t('configDetails.connection.displayNamePlaceholder')} value={form.displayName} onChange={handleChange('displayName')} {...inputStyles} />
            <Text fontSize="xs" color="gray.400" mt="1">{t('configDetails.connection.displayNameHint')}</Text>
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

              <Box mb="4">
                <Label htmlFor="conn-tenant-url" fontSize="sm" fontWeight="600" display="block" mb="1">{t('configDetails.connection.workday.tenantUrl')}</Label>
                <Input id="conn-tenant-url" type="url" placeholder={t('configDetails.connection.workday.tenantUrlPlaceholder')} value={form.workday.tenantUrl} onChange={handleWorkdayChange('tenantUrl')} onBlur={() => validateField('workday.tenantUrl')} aria-invalid={!!errors['workday.tenantUrl']} aria-describedby={errors['workday.tenantUrl'] ? 'conn-tenant-url-error' : undefined} {...getInputProps(errors['workday.tenantUrl'])} />
                <FieldError id="conn-tenant-url-error" error={errors['workday.tenantUrl']} />
                <Text fontSize="xs" color="gray.400" mt="1">{t('configDetails.connection.workday.tenantUrlHint')}</Text>
              </Box>

              <Box mb="4">
                <Label htmlFor="conn-tenant-id" fontSize="sm" fontWeight="600" display="block" mb="1">{t('configDetails.connection.workday.tenantId')}</Label>
                <Input id="conn-tenant-id" type="text" placeholder={t('configDetails.connection.workday.tenantIdPlaceholder')} value={form.workday.tenantId} onChange={handleWorkdayChange('tenantId')} onBlur={() => validateField('workday.tenantId')} aria-invalid={!!errors['workday.tenantId']} aria-describedby={errors['workday.tenantId'] ? 'conn-tenant-id-error' : undefined} {...getInputProps(errors['workday.tenantId'])} />
                <FieldError id="conn-tenant-id-error" error={errors['workday.tenantId']} />
              </Box>

              <Flex gap="4" mb="4">
                <Box flex="1">
                  <Label htmlFor="conn-username" fontSize="sm" fontWeight="600" display="block" mb="1">{t('configDetails.connection.workday.username')}</Label>
                  <Input id="conn-username" type="text" placeholder={t('configDetails.connection.workday.usernamePlaceholder')} value={form.workday.username} onChange={handleWorkdayChange('username')} onBlur={() => validateField('workday.username')} aria-invalid={!!errors['workday.username']} aria-describedby={errors['workday.username'] ? 'conn-username-error' : undefined} {...getInputProps(errors['workday.username'])} />
                  <FieldError id="conn-username-error" error={errors['workday.username']} />
                </Box>
                <Box flex="1">
                  <Label htmlFor="conn-password" fontSize="sm" fontWeight="600" display="block" mb="1">{t('configDetails.connection.workday.password')}</Label>
                  <Input id="conn-password" type="password" placeholder={t('configDetails.connection.workday.passwordPlaceholder')} value={form.workday.password} onChange={handleWorkdayChange('password')} onBlur={() => validateField('workday.password')} aria-invalid={!!errors['workday.password']} aria-describedby={errors['workday.password'] ? 'conn-password-error' : undefined} {...getInputProps(errors['workday.password'])} />
                  <FieldError id="conn-password-error" error={errors['workday.password']} />
                </Box>
              </Flex>
            </Box>

            <Box as="fieldset" border="none" p="0" m="0" mb="5">
              <Flex as="legend" align="center" gap="2" fontSize="sm" fontWeight="700" color="gray.700" mb="4" pb="2" borderBottom="1px solid" borderColor="gray.100">
                <Text>⚙️</Text>
                <Text>{t('configDetails.connection.workday.queryOptionsTitle')}</Text>
              </Flex>

              <Box mb="4">
                <Label htmlFor="conn-worker-count" fontSize="sm" fontWeight="600" display="block" mb="1">{t('configDetails.connection.workday.workerCountLimit')}</Label>
                <Input id="conn-worker-count" type="number" min={1} max={999} value={form.workday.workerCountLimit} onChange={handleWorkdayChange('workerCountLimit')} {...inputStyles} />
                <Text fontSize="xs" color="gray.400" mt="1">{t('configDetails.connection.workday.workerCountLimitHint')}</Text>
              </Box>

              <Box mb="4">
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
                      {opt.label}
                    </StyledButton>
                  ))}
                </Flex>
                <FieldError id="response-group-error" error={errors['workday.responseGroup']} />
                <Text fontSize="xs" color="gray.400" mt="1">{t('configDetails.connection.workday.responseGroupsHint')}</Text>
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
