import React, { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Badge,
  Box,
  Button,
  Center,
  Flex,
  Heading,
  Spinner,
  Text,
} from '@chakra-ui/react';
import { useSettingsState } from '../../state/useSettingsState';
import { useSettingsValidation } from '../../state/useSettingsValidation';
import {
  AuthTypeSelector,
  BearerSettings,
  OAuth2Settings,
  FieldSettings,
  RetrySettings,
  SettingsHeader,
  ErrorBanner,
  SettingsFooter,
} from '../components';
import { fetchSettingsThunk } from '../..';
import { useAppDispatch } from '@/store';
import { LoadingStatus } from '../../state/settingsSlice';

export function SettingsPage() {
  const { t } = useTranslation();
  const {
    settings,
    dirty,
    loadingStatus,
    isSaving,
    error,
    handleSave,
    handleTestConnection,
    handleClearError,
  } = useSettingsState();

  const [showAdvanced, setShowAdvanced] = useState(false);
  const dispatch = useAppDispatch();
  const { isValid, validateAll } = useSettingsValidation(settings);

  const onSave = () => {
    if (!validateAll()) return;
    handleSave();
  };

  /* ── Load settings on mount (only once for all hook instances) ───── */
  // multiple components use this hook; run fetch exactly once for the
  // whole module. a plain variable outside the function scope survives
  // across hook invocations.
  useEffect(() => {
    if (loadingStatus !== LoadingStatus.Idle) return;
    dispatch(fetchSettingsThunk());
  }, [dispatch, loadingStatus]);

  if (loadingStatus === LoadingStatus.Loading) {
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
      <SettingsHeader />

      {/* Error banner */}
      {error && <ErrorBanner message={error} onDismiss={handleClearError} />}

      {/* Auth type selection */}
      <AuthTypeSelector />

      {/* provider-specific configuration */}
      {settings.authType === 'bearer' && <BearerSettings showAdvanced={showAdvanced} />}
      {settings.authType === 'oauth2' && <OAuth2Settings />}

      {/* toggle for advanced controls */}
      <Flex justifyContent="flex-end" mb={5}>
        <Button variant="outline" size="sm" onClick={() => setShowAdvanced((s) => !s)}>
          {showAdvanced ? t('settings.hideAdvanced') : t('settings.showAdvanced')}
        </Button>
      </Flex>

      {/* dynamic field settings and body path */}
      <FieldSettings />

      {/* retry policy */}
      {showAdvanced && <RetrySettings />}

      {/* Footer */}
      <SettingsFooter
        onTest={handleTestConnection}
        onSave={onSave}
        disabledSave={(!dirty && !isSaving) || !isValid || isSaving}
        isSaving={isSaving}
      />
    </Box>
  );
}
