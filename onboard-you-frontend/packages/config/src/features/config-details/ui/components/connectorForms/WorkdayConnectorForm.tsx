import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Text, chakra } from '@chakra-ui/react';
import { LockIcon, GearIcon } from '@/shared/ui';
import type { WorkdayFields } from '../../../domain';
import { RESPONSE_GROUP_OPTIONS } from '../../../domain';
import { FieldError } from '../FieldError';
import { FormField } from '@/shared/ui';
import type { ConnectorFormProps } from './types';

const StyledButton = chakra('button');

interface FieldDef {
  id: string;
  fieldKey: string;
  validationKey?: string;
  type?: string;
  min?: number;
  max?: number;
}

const CREDENTIAL_FIELDS: FieldDef[] = [
  { id: 'conn-tenant-url', fieldKey: 'tenantUrl', validationKey: 'workday.tenantUrl', type: 'url' },
  { id: 'conn-tenant-id', fieldKey: 'tenantId', validationKey: 'workday.tenantId' },
];

const INLINE_CREDENTIALS: FieldDef[] = [
  { id: 'conn-username', fieldKey: 'username', validationKey: 'workday.username' },
  { id: 'conn-password', fieldKey: 'password', validationKey: 'workday.password', type: 'password' },
];

const QUERY_FIELDS: FieldDef[] = [
  { id: 'conn-worker-count', fieldKey: 'workerCountLimit', type: 'number', min: 1, max: 999 },
];

export function WorkdayConnectorForm({ form, errors, onChange, validateField }: ConnectorFormProps) {
  const { t } = useTranslation();

  const activeGroups = useMemo(
    () => new Set(form.workday.responseGroup.split(',').filter(Boolean)),
    [form.workday.responseGroup],
  );

  const getValue = (key: string) => form.workday[key as keyof WorkdayFields] as string | number;
  const getHandler = (key: string) =>
    (e: React.ChangeEvent<HTMLInputElement>) => onChange({ type: 'field', key, value: e.target.value });

  const renderFields = (fields: FieldDef[]) =>
    fields.map((def) => (
      <FormField
        key={def.id}
        id={def.id}
        label={t(`configDetails.connection.workday.${def.fieldKey}`)}
        placeholder={t(`configDetails.connection.workday.${def.fieldKey}Placeholder`, { defaultValue: '' }) || undefined}
        helperText={t(`configDetails.connection.workday.${def.fieldKey}Hint`, { defaultValue: '' }) || undefined}
        type={def.type}
        min={def.min}
        max={def.max}
        value={getValue(def.fieldKey)}
        onChange={getHandler(def.fieldKey)}
        onBlur={def.validationKey ? () => validateField(def.validationKey!) : undefined}
        error={def.validationKey ? errors[def.validationKey] : undefined}
      />
    ));

  return (
    <>
      <Box as="fieldset" border="none" p="0" m="0" mb="5">
        <Flex as="legend" align="center" gap="2" fontSize="sm" fontWeight="700" color="gray.700" mb="4" pb="2" borderBottom="1px solid" borderColor="gray.100">
          <LockIcon size="1em" />
          <Text>{t('configDetails.connection.workday.credentialsTitle')}</Text>
        </Flex>

        <Flex direction="column" gap="4">
          {renderFields(CREDENTIAL_FIELDS)}
        </Flex>

        <Flex gap="4" mt="4">
          {INLINE_CREDENTIALS.map((def) => (
            <Box flex="1" key={def.id}>
              <FormField
                id={def.id}
                label={t(`configDetails.connection.workday.${def.fieldKey}`)}
                placeholder={t(`configDetails.connection.workday.${def.fieldKey}Placeholder`, { defaultValue: '' }) || undefined}
                type={def.type}
                value={getValue(def.fieldKey)}
                onChange={getHandler(def.fieldKey)}
                onBlur={def.validationKey ? () => validateField(def.validationKey!) : undefined}
                error={def.validationKey ? errors[def.validationKey] : undefined}
              />
            </Box>
          ))}
        </Flex>
      </Box>

      <Box as="fieldset" border="none" p="0" m="0" mb="5">
        <Flex as="legend" align="center" gap="2" fontSize="sm" fontWeight="700" color="gray.700" mb="4" pb="2" borderBottom="1px solid" borderColor="gray.100">
          <GearIcon size="1em" />
          <Text>{t('configDetails.connection.workday.queryOptionsTitle')}</Text>
        </Flex>

        <Flex direction="column" gap="4">
          {renderFields(QUERY_FIELDS)}
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
                onClick={() => onChange({ type: 'toggle', key: opt.value })}
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
  );
}
