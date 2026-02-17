import { useAppSelector, useAppDispatch } from '@/store';
import { selectAuth, performLogin, performLogout } from '@/features/auth/state/authSlice';
import {
  setOrganization,
  toggleTheme,
  showNotification,
  dismissNotification,
  selectGlobal,
} from '@/shared/state/globalSlice';
import { API_BASE_URL } from '@/shared/domain/constants';
import type { Organization, NotificationType } from '@/shared/domain/types';

export function useGlobal() {
  const dispatch = useAppDispatch();
  const authState = useAppSelector(selectAuth);
  const globalState = useAppSelector(selectGlobal);

  return {
    // Auth
    user: authState.user,
    isAuthenticated: authState.isAuthenticated,
    token: authState.token,
    login: (email: string, password: string) =>
      dispatch(performLogin({ email, password })),
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

    // API — the generated client singleton is configured in App.tsx;
    // remote packages receive the baseUrl so they can configure their own.
    apiBaseUrl: API_BASE_URL,
  };
}
