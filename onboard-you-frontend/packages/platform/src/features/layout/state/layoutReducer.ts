import type { LayoutState } from '@/features/layout/domain/types';

export type LayoutAction =
  | { type: 'TOGGLE_SIDEBAR' }
  | { type: 'SET_SIDEBAR_OPEN'; payload: boolean }
  | { type: 'TOGGLE_SIDEBAR_COLLAPSED' };

export const initialLayoutState: LayoutState = {
  sidebarOpen: false,
  sidebarCollapsed: false,
};

export function layoutReducer(state: LayoutState, action: LayoutAction): LayoutState {
  switch (action.type) {
    case 'TOGGLE_SIDEBAR':
      return { ...state, sidebarOpen: !state.sidebarOpen };
    case 'SET_SIDEBAR_OPEN':
      return { ...state, sidebarOpen: action.payload };
    case 'TOGGLE_SIDEBAR_COLLAPSED':
      return { ...state, sidebarCollapsed: !state.sidebarCollapsed };
    default:
      return state;
  }
}
