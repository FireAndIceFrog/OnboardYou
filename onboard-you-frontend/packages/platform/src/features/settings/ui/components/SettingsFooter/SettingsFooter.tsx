import React from 'react';
import { Button, Flex } from '@chakra-ui/react';
import { useTranslation } from 'react-i18next';

interface SettingsFooterProps {
  onTest: () => void;
  onSave: () => void;
  disabledSave: boolean;
  isSaving: boolean;
}

export function SettingsFooter({ onTest, onSave, disabledSave, isSaving }: SettingsFooterProps) {
  const { t } = useTranslation();

  return (
    <Flex
      justifyContent="flex-end"
      gap={3}
      pt={5}
      borderTopWidth="1px"
      borderColor="border"
    >
      <Button variant="outline" onClick={onTest}> 
        {t('settings.testConnection')}
      </Button>
      <Button
        colorPalette="blue"
        onClick={onSave}
        disabled={disabledSave}
        loading={isSaving}
        loadingText={t('settings.saving')}
      >
        {t('settings.saveSettings')}
      </Button>
    </Flex>
  );
}
