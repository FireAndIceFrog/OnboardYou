import { useTranslation } from 'react-i18next';
import { Box, Flex, Text, chakra } from '@chakra-ui/react';
import type { SageHrFields } from '../../../domain';
import { SAGE_HR_HISTORY_OPTIONS } from '../../../domain';
import { FieldError } from '../FieldError';
import { FormField } from '@/shared/ui';
import type { ConnectorFormProps } from './types';

const StyledButton = chakra('button');

interface FieldDef {
  id: string;
  fieldKey: string;
  validationKey?: string;
  type?: string;
}

const CREDENTIAL_FIELDS: FieldDef[] = [
  { id: 'conn-sage-subdomain', fieldKey: 'subdomain', validationKey: 'sageHr.subdomain' },
  { id: 'conn-sage-api-token', fieldKey: 'apiToken', validationKey: 'sageHr.apiToken', type: 'password' },
];

export function SageHrConnectorForm({ form, errors, onChange, validateField }: ConnectorFormProps) {
  const { t } = useTranslation();

  const getValue = (key: string) => form.sageHr[key as keyof SageHrFields] as string;
  const getHandler = (key: string) =>
    (e: React.ChangeEvent<HTMLInputElement>) => onChange({ type: 'field', key, value: e.target.value });

  return (
    <>
      <Box as="fieldset" border="none" p="0" m="0" mb="5">
        <Flex as="legend" align="center" gap="2" fontSize="sm" fontWeight="700" color="gray.700" mb="4" pb="2" borderBottom="1px solid" borderColor="gray.100">
          <Text>🔐</Text>
          <Text>{t('configDetails.connection.sageHr.credentialsTitle')}</Text>
        </Flex>

        <Flex direction="column" gap="4">
          {CREDENTIAL_FIELDS.map((def) => (
            <FormField
              key={def.id}
              id={def.id}
              label={t(`configDetails.connection.sageHr.${def.fieldKey}`)}
              placeholder={t(`configDetails.connection.sageHr.${def.fieldKey}Placeholder`, { defaultValue: '' }) || undefined}
              type={def.type}
              value={getValue(def.fieldKey)}
              onChange={getHandler(def.fieldKey)}
              onBlur={def.validationKey ? () => validateField(def.validationKey!) : undefined}
              error={def.validationKey ? errors[def.validationKey] : undefined}
            />
          ))}
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
                onClick={() => onChange({ type: 'toggle', key: opt.value })}
              >
                {t(opt.labelKey)}
              </StyledButton>
            ))}
          </Flex>
          <Text fontSize="xs" color="gray.400" mt="1">{t('configDetails.connection.sageHr.historyOptionsHint')}</Text>
        </Box>
      </Box>
    </>
  );
}
