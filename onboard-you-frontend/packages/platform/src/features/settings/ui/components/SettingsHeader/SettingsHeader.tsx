import React from 'react';
import { Badge, Box, Button, Flex, Heading, Text } from '@chakra-ui/react';
import { useTranslation } from 'react-i18next';
import { useSettingsState } from '@/features/settings';

export function SettingsHeader() {
  const { t } = useTranslation();
  const { saved, dirty, isSaving, showAdvanced, handleToggleShowAdvanced } = useSettingsState();

  return (
    <Flex justifyContent="space-between" alignItems="flex-start" mb={7}>
      <Box>
        <Heading as="h1" size="2xl" fontWeight="bold" mb={1}>
          {t('settings.title')}
        </Heading>
        <Flex gap={2} alignItems="center" mb={2}>
            <Text fontSize="sm" color="fg.muted" maxW="480px">
            {t('settings.subtitle')}
            </Text>
            {/* toggle for advanced controls */}
            <Flex justifyContent="flex-end" mb={5}>
            <Button variant="solid" size="lg" onClick={handleToggleShowAdvanced}>
                {showAdvanced ? t('settings.hideAdvanced') : t('settings.showAdvanced')}
            </Button>
            </Flex>
        </Flex>
      </Box>
      <Flex gap={2} alignItems="center">
        {saved && <Badge colorPalette="green">{t('settings.saved')}</Badge>}
        {dirty && <Badge colorPalette="gray">{t('settings.unsaved')}</Badge>}
        {isSaving && <Badge colorPalette="gray">{t('settings.saving')}</Badge>}
      </Flex>
    </Flex>
  );
}
