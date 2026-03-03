import React from 'react';
import { useTranslation } from 'react-i18next';
import {
  Box,
  Field,
  Heading,
  Input,
  NativeSelect,
  SimpleGrid,
} from '@chakra-ui/react';
import { PLACEMENT_OPTIONS } from '../../../domain/types';
import { useSettingsState } from '../../../state/useSettingsState';
import { useSettingsValidation } from '../../../state/useSettingsValidation';
import { FieldError } from '../FieldError/FieldError';

interface BearerSettingsProps {
  showAdvanced: boolean;
}

export function BearerSettings({ showAdvanced }: BearerSettingsProps) {
  const { t } = useTranslation();
  const { settings, updateBearer } = useSettingsState();
  const { errors } = useSettingsValidation(settings);

  return (
    <Box as="fieldset" mb={5}>
      <Heading as="legend" size="lg" fontWeight="semibold" mb={4}>
        {t('settings.bearer.title')}
      </Heading>

      <Box mb={4}>
        <Field.Root>
          <Field.Label>{t('settings.bearer.destinationUrl')}</Field.Label>
          <Input
            type="url"
            placeholder={t('settings.bearer.destinationUrlPlaceholder')}
            value={settings.bearer.destinationUrl}
            onChange={updateBearer('destinationUrl')}
          />
          <Field.HelperText>
            {t('settings.bearer.destinationUrlHint')}
          </Field.HelperText>
        </Field.Root>
      </Box>

      <Box mb={4}>
        <Field.Root invalid={!!errors['bearer.token']}>
          <Field.Label>{t('settings.bearer.token')}</Field.Label>
          <Input
            type="password"
            placeholder={t('settings.bearer.tokenPlaceholder')}
            value={settings.bearer.token}
            onChange={updateBearer('token')}
          />
          <FieldError
            id="bearer-token-error"
            error={errors['bearer.token']}
          />
        </Field.Root>
      </Box>

      {/* placement fields are advanced only */}
      {showAdvanced && (
        <SimpleGrid columns={{ base: 1, sm: 2 }} gap={4}>
          <Field.Root>
            <Field.Label>{t('settings.bearer.tokenPlacement')}</Field.Label>
            <NativeSelect.Root>
              <NativeSelect.Field
                value={settings.bearer.placement}
                onChange={updateBearer('placement')}
              >
                {PLACEMENT_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {t(`settings.placementOptions.${opt.value}`)}
                  </option>
                ))}
              </NativeSelect.Field>
              <NativeSelect.Indicator />
            </NativeSelect.Root>
          </Field.Root>
          <Field.Root>
            <Field.Label>{t('settings.bearer.placementKey')}</Field.Label>
            <Input
              type="text"
              placeholder={t('settings.bearer.placementKeyPlaceholder')}
              value={settings.bearer.placementKey}
              onChange={updateBearer('placementKey')}
            />
            <Field.HelperText>
              {t('settings.bearer.placementKeyHint')}
            </Field.HelperText>
          </Field.Root>
        </SimpleGrid>
      )}
    </Box>
  );
}
