import { Component, type ErrorInfo, type ReactNode } from 'react';
import { Box, Heading, Text, Button } from '@chakra-ui/react';
import i18n from '@/i18n';

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
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
    console.error('[ErrorBoundary] Uncaught error:', error, errorInfo);
  }

  private handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <Box
          display="flex"
          flexDirection="column"
          alignItems="center"
          justifyContent="center"
          minH="200px"
          p="8"
          textAlign="center"
          gap="4"
          role="alert"
          aria-live="assertive"
        >
          <Text fontSize="2.5rem">⚠️</Text>
          <Heading size="lg">{i18n.t('errorBoundary.title')}</Heading>
          <Text fontSize="sm" color="gray.500" maxW="420px" lineHeight="1.5">
            {i18n.t('errorBoundary.message')}
          </Text>
          <Button size="sm" colorPalette="blue" onClick={this.handleReset}>
            {i18n.t('errorBoundary.retry')}
          </Button>
        </Box>
      );
    }

    return this.props.children;
  }
}
