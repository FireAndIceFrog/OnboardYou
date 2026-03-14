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
        <Heading as="h1" size="2xl" fontWeight="bold" mb={1} color="primary.500">
          {t('settings.title')}
        </Heading>
            <Text fontSize="sm" color="tertiary.500" maxW="480px">
            {t('settings.subtitle')}
            </Text>
      </Box>
      <Box>
            {/* toggle for advanced controls */}
            <Button variant="outline" size="lg" borderColor="tertiary.300" color="tertiary.600" onClick={handleToggleShowAdvanced}>
                {showAdvanced ? t('settings.hideAdvanced') : t('settings.showAdvanced')}
            </Button>
        <Flex gap={2} alignItems="center" mt={5}>
            {saved && <Badge colorPalette="green">{t('settings.saved')}</Badge>}
            {dirty && <Badge colorPalette="gray">{t('settings.unsaved')}</Badge>}
            {isSaving && <Badge colorPalette="gray">{t('settings.saving')}</Badge>}

        </Flex>
      </Box>
    </Flex>
  );
}
