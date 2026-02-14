import { useContext, useMemo } from 'react';
import { GlobalContext } from '@/shared/state/GlobalContext';
import { AuthContext } from '@/features/auth/state/AuthContext';
import { ApiClient } from '@/shared/services/apiClient';
import type { Organization, NotificationType } from '@/shared/domain/types';

export function useGlobal() {
  const globalCtx = useContext(GlobalContext);
  const authCtx = useContext(AuthContext);

  if (!globalCtx) {
    throw new Error('useGlobal must be used within a GlobalProvider');
  }
  if (!authCtx) {
    throw new Error('useGlobal must be used within an AuthProvider');
  }

  const { state, dispatch } = globalCtx;
  const { state: authState, login, logout, getToken } = authCtx;

  const apiClient = useMemo(
    () => new ApiClient(getToken),
    [getToken],
  );

  return {
    // Auth
    user: authState.user,
    isAuthenticated: authState.isAuthenticated,
    token: authState.token,
    login,
    logout,

    // Organization
    organization: state.organization,
    setOrganization: (org: Organization | null) =>
      dispatch({ type: 'SET_ORGANIZATION', payload: org }),

    // Theme
    theme: state.theme,
    toggleTheme: () => dispatch({ type: 'TOGGLE_THEME' }),

    // Notifications
    notifications: state.notifications,
    showNotification: (message: string, type: NotificationType) =>
      dispatch({ type: 'ADD_NOTIFICATION', payload: { message, type } }),
    dismissNotification: (id: string) =>
      dispatch({ type: 'DISMISS_NOTIFICATION', payload: id }),

    // API
    apiClient,
  };
}
