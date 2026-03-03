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
        <Field.Root>
          <Field.Label>{t('settings.oauth2.destinationUrl')}</Field.Label>
          <Input
            type="url"
            placeholder={t('settings.oauth2.destinationUrlPlaceholder')}
            value={settings.oauth2.destinationUrl}
            onChange={updateOAuth2('destinationUrl')}
          />
        </Field.Root>
      </Box>

      <SimpleGrid columns={{ base: 1, sm: 2 }} gap={4} mb={4}>
        <Field.Root invalid={!!errors['oauth2.clientId']}>
          <Field.Label>{t('settings.oauth2.clientId')}</Field.Label>
          <Input
            type="text"
            placeholder={t('settings.oauth2.clientIdPlaceholder')}
            value={settings.oauth2.clientId}
            onChange={updateOAuth2('clientId')}
          />
          <FieldError
            id="oauth2-client-id-error"
            error={errors['oauth2.clientId']}
          />
        </Field.Root>
        <Field.Root invalid={!!errors['oauth2.clientSecret']}>
          <Field.Label>{t('settings.oauth2.clientSecret')}</Field.Label>
          <Input
            type="password"
            placeholder={t('settings.oauth2.clientSecretPlaceholder')}
            value={settings.oauth2.clientSecret}
            onChange={updateOAuth2('clientSecret')}
          />
          <FieldError
            id="oauth2-client-secret-error"
            error={errors['oauth2.clientSecret']}
          />
        </Field.Root>
      </SimpleGrid>

      <Box mb={4}>
        <Field.Root invalid={!!errors['oauth2.tokenUrl']}>
          <Field.Label>{t('settings.oauth2.tokenUrl')}</Field.Label>
          <Input
            type="url"
            placeholder={t('settings.oauth2.tokenUrlPlaceholder')}
            value={settings.oauth2.tokenUrl}
            onChange={updateOAuth2('tokenUrl')}
          />
          <FieldError
            id="oauth2-token-url-error"
            error={errors['oauth2.tokenUrl']}
          />
        </Field.Root>
      </Box>

      <SimpleGrid columns={{ base: 1, sm: 2 }} gap={4} mb={4}>
        <Field.Root>
          <Field.Label>{t('settings.oauth2.grantType')}</Field.Label>
          <NativeSelect.Root>
            <NativeSelect.Field
              value={settings.oauth2.grantType}
              onChange={updateOAuth2('grantType')}
            >
              {GRANT_TYPE_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {t(`settings.grantTypeOptions.${opt.value}`)}
                </option>
              ))}
            </NativeSelect.Field>
            <NativeSelect.Indicator />
          </NativeSelect.Root>
        </Field.Root>
        <Field.Root>
          <Field.Label>{t('settings.oauth2.scopes')}</Field.Label>
          <Input
            type="text"
            placeholder={t('settings.oauth2.scopesPlaceholder')}
            value={settings.oauth2.scopes}
            onChange={updateOAuth2('scopes')}
          />
        </Field.Root>
      </SimpleGrid>

      {settings.oauth2.grantType === 'authorization_code' && (
        <Field.Root>
          <Field.Label>{t('settings.oauth2.refreshToken')}</Field.Label>
          <Input
            type="password"
            placeholder={t('settings.oauth2.refreshTokenPlaceholder')}
            value={settings.oauth2.refreshToken}
            onChange={updateOAuth2('refreshToken')}
          />
        </Field.Root>
      )}
    </Box>
  );
}
