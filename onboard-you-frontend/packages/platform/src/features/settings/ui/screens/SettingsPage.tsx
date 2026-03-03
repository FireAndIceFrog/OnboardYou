import React, { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Box,
  Center,
  Spinner,
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
  WizardNavigation,
} from '../components';
import { fetchSettingsThunk } from '../..';
import { useAppDispatch } from '@/store';
import { LoadingStatus } from '../../state/settingsSlice';

export function SettingsPage() {
  const { t } = useTranslation();
  const {
    showAdvanced,
    wizardStep,
    settings,
    dirty,
    loadingStatus,
    isSaving,
    error,
    handleSave,
    handleTestConnection,
    handleClearError,
  } = useSettingsState();

  const dispatch = useAppDispatch();
  const { isValid, validateAll } = useSettingsValidation(settings);

  const onSave = () => {
    if (!validateAll()) return;
    handleSave();
  };

  /* ── Load settings on mount ─────────────────────────────── */
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
      
      {/* Wizard navigation */}
      <WizardNavigation />

      {/* Error banner */}
      {error && <ErrorBanner message={error} onDismiss={handleClearError} />}

      {/* ── Step 0: Connection / Auth ──────────────────────── */}
      {wizardStep === 0 && (
        <>
          <AuthTypeSelector />
          {settings.authType === 'bearer' && <BearerSettings showAdvanced={showAdvanced} />}
          {settings.authType === 'oauth2' && <OAuth2Settings />}
        </>
      )}

      {/* ── Step 1: Dynamic Payload ────────────────────────── */}
      {wizardStep === 1 && <FieldSettings />}

      {/* ── Step 2: Retry Policy (advanced only) ───────────── */}
      {wizardStep === 2 && showAdvanced && <RetrySettings />}


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
