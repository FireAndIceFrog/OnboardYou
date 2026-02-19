import { Component } from 'react';
import type { ErrorInfo, ReactNode } from 'react';
import { Box, Button, Center, Heading, Text, VStack } from '@chakra-ui/react';

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode | ((error: Error, reset: () => void) => ReactNode);
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    console.error('[ErrorBoundary] Caught error:', error, errorInfo);
  }

  private handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    const { hasError, error } = this.state;
    const { children, fallback } = this.props;

    if (hasError && error) {
      if (typeof fallback === 'function') {
        return fallback(error, this.handleReset);
      }

      if (fallback) {
        return fallback;
      }

      return (
        <Center minH="300px" p={8} role="alert" aria-live="assertive">
          <VStack gap={4} maxW="420px" textAlign="center">
            <Text fontSize="4xl" aria-hidden="true">⚠️</Text>
            <Heading size="lg">Something went wrong</Heading>
            <Text fontSize="sm" color="fg.muted">
              An unexpected error occurred. Please try again.
            </Text>
            <Button colorPalette="blue" onClick={this.handleReset}>
              Try Again
            </Button>
          </VStack>
        </Center>
      );
    }

    return children;
  }
}
