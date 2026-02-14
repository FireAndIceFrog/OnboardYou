import { useContext } from 'react';
import { AuthContext } from './state/AuthContext';
import type { AuthContextValue } from './state/AuthContext';

/**
 * Convenience hook to consume the auth context.
 */
export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return ctx;
}

// State
export { AuthProvider } from './state';
export { AuthContext } from './state';
export type { AuthContextValue } from './state';

// UI
export { LoginPage, CallbackPage, ProtectedRoute } from './ui';

// Services
export * from './services';

// Domain
export * from './domain';
