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

/* ── Module-level singleton ──────────────────────────────── */

let _injectedValue: GlobalContextValue | null = null;
let _standaloneValue: GlobalContextValue | null = null;

/**
 * Call from the Module Federation host to inject platform globals
 * before rendering any config routes.
 */
export function setGlobalValue(value: GlobalContextValue): void {
  _injectedValue = value;
}

/**
 * Local bridge hook that config components consume.
 *
 * When loaded via Module Federation the host calls `setGlobalValue()`
 * before rendering.  In standalone / mock-mode dev this falls back
 * to mock data.
 */
export function useGlobal(): GlobalContextValue {
  if (_injectedValue) return _injectedValue;

  if (!MOCK_MODE) {
    console.warn('[useGlobal] No value injected and not in mock mode');
  }

  if (!_standaloneValue) {
    _standaloneValue = {
      user: MOCK_USER,
      isAuthenticated: true,
      token: 'mock-token',
      organization: MOCK_ORG,
      theme: 'light' as const,
      apiClient: new ApiClient(() => 'mock-token', API_BASE_URL),
      showNotification: (message: string, type: NotificationType) => {
        console.log(`[Notification] ${type}: ${message}`);
      },
    };
  }

  return _standaloneValue;
}
