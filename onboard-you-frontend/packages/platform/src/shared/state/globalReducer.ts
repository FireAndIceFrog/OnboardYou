import type { Organization, Notification, NotificationType, Theme } from '@/shared/domain/types';

export interface GlobalState {
  organization: Organization | null;
  notifications: Notification[];
  theme: Theme;
}

export type GlobalAction =
  | { type: 'SET_ORGANIZATION'; payload: Organization | null }
  | { type: 'SET_THEME'; payload: Theme }
  | { type: 'TOGGLE_THEME' }
  | { type: 'ADD_NOTIFICATION'; payload: { message: string; type: NotificationType } }
  | { type: 'DISMISS_NOTIFICATION'; payload: string };

let notificationId = 0;

export function globalReducer(state: GlobalState, action: GlobalAction): GlobalState {
  switch (action.type) {
    case 'SET_ORGANIZATION':
      return { ...state, organization: action.payload };
    case 'SET_THEME':
      return { ...state, theme: action.payload };
    case 'TOGGLE_THEME':
      return { ...state, theme: state.theme === 'light' ? 'dark' : 'light' };
    case 'ADD_NOTIFICATION':
      return {
        ...state,
        notifications: [
          ...state.notifications,
          {
            id: `notif-${++notificationId}`,
            message: action.payload.message,
            type: action.payload.type,
            timestamp: Date.now(),
          },
        ],
      };
    case 'DISMISS_NOTIFICATION':
      return {
        ...state,
        notifications: state.notifications.filter((n) => n.id !== action.payload),
      };
    default:
      return state;
  }
}

export const initialGlobalState: GlobalState = {
  organization: null,
  notifications: [],
  theme: 'light',
};
