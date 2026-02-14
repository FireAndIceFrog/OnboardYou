// Shared UI components
export { Button, Spinner, Badge, Card } from './ui';
export type { ButtonProps, SpinnerProps, BadgeProps, CardProps } from './ui';

// Domain
export * from './domain';

// State
export { GlobalProvider, GlobalContext } from './state';
export type { GlobalContextValue, GlobalState, GlobalAction } from './state';

// Services
export { ApiClient } from './services';

// Hooks
export { useGlobal } from './hooks';
