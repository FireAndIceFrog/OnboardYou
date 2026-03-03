import React from 'react';
import { useTranslation } from 'react-i18next';
import {
  Box,
  Heading,
  SimpleGrid,
} from '@chakra-ui/react';
import { PLACEMENT_OPTIONS } from '../../../domain/types';
import { useSettingsState } from '../../../state/useSettingsState';
import { useSettingsValidation } from '../../../state/useSettingsValidation';
import { FormField } from '@/shared/ui/FormField/FormField';
import { FormSelect } from '@/shared/ui/FormSelect/FormSelect';

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
        <FormField
          label={t('settings.bearer.destinationUrl')}
          type="url"
          placeholder={t('settings.bearer.destinationUrlPlaceholder')}
          value={settings.bearer.destinationUrl}
          onChange={updateBearer('destinationUrl')}
          helperText={t('settings.bearer.destinationUrlHint')}
        />
      </Box>

      <Box mb={4}>
        <FormField
          error={errors['bearer.token']}
          label={t('settings.bearer.token')}
          type="password"
          placeholder={t('settings.bearer.tokenPlaceholder')}
          value={settings.bearer.token}
          onChange={updateBearer('token')}
        />
      </Box>

      {/* placement fields are advanced only */}
      {showAdvanced && (
        <SimpleGrid columns={{ base: 1, sm: 2 }} gap={4}>
          <FormSelect
            label={t('settings.bearer.tokenPlacement')}
            value={settings.bearer.placement}
            onChange={updateBearer('placement')}
          >
            {PLACEMENT_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {t(`settings.placementOptions.${opt.value}`)}
              </option>
            ))}
          </FormSelect>
          <FormField
            label={t('settings.bearer.placementKey')}
            placeholder={t('settings.bearer.placementKeyPlaceholder')}
            value={settings.bearer.placementKey}
            onChange={updateBearer('placementKey')}
            helperText={t('settings.bearer.placementKeyHint')}
          />
        </SimpleGrid>
      )}
    </Box>
  );
}
