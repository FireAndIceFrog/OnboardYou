import { MOCK_MODE } from '@/shared/domain/constants';
import type { NotificationType } from '@/shared/domain/types';
import { configureApiClient } from '@/shared/services/configureApiClient';

/**
 * The slim contract between platform (host) and config (remote).
 * Auth, user, and organization details stay in the platform —
 * config receives the API base URL and configures its own generated client.
 */
export interface GlobalContextValue {
  apiBaseUrl: string;
  showNotification: (message: string, type: NotificationType) => void;
  theme: 'light' | 'dark';
}

/* ── Module-level singleton ──────────────────────────────── */

let _injectedValue: GlobalContextValue | null = null;

/**
 * Call from the Module Federation host to inject platform globals
 * before rendering any config routes.
 *
 * Also configures config's generated API client with the platform's
 * base URL + an auth interceptor (reads token from sessionStorage).
 */
export function setGlobalValue(value: GlobalContextValue): void {
  _injectedValue = value;
  configureApiClient(value.apiBaseUrl);
}

/**
 * Non-hook accessor — used by Redux store extras so thunks
 * can pull showNotification without UI coupling.
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