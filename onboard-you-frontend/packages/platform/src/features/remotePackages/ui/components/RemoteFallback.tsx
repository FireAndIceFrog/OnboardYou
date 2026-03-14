import { Button, Center, Heading, Text, VStack } from '@chakra-ui/react';
import { AlertTriangleIcon } from '@/shared/ui';

export function RemoteLoadFallback({ reset }: { reset: () => void }) {
  return (
    <Center p={16} role="alert" aria-live="assertive">
      <VStack gap={4} textAlign="center">
        <AlertTriangleIcon size="1.5em" aria-hidden="true" />
        <Heading size="md">Failed to load module</Heading>
        <Text color="fg.muted">
          The remote module could not be loaded. Please check your connection
          and try again.
        </Text>
        <Button colorPalette="blue" onClick={reset}>
          Try Again
        </Button>
      </VStack>
    </Center>
  );
}
