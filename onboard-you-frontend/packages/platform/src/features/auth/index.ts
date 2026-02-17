import { useAppSelector, useAppDispatch } from '@/store';
import {
  selectAuth,
  performLogin,
  performLogout,
} from './state/authSlice';

/**
 * Convenience hook to consume auth state from Redux.
 * Keeps the same external API shape as the old context hook.
 */
export function useAuth() {
  const dispatch = useAppDispatch();
  const state = useAppSelector(selectAuth);

  return {
    state,
    login: (email: string, password: string) => {
      dispatch(performLogin({ email, password }));
    },
    logout: () => {
      dispatch(performLogout());
    },
    getToken: () => state.token,
  };
}

// State
export {
  initAuth,
  performLogin,
  performLogout,
  selectAuth,
  selectUser,
  selectIsAuthenticated,
  selectIsLoading,
} from './state/authSlice';

// UI
export { LoginPage, ProtectedRoute } from './ui';

// Services
export * from './services';

// Domain
export * from './domain';
