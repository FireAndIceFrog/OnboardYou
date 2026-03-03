import React from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Heading, SimpleGrid, Text } from '@chakra-ui/react';
import { useSettingsState } from '../../../state/useSettingsState';

export function AuthTypeSelector() {
  const { t } = useTranslation();
  const { settings, handleAuthTypeChange } = useSettingsState();

  return (
    <Box as="fieldset" mb={5}>
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
            borderColor={settings.authType === type ? 'blue.500' : 'border'}
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
            <Text fontSize="xl">{type === 'bearer' ? '🔑' : '🛡️'}</Text>
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
  );
}
