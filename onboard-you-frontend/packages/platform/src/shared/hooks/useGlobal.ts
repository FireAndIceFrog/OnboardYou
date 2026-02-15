import { useMemo } from 'react';
import { useAppSelector, useAppDispatch } from '@/store';
import { selectAuth, performLogin, performLogout } from '@/features/auth/state/authSlice';
import {
  setOrganization,
  toggleTheme,
  showNotification,
  dismissNotification,
  selectGlobal,
} from '@/shared/state/globalSlice';
import { ApiClient } from '@/shared/services/apiClient';
import type { Organization, NotificationType } from '@/shared/domain/types';

export function useGlobal() {
  const dispatch = useAppDispatch();
  const authState = useAppSelector(selectAuth);
  const globalState = useAppSelector(selectGlobal);

  const apiClient = useMemo(
    () => new ApiClient(() => authState.token),
    [authState.token],
  );

  return {
    // Auth
    user: authState.user,
    isAuthenticated: authState.isAuthenticated,
    token: authState.token,
    login: () => dispatch(performLogin()),
    logout: () => dispatch(performLogout()),

    // Organization
    organization: globalState.organization,
    setOrganization: (org: Organization | null) => dispatch(setOrganization(org)),

    // Theme
    theme: globalState.theme,
    toggleTheme: () => dispatch(toggleTheme()),

    // Notifications
    notifications: globalState.notifications,
    showNotification: (message: string, type: NotificationType) =>
      dispatch(showNotification(message, type)),
    dismissNotification: (id: string) => dispatch(dismissNotification(id)),

    // API
    apiClient,
  };
}
