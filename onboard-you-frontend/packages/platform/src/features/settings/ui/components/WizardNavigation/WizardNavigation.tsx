import React from 'react';
import { Box, Button, Flex, Text } from '@chakra-ui/react';
import { useTranslation } from 'react-i18next';
import { useSettingsState } from '../../../state/useSettingsState';

/** Step labels displayed in the dot indicator */
const STEP_KEYS = [
  'settings.wizard.stepConnection',
  'settings.wizard.stepPayload',
  'settings.wizard.stepRetries',
] as const;

export function WizardNavigation() {
  const { t } = useTranslation();
  const { wizardStep, totalSteps, goNext, goPrev } = useSettingsState();

  const isFirst = wizardStep === 0;
  const isLast = wizardStep === totalSteps - 1;

  return (
    <Flex
      justifyContent="space-between"
      alignItems="center"
      pt={6}
      mt={6}
      pb={6}
      mb={6}
      borderTopWidth="1px"
      borderBottomWidth="1px"
      borderColor="border"
    >
      {/* Previous button */}
      <Button
        variant="outline"
        onClick={goPrev}
        disabled={isFirst}
        aria-label={t('settings.wizard.previous')}
      >
        ← {t('settings.wizard.previous')}
      </Button>

      {/* Step dots + label */}
      <Flex direction="column" alignItems="center" gap={1}>
        <Flex gap={2} mb={3}>
          {Array.from({ length: totalSteps }).map((_, i) => (
            <Box
              key={i}
              w="10px"
              h="10px"
              borderRadius="full"
              bg={i === wizardStep ? 'blue.500' : 'gray.300'}
              transition="background 0.2s"
              aria-current={i === wizardStep ? 'step' : undefined}
            />
          ))}
        </Flex>
        <Text fontSize="xs" color="fg.muted">
          {t(STEP_KEYS[wizardStep])} ({wizardStep + 1}/{totalSteps})
        </Text>
      </Flex>

      {/* Next button */}
      <Button
        variant="outline"
        onClick={goNext}
        disabled={isLast}
        aria-label={t('settings.wizard.next')}
      >
        {t('settings.wizard.next')} →
      </Button>
    </Flex>
  );
}
