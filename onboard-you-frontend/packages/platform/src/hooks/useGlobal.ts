// ============================================================================
// OnboardYou — useGlobal Hook
//
// THE key hook that microfrontends consume. Combines a Zustand store (for
// UI / org state) with the AuthContext (for tokens and user info) and exposes
// a unified API including a pre-configured fetch wrapper.
// ============================================================================

import { useCallback, useMemo } from 'react';
import { create } from 'zustand';
import { useAuth } from '@/auth/AuthContext';
import type { Organization, Theme } from '@/types';

// ---------------------------------------------------------------------------
// Notification types
// ---------------------------------------------------------------------------

export type NotificationType = 'success' | 'error' | 'warning' | 'info';

export interface Notification {
  id: string;
  message: string;
  type: NotificationType;
}

// ---------------------------------------------------------------------------
// API client type
// ---------------------------------------------------------------------------

export interface ApiClient {
  get: <T = unknown>(path: string) => Promise<T>;
  post: <T = unknown>(path: string, body?: unknown) => Promise<T>;
  put: <T = unknown>(path: string, body?: unknown) => Promise<T>;
  patch: <T = unknown>(path: string, body?: unknown) => Promise<T>;
  del: <T = unknown>(path: string) => Promise<T>;
}

// ---------------------------------------------------------------------------
// Zustand store (non-auth global state)
// ---------------------------------------------------------------------------

interface GlobalStore {
  /** Current organization / tenant. */
  organization: Organization | null;
  setOrganization: (org: Organization | null) => void;

  /** Sidebar open/collapsed state. */
  sidebarOpen: boolean;
  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;

  /** Theme preference. */
  theme: Theme;
  toggleTheme: () => void;
  setTheme: (theme: Theme) => void;

  /** In-app toast notifications. */
  notifications: Notification[];
  showNotification: (message: string, type: NotificationType) => void;
  dismissNotification: (id: string) => void;
}

let notificationCounter = 0;

export const useGlobalStore = create<GlobalStore>()((set) => ({
  // ---- Organization ----
  organization: null,
  setOrganization: (organization) => set({ organization }),

  // ---- Sidebar ----
  sidebarOpen: true,
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  setSidebarOpen: (sidebarOpen) => set({ sidebarOpen }),

  // ---- Theme ----
  theme: 'light',
  toggleTheme: () =>
    set((s) => ({ theme: s.theme === 'light' ? 'dark' : 'light' })),
  setTheme: (theme) => set({ theme }),

  // ---- Notifications ----
  notifications: [],
  showNotification: (message, type) => {
    const id = `notif-${++notificationCounter}-${Date.now()}`;
    set((s) => ({
      notifications: [...s.notifications, { id, message, type }],
    }));

    // Auto-dismiss after 5 seconds
    setTimeout(() => {
      set((s) => ({
        notifications: s.notifications.filter((n) => n.id !== id),
      }));
    }, 5000);
  },
  dismissNotification: (id) =>
    set((s) => ({
      notifications: s.notifications.filter((n) => n.id !== id),
    })),
}));

// ---------------------------------------------------------------------------
// API client factory
// ---------------------------------------------------------------------------

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL as string | undefined;

function createApiClient(
  getToken: () => string | null,
  organizationId: string | undefined,
): ApiClient {
  async function request<T>(
    method: string,
    path: string,
    body?: unknown,
  ): Promise<T> {
    const token = getToken();
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }
    if (organizationId) {
      headers['X-Organization-Id'] = organizationId;
    }

    const url = `${API_BASE_URL ?? ''}${path}`;

    const res = await fetch(url, {
      method,
      headers,
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });

    if (!res.ok) {
      const text = await res.text();
      throw new Error(`API ${method} ${path} failed (${res.status}): ${text}`);
    }

    // 204 No Content — return undefined as T
    if (res.status === 204) return undefined as T;
    return (await res.json()) as T;
  }

  return {
    get: <T = unknown>(path: string) => request<T>('GET', path),
    post: <T = unknown>(path: string, body?: unknown) =>
      request<T>('POST', path, body),
    put: <T = unknown>(path: string, body?: unknown) =>
      request<T>('PUT', path, body),
    patch: <T = unknown>(path: string, body?: unknown) =>
      request<T>('PATCH', path, body),
    del: <T = unknown>(path: string) => request<T>('DELETE', path),
  };
}

// ---------------------------------------------------------------------------
// Composite hook: Zustand store + AuthContext + apiClient
// ---------------------------------------------------------------------------

export interface UseGlobalReturn extends Omit<GlobalStore, never> {
  auth: {
    user: ReturnType<typeof useAuth>['user'];
    isAuthenticated: boolean;
    token: string | null;
  };
  apiClient: ApiClient;
}

export function useGlobal(): UseGlobalReturn {
  const store = useGlobalStore();
  const { user, isAuthenticated, getToken } = useAuth();

  const token = getToken();
  const organizationId = user?.organizationId;

  const apiClient = useMemo(
    () => createApiClient(getToken, organizationId),
    // getToken is stable (useCallback in provider), organizationId changes on user switch
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [organizationId],
  );

  const auth = useMemo(
    () => ({ user, isAuthenticated, token }),
    [user, isAuthenticated, token],
  );

  // Return a combined object
  return useMemo(
    () => ({
      ...store,
      auth,
      apiClient,
    }),
    [store, auth, apiClient],
  );
}

// ---------------------------------------------------------------------------
// Convenience: expose showNotification as a standalone callable (non-hook)
// so it can be used outside React components (e.g. in API interceptors).
// ---------------------------------------------------------------------------

export const showNotification = useGlobalStore.getState().showNotification;
