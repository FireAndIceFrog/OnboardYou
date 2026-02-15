import { useAppSelector, useAppDispatch } from '@/store';
import {
  selectAuth,
  performLogin,
  performLogout,
  exchangeCode,
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
    login: () => {
      dispatch(performLogin());
    },
    logout: () => {
      dispatch(performLogout());
    },
    getToken: () => state.token,
    exchangeCode: (code: string) => dispatch(exchangeCode(code)).unwrap(),
  };
}

// State
export {
  initAuth,
  exchangeCode,
  performLogin,
  performLogout,
  selectAuth,
  selectUser,
  selectIsAuthenticated,
  selectIsLoading,
} from './state/authSlice';

// UI
export { LoginPage, CallbackPage, ProtectedRoute } from './ui';

// Services
export * from './services';

// Domain
export * from './domain';
