// ============================================================================
// OnboardYou — Auth Context
// ============================================================================

import { createContext, useContext } from 'react';
import type { User } from '@/types';

/** Shape of the authentication context exposed to consumers. */
export interface AuthContextValue {
  /** Currently authenticated user, or null. */
  user: User | null;
  /** Whether the user is authenticated. */
  isAuthenticated: boolean;
  /** Whether the auth state is being resolved (e.g. on first load). */
  isLoading: boolean;
  /** Redirect to Cognito hosted UI for sign-in. */
  login: () => void;
  /** Clear the session and redirect to Cognito logout. */
  logout: () => void;
  /** Return the current access token JWT, or null. */
  getToken: () => string | null;
  /** Exchange an OAuth authorization code for tokens (used by CallbackPage). */
  exchangeCode: (code: string) => Promise<void>;
}

export const AuthContext = createContext<AuthContextValue | undefined>(undefined);

/**
 * Hook to consume the AuthContext.
 * Must be used inside an `<AuthProvider>`.
 */
export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an <AuthProvider>');
  }
  return context;
}
