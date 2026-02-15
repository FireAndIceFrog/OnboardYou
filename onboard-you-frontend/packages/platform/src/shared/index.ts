// Shared UI components
export { Button, Spinner, Badge, Card, ErrorBoundary } from './ui';
export type { ButtonProps, SpinnerProps, BadgeProps, CardProps } from './ui';

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
export { ApiClient } from './services';

// Hooks
export { useGlobal } from './hooks';
