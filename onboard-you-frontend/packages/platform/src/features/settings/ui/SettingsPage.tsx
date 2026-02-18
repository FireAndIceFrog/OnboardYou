import { useTranslation } from 'react-i18next';
import {
  Badge,
  Box,
  Button,
  Card,
  Center,
  Field,
  Flex,
  Heading,
  Input,
  NativeSelect,
  SimpleGrid,
  Spinner,
  Text,
} from '@chakra-ui/react';
import { PLACEMENT_OPTIONS, GRANT_TYPE_OPTIONS } from '../domain/types';
import { useSettingsState } from '../state/useSettingsState';
import { useSettingsValidation } from '../state/useSettingsValidation';
import { FieldError } from './FieldError';

export function SettingsPage() {
  const { t } = useTranslation();
  const {
    settings,
    saved,
    dirty,
    isLoading,
    isSaving,
    error,
    updateBearer,
    updateOAuth2,
    updateRetry,
    handleAuthTypeChange,
    handleSave,
    handleTestConnection,
    handleClearError,
  } = useSettingsState();

  const { errors, isValid, validateAll } = useSettingsValidation(settings);

  const onSave = () => {
    if (!validateAll()) return;
    handleSave();
  };

  if (isLoading) {
    return (
      <Center minH="300px" role="status" aria-label={t('settings.loading')}>
        <Spinner />
      </Center>
    );
  }

  return (
    <Box
      as="form"
      maxW="800px"
      mx="auto"
      py={8}
      px={6}
      onSubmit={(e: React.FormEvent) => e.preventDefault()}
    >
      {/* Header */}
      <Flex justifyContent="space-between" alignItems="flex-start" mb={7}>
        <Box>
          <Heading as="h1" size="2xl" fontWeight="bold" mb={1}>
            {t('settings.title')}
          </Heading>
          <Text fontSize="sm" color="fg.muted" maxW="480px">
            {t('settings.subtitle')}
          </Text>
        </Box>
        <Flex gap={2} alignItems="center">
          {saved && <Badge colorPalette="green">{t('settings.saved')}</Badge>}
          {dirty && <Badge colorPalette="gray">{t('settings.unsaved')}</Badge>}
          {isSaving && (
            <Badge colorPalette="gray">{t('settings.saving')}</Badge>
          )}
        </Flex>
      </Flex>

      {/* Error banner */}
      {error && (
        <Card.Root mb={5} role="alert">
          <Card.Body>
            <Flex
              alignItems="center"
              justifyContent="space-between"
              gap={3}
              color="fg.error"
            >
              <Text fontSize="sm">{error}</Text>
              <Button
                variant="ghost"
                size="xs"
                onClick={handleClearError}
                aria-label={t('settings.dismissError')}
              >
                ✕
              </Button>
            </Flex>
          </Card.Body>
        </Card.Root>
      )}

      {/* Auth type selection */}
      <Card.Root mb={5}>
        <Card.Body p={6}>
          <Box as="fieldset">
            <Heading as="legend" size="lg" fontWeight="semibold" mb={1}>
              {t('settings.authType.title')}
            </Heading>
            <Text fontSize="sm" color="fg.muted" mb={5}>
              {t('settings.authType.description')}
            </Text>
            <SimpleGrid columns={2} gap={3}>
              {(['bearer', 'oauth2'] as const).map((type) => (
                <Box
                  key={type}
                  role="button"
                  tabIndex={0}
                  display="flex"
                  flexDirection="column"
                  alignItems="flex-start"
                  gap={1}
                  p={4}
                  bg="bg"
                  borderWidth="2px"
                  borderColor={
                    settings.authType === type ? 'blue.500' : 'border'
                  }
                  borderRadius="xl"
                  cursor="pointer"
                  textAlign="left"
                  transition="all 0.15s ease"
                  _hover={{
                    borderColor:
                      settings.authType === type
                        ? 'blue.500'
                        : 'border.emphasized',
                    shadow: 'sm',
                  }}
                  shadow={
                    settings.authType === type
                      ? '0 0 0 3px var(--chakra-colors-blue-100)'
                      : 'none'
                  }
                  onClick={() => handleAuthTypeChange(type)}
                  onKeyDown={(e: React.KeyboardEvent) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      handleAuthTypeChange(type);
                    }
                  }}
                >
                  <Text fontSize="xl">
                    {type === 'bearer' ? '🔑' : '🛡️'}
                  </Text>
                  <Text fontSize="sm" fontWeight="semibold">
                    {t(`settings.authType.${type}`)}
                  </Text>
                  <Text fontSize="xs" color="fg.muted">
                    {t(`settings.authType.${type}Desc`)}
                  </Text>
                </Box>
              ))}
            </SimpleGrid>
          </Box>
        </Card.Body>
      </Card.Root>

      {/* Bearer config */}
      {settings.authType === 'bearer' && (
        <Card.Root mb={5}>
          <Card.Body p={6}>
            <Box as="fieldset">
              <Heading as="legend" size="lg" fontWeight="semibold" mb={4}>
                {t('settings.bearer.title')}
              </Heading>

              <Box mb={4}>
                <Field.Root>
                  <Field.Label>
                    {t('settings.bearer.destinationUrl')}
                  </Field.Label>
                  <Input
                    type="url"
                    placeholder={t(
                      'settings.bearer.destinationUrlPlaceholder',
                    )}
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

              <SimpleGrid columns={{ base: 1, sm: 2 }} gap={4}>
                <Field.Root>
                  <Field.Label>
                    {t('settings.bearer.tokenPlacement')}
                  </Field.Label>
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
                  <Field.Label>
                    {t('settings.bearer.placementKey')}
                  </Field.Label>
                  <Input
                    type="text"
                    placeholder={t(
                      'settings.bearer.placementKeyPlaceholder',
                    )}
                    value={settings.bearer.placementKey}
                    onChange={updateBearer('placementKey')}
                  />
                  <Field.HelperText>
                    {t('settings.bearer.placementKeyHint')}
                  </Field.HelperText>
                </Field.Root>
              </SimpleGrid>
            </Box>
          </Card.Body>
        </Card.Root>
      )}

      {/* OAuth2 config */}
      {settings.authType === 'oauth2' && (
        <Card.Root mb={5}>
          <Card.Body p={6}>
            <Box as="fieldset">
              <Heading as="legend" size="lg" fontWeight="semibold" mb={4}>
                {t('settings.oauth2.title')}
              </Heading>

              <Box mb={4}>
                <Field.Root>
                  <Field.Label>
                    {t('settings.oauth2.destinationUrl')}
                  </Field.Label>
                  <Input
                    type="url"
                    placeholder={t(
                      'settings.oauth2.destinationUrlPlaceholder',
                    )}
                    value={settings.oauth2.destinationUrl}
                    onChange={updateOAuth2('destinationUrl')}
                  />
                </Field.Root>
              </Box>

              <SimpleGrid columns={{ base: 1, sm: 2 }} gap={4} mb={4}>
                <Field.Root invalid={!!errors['oauth2.clientId']}>
                  <Field.Label>
                    {t('settings.oauth2.clientId')}
                  </Field.Label>
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
                  <Field.Label>
                    {t('settings.oauth2.clientSecret')}
                  </Field.Label>
                  <Input
                    type="password"
                    placeholder={t(
                      'settings.oauth2.clientSecretPlaceholder',
                    )}
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
                  <Field.Label>
                    {t('settings.oauth2.tokenUrl')}
                  </Field.Label>
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
                  <Field.Label>
                    {t('settings.oauth2.grantType')}
                  </Field.Label>
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
                  <Field.Label>
                    {t('settings.oauth2.refreshToken')}
                  </Field.Label>
                  <Input
                    type="password"
                    placeholder={t(
                      'settings.oauth2.refreshTokenPlaceholder',
                    )}
                    value={settings.oauth2.refreshToken}
                    onChange={updateOAuth2('refreshToken')}
                  />
                </Field.Root>
              )}
            </Box>
          </Card.Body>
        </Card.Root>
      )}

      {/* Retry policy */}
      <Card.Root mb={5}>
        <Card.Body p={6}>
          <Box as="fieldset">
            <Heading as="legend" size="lg" fontWeight="semibold" mb={1}>
              {t('settings.retry.title')}
            </Heading>
            <Text fontSize="sm" color="fg.muted" mb={5}>
              {t('settings.retry.description')}
            </Text>

            <SimpleGrid columns={{ base: 1, sm: 2 }} gap={4} mb={4}>
              <Field.Root invalid={!!errors['retry.maxAttempts']}>
                <Field.Label>
                  {t('settings.retry.maxAttempts')}
                </Field.Label>
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
                <Field.Label>
                  {t('settings.retry.initialBackoff')}
                </Field.Label>
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
              <Field.Label>
                {t('settings.retry.retryableStatusCodes')}
              </Field.Label>
              <Input
                type="text"
                placeholder={t(
                  'settings.retry.retryableStatusCodesPlaceholder',
                )}
                value={settings.retryPolicy.retryableStatusCodes.join(', ')}
                onChange={updateRetry('retryableStatusCodes')}
              />
              <Field.HelperText>
                {t('settings.retry.retryableStatusCodesHint')}
              </Field.HelperText>
            </Field.Root>
          </Box>
        </Card.Body>
      </Card.Root>

      {/* Footer */}
      <Flex
        justifyContent="flex-end"
        gap={3}
        pt={5}
        borderTopWidth="1px"
        borderColor="border"
      >
        <Button variant="outline" onClick={handleTestConnection}>
          {t('settings.testConnection')}
        </Button>
        <Button
          colorPalette="blue"
          onClick={onSave}
          disabled={(!dirty && !isSaving) || !isValid || isSaving}
          loading={isSaving}
          loadingText={t('settings.saving')}
        >
          {t('settings.saveSettings')}
        </Button>
      </Flex>
    </Box>
  );
}
