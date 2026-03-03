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
import { GRANT_TYPE_OPTIONS } from '../../../domain/types';
import { useSettingsState } from '../../../state/useSettingsState';
import { useSettingsValidation } from '../../../state/useSettingsValidation';
import { FieldError } from '../FieldError/FieldError';
import { FormField } from '@/shared/ui/FormField/FormField';
import { FormSelect } from '@/shared/ui/FormSelect/FormSelect';

export function OAuth2Settings() {
  const { t } = useTranslation();
  const { settings, updateOAuth2 } = useSettingsState();
  const { errors } = useSettingsValidation(settings);

  return (
    <Box as="fieldset" mb={5}>
      <Heading as="legend" size="lg" fontWeight="semibold" mb={4}>
        {t('settings.oauth2.title')}
      </Heading>

      <Box mb={4}>
        <FormField
          error={errors['oauth2.destinationUrl']}
          label={t('settings.oauth2.destinationUrl')}
          placeholder={t('settings.oauth2.destinationUrlPlaceholder')}
          value={settings.oauth2.destinationUrl}
          onChange={updateOAuth2('destinationUrl')}
        />
      </Box>

      <SimpleGrid columns={{ base: 1, sm: 2 }} gap={4} mb={4}>
        <FormField 
          error={errors['oauth2.clientId']}
          label={t('settings.oauth2.clientId')}
          placeholder={t('settings.oauth2.clientIdPlaceholder')}
          value={settings.oauth2.clientId}
          onChange={updateOAuth2('clientId')}
        />
        <FormField
          error={errors['oauth2.clientSecret']}
          label={t('settings.oauth2.clientSecret')}
          placeholder={t('settings.oauth2.clientSecretPlaceholder')}
          value={settings.oauth2.clientSecret}
          onChange={updateOAuth2('clientSecret')}
        />
      </SimpleGrid>

      <Box mb={4}>
        <FormField
          error={errors['oauth2.tokenUrl']}
          label={t('settings.oauth2.tokenUrl')}
          placeholder={t('settings.oauth2.tokenUrlPlaceholder')}
          value={settings.oauth2.tokenUrl}
          onChange={updateOAuth2('tokenUrl')}
        />
      </Box>

      <SimpleGrid columns={{ base: 1, sm: 2 }} gap={4} mb={4}>
        <FormSelect label={t('settings.oauth2.grantType')} value={settings.oauth2.grantType} onChange={updateOAuth2('grantType')}>
          {GRANT_TYPE_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {t(`settings.grantTypeOptions.${opt.value}`)}
            </option>
          ))}
        </FormSelect>
        <FormField
          error={errors['oauth2.scopes']}
          label={t('settings.oauth2.scopes')}
          placeholder={t('settings.oauth2.scopesPlaceholder')}
          value={settings.oauth2.scopes}
          onChange={updateOAuth2('scopes')}
        />
      </SimpleGrid>

      {settings.oauth2.grantType === 'authorization_code' && (
        <FormField
          error={errors['oauth2.refreshToken']}
          label={t('settings.oauth2.refreshToken')}
          placeholder={t('settings.oauth2.refreshTokenPlaceholder')}
          value={settings.oauth2.refreshToken}
          onChange={updateOAuth2('refreshToken')}
          type="password"
        />
      )}
    </Box>
  );
}
