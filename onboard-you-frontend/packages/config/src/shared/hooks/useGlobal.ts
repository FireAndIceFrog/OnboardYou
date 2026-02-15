import type { ApiClient } from '@/shared/services';
import { MOCK_MODE } from '@/shared/domain/constants';
import type { NotificationType } from '@/shared/domain/types';

/**
 * The slim contract between platform (host) and config (remote).
 * Auth, user, and organization details stay in the platform —
 * config only receives a ready-to-use API client.
 */
export interface GlobalContextValue {
  apiClient: ApiClient;
  showNotification: (message: string, type: NotificationType) => void;
  theme: 'light' | 'dark';
}

/* ── Module-level singleton ──────────────────────────────── */

let _injectedValue: GlobalContextValue | null = null;

/**
 * Call from the Module Federation host to inject platform globals
 * before rendering any config routes.
 */
export function setGlobalValue(value: GlobalContextValue): void {
  _injectedValue = value;
}

/**
 * Non-hook accessor — used by Redux store extras so thunks
 * can pull apiClient / showNotification without UI coupling.
 */
export function getGlobalValue(): GlobalContextValue | null {
  return _injectedValue;
}

/**
 * Local bridge hook that config components consume.
 *
 * When loaded via Module Federation the host calls `setGlobalValue()`
 * before rendering.  In standalone / mock-mode dev this falls back
 * to a minimal mock.
 */
export function useGlobal(): GlobalContextValue {
  if (_injectedValue) return _injectedValue;

  if (!MOCK_MODE) {
    console.warn('[useGlobal] No value injected and not in mock mode');
  }

  throw new Error('No global value injected. useGlobal() cannot function without it.');
}