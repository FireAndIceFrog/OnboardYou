import React from 'react';
import { useTranslation } from 'react-i18next';
import { Box, SimpleGrid, Heading, Text } from '@chakra-ui/react';
import { useSettingsState } from '../../../state/useSettingsState';
import { useSettingsValidation } from '../../../state/useSettingsValidation';
import { FormField } from '@/shared/ui/FormField/FormField';

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
        <FormField
          error={errors['retry.maxAttempts']}
          label={t('settings.retry.maxAttempts')}
          type="number"
          min={1}
          max={10}
          value={settings.retryPolicy.maxAttempts}
          onChange={updateRetry('maxAttempts')}
        />
        <FormField
          error={errors['retry.initialBackoffMs']}
          label={t('settings.retry.initialBackoff')}
          type="number"
          min={100}
          step={100}
          value={settings.retryPolicy.initialBackoffMs}
          onChange={updateRetry('initialBackoffMs')}
        />
      </SimpleGrid>

      <FormField
        label={t('settings.retry.retryableStatusCodes')}
        placeholder={t('settings.retry.retryableStatusCodesPlaceholder')}
        value={settings.retryPolicy.retryableStatusCodes.join(', ')}
        onChange={updateRetry('retryableStatusCodes')}
        helperText={t('settings.retry.retryableStatusCodesHint')}
      />
    </Box>
  );
}
