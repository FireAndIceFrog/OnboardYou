import React from 'react';
import { Box, Button, Card, Flex, Text } from '@chakra-ui/react';
import { useTranslation } from 'react-i18next';

interface ErrorBannerProps {
  message: string;
  onDismiss: () => void;
}

export function ErrorBanner({ message, onDismiss }: ErrorBannerProps) {
  const { t } = useTranslation();

  if (!message) return null;

  return (
    <Card.Root mb={5} role="alert">
      <Card.Body>
        <Flex
          alignItems="center"
          justifyContent="space-between"
          gap={3}
          color="fg.error"
        >
          <Text fontSize="sm">{message}</Text>
          <Button
            variant="ghost"
            size="xs"
            onClick={onDismiss}
            aria-label={t('settings.dismissError')}
          >
            ✕
          </Button>
        </Flex>
      </Card.Body>
    </Card.Root>
  );
}
