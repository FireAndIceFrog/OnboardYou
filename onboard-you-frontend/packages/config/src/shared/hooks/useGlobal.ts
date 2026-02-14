import { createContext, useContext, useMemo } from 'react';
import { ApiClient } from '@/shared/services';
import { API_BASE_URL, MOCK_MODE } from '@/shared/domain/constants';
import type { User, Organization, NotificationType } from '@/shared/domain/types';

export interface GlobalContextValue {
  user: User | null;
  isAuthenticated: boolean;
  token: string | null;
  organization: Organization | null;
  theme: 'light' | 'dark';
  apiClient: ApiClient;
  showNotification: (message: string, type: NotificationType) => void;
}

export const ConfigGlobalContext = createContext<GlobalContextValue | null>(null);

const MOCK_USER: User = {
  id: 'user-001',
  email: 'demo@onboardyou.com',
  name: 'Demo User',
  organizationId: 'org-001',
  role: 'admin',
};

const MOCK_ORG: Organization = {
  id: 'org-001',
  name: 'Acme Corp',
  plan: 'enterprise',
};

/**
 * Local bridge hook that config components consume.
 *
 * When loaded via Module Federation into the platform shell, the platform
 * injects its real GlobalContextValue through ConfigGlobalContext.Provider.
 * In standalone / mock-mode dev, this falls back to mock data.
 */
export function useGlobal(): GlobalContextValue {
  const ctx = useContext(ConfigGlobalContext);

  // If running inside platform via Module Federation, context will be provided
  if (ctx) return ctx;

  // Standalone fallback (dev mode)
  if (!MOCK_MODE) {
    console.warn('[useGlobal] No context provided and not in mock mode');
  }

  // eslint-disable-next-line react-hooks/rules-of-hooks
  return useMemo(
    () => ({
      user: MOCK_USER,
      isAuthenticated: true,
      token: 'mock-token',
      organization: MOCK_ORG,
      theme: 'light' as const,
      apiClient: new ApiClient(() => 'mock-token', API_BASE_URL),
      showNotification: (message: string, type: NotificationType) => {
        console.log(`[Notification] ${type}: ${message}`);
      },
    }),
    [],
  );
}
