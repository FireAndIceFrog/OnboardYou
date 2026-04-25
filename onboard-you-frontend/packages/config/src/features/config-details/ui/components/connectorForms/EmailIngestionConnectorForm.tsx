import { useTranslation } from 'react-i18next';
import { Box, Flex, Text, Textarea, Input } from '@chakra-ui/react';
import { EnvelopeIcon } from '@/shared/ui';
import { FieldError } from '../FieldError';
import type { ConnectorFormProps } from './types';

export function EmailIngestionConnectorForm({ form, errors, onChange }: ConnectorFormProps) {
  const { t } = useTranslation();
  const { allowedSenders, subjectFilter } = form.emailIngestion;

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
        <EnvelopeIcon size="1em" />
        <Text>{t('configDetails.connection.emailIngestion.title', 'Email Ingestion')}</Text>
      </Flex>

      {/* Allowed senders */}
      <Box mb="4">
        <label htmlFor="conn-email-senders" style={{ fontSize: 'var(--chakra-font-sizes-sm)', fontWeight: 600, display: 'block', marginBottom: '4px' }}>
          {t('configDetails.connection.emailIngestion.allowedSendersLabel', 'Allowed Senders')}
          <Text as="span" color="red.500" ml="1" aria-hidden>*</Text>
        </label>
        <Textarea
          id="conn-email-senders"
          value={allowedSenders}
          placeholder="hr@acme.com, @partner.com"
          fontSize="sm"
          rows={3}
          aria-describedby="email-senders-hint email-senders-error"
          aria-required
          onChange={(e) =>
            onChange({ type: 'field', key: 'allowedSenders', value: e.target.value })
          }
        />
        <FieldError id="email-senders-error" error={errors['emailIngestion.allowedSenders']} />
        <Text id="email-senders-hint" fontSize="xs" color="gray.400" mt="1">
          {t(
            'configDetails.connection.emailIngestion.allowedSendersHint',
            'Comma-separated email addresses or domain globs (e.g. hr@acme.com, @partner.com)',
          )}
        </Text>
      </Box>

      {/* Subject filter */}
      <Box mb="2">
        <label htmlFor="conn-email-subject" style={{ fontSize: 'var(--chakra-font-sizes-sm)', fontWeight: 600, display: 'block', marginBottom: '4px' }}>
          {t('configDetails.connection.emailIngestion.subjectFilterLabel', 'Subject Filter')}
          <Text as="span" color="gray.400" fontSize="xs" fontWeight="400" ml="2">
            ({t('common.optional', 'optional')})
          </Text>
        </label>
        <Input
          id="conn-email-subject"
          value={subjectFilter}
          placeholder={t('configDetails.connection.emailIngestion.subjectFilterPlaceholder', 'e.g. Monthly Roster')}
          fontSize="sm"
          aria-describedby="email-subject-hint"
          onChange={(e) =>
            onChange({ type: 'field', key: 'subjectFilter', value: e.target.value })
          }
        />
        <Text id="email-subject-hint" fontSize="xs" color="gray.400" mt="1">
          {t(
            'configDetails.connection.emailIngestion.subjectFilterHint',
            'Only process emails whose subject contains this text (case-insensitive). Leave blank to accept all.',
          )}
        </Text>
      </Box>
    </Box>
  );
}
