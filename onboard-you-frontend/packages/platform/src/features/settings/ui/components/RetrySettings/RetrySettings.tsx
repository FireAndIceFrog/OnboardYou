import React from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Field, Input, SimpleGrid, Heading, Text } from '@chakra-ui/react';
import { useSettingsState } from '../../../state/useSettingsState';
import { useSettingsValidation } from '../../../state/useSettingsValidation';
import { FieldError } from '../FieldError/FieldError';

export function RetrySettings() {
  const { t } = useTranslation();
  const { settings, updateRetry } = useSettingsState();
  const { errors } = useSettingsValidation(settings);

  return (
    <Box as="fieldset" mb={5}>
      <Heading as="legend" size="lg" fontWeight="semibold" mb={1}>
        {t('settings.retry.title')}
      </Heading>
      <Text fontSize="sm" color="fg.muted" mb={5}>
        {t('settings.retry.description')}
      </Text>

      <SimpleGrid columns={{ base: 1, sm: 2 }} gap={4} mb={4}>
        <Field.Root invalid={!!errors['retry.maxAttempts']}>
          <Field.Label>{t('settings.retry.maxAttempts')}</Field.Label>
          <Input
            type="number"
            min={1}
            max={10}
            value={settings.retryPolicy.maxAttempts}
            onChange={updateRetry('maxAttempts')}
          />
          <FieldError
            id="retry-max-attempts-error"
            error={errors['retry.maxAttempts']}
          />
        </Field.Root>
        <Field.Root invalid={!!errors['retry.initialBackoffMs']}>
          <Field.Label>{t('settings.retry.initialBackoff')}</Field.Label>
          <Input
            type="number"
            min={100}
            step={100}
            value={settings.retryPolicy.initialBackoffMs}
            onChange={updateRetry('initialBackoffMs')}
          />
          <FieldError
            id="retry-initial-backoff-error"
            error={errors['retry.initialBackoffMs']}
          />
        </Field.Root>
      </SimpleGrid>

      <Field.Root>
        <Field.Label>{t('settings.retry.retryableStatusCodes')}</Field.Label>
        <Input
          type="text"
          placeholder={t('settings.retry.retryableStatusCodesPlaceholder')}
          value={settings.retryPolicy.retryableStatusCodes.join(', ')}
          onChange={updateRetry('retryableStatusCodes')}
        />
        <Field.HelperText>
          {t('settings.retry.retryableStatusCodesHint')}
        </Field.HelperText>
      </Field.Root>
    </Box>
  );
}
