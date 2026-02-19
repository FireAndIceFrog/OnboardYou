import { Button, Center, Heading, Text, VStack } from '@chakra-ui/react';

export function RemoteLoadFallback({ reset }: { reset: () => void }) {
  return (
    <Center p={16} role="alert" aria-live="assertive">
      <VStack gap={4} textAlign="center">
        <Text fontSize="2xl" aria-hidden="true">
          ⚠️
        </Text>
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
