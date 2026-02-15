import { Component, type ErrorInfo, type ReactNode } from 'react';
import i18n from '@/i18n';
import styles from './ErrorBoundary.module.scss';

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
        <div className={styles.errorBoundary} role="alert" aria-live="assertive">
          <span className={styles.icon}>⚠️</span>
          <h2 className={styles.title}>{i18n.t('errorBoundary.title')}</h2>
          <p className={styles.message}>
            {i18n.t('errorBoundary.message')}
          </p>
          <button type="button" className={styles.retryButton} onClick={this.handleReset}>
            {i18n.t('errorBoundary.retry')}
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
