import { createSlice, nanoid, PayloadAction } from '@reduxjs/toolkit';
import type { Organization, Notification, NotificationType, Theme } from '@/shared/domain/types';
import type { RootState, AppDispatch } from '@/store';

const NOTIFICATION_DISMISS_MS = 5_000;
const THEME_STORAGE_KEY = 'onboardyou-theme';

function getInitialTheme(): Theme {
  try {
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    if (stored === 'light' || stored === 'dark') return stored;
  } catch {
    // localStorage unavailable (SSR / sandboxed iframe)
  }
  if (typeof window !== 'undefined' && window.matchMedia?.('(prefers-color-scheme: dark)').matches) {
    return 'dark';
  }
  return 'light';
}

/* ── State type ───────────────────────────────────────────── */

export interface GlobalState {
  organization: Organization | null;
  notifications: Notification[];
  theme: Theme;
}

/* ── Initial state ────────────────────────────────────────── */

const initialState: GlobalState = {
  organization: null,
  notifications: [],
  theme: getInitialTheme(),
};

/* ── Slice ────────────────────────────────────────────────── */

const globalSlice = createSlice({
  name: 'global',
  initialState,
  reducers: {
    setOrganization(state, action: PayloadAction<Organization | null>) {
      state.organization = action.payload;
    },
    setTheme(state, action: PayloadAction<Theme>) {
      state.theme = action.payload;
    },
    toggleTheme(state) {
      state.theme = state.theme === 'light' ? 'dark' : 'light';
    },
    addNotification: {
      reducer(state, action: PayloadAction<Notification>) {
        state.notifications.push(action.payload);
      },
      prepare(message: string, type: NotificationType) {
        return {
          payload: {
            id: nanoid(),
            message,
            type,
            timestamp: Date.now(),
          },
        };
      },
    },
    dismissNotification(state, action: PayloadAction<string>) {
      state.notifications = state.notifications.filter((n) => n.id !== action.payload);
    },
  },
});

export const {
  setOrganization,
  setTheme,
  toggleTheme,
  addNotification,
  dismissNotification,
} = globalSlice.actions;

/* ── Thunks ───────────────────────────────────────────────── */

/** Adds a notification and auto-dismisses it after 5 seconds. */
export const showNotification =
  (message: string, type: NotificationType) => (dispatch: AppDispatch) => {
    const action = addNotification(message, type);
    dispatch(action);
    setTimeout(() => {
      dispatch(dismissNotification(action.payload.id));
    }, NOTIFICATION_DISMISS_MS);
  };

/* ── Selectors ────────────────────────────────────────────── */

export const selectGlobal = (state: RootState) => state.global;
export const selectOrganization = (state: RootState) => state.global.organization;
export const selectTheme = (state: RootState) => state.global.theme;
export const selectNotifications = (state: RootState) => state.global.notifications;

export default globalSlice.reducer;
