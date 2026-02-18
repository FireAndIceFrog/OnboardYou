// Shared UI components
export { ErrorBoundary } from './ui';

// Re-export Chakra primitives used by remote packages for convenience
export { Button, Spinner, Badge } from '@chakra-ui/react';

// Domain
export * from './domain';

// State (Redux)
export {
  setOrganization,
  setTheme,
  toggleTheme,
  addNotification,
  showNotification,
  dismissNotification,
  selectGlobal,
  selectOrganization,
  selectTheme,
  selectNotifications,
} from './state/globalSlice';
export type { GlobalState } from './state/globalSlice';

// Services
export { configureApiClient } from './services';

// Hooks
export { useGlobal } from './hooks';
