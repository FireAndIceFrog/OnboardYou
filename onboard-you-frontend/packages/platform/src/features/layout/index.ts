import { useAppSelector, useAppDispatch } from '@/store';
import {
  selectLayout,
  toggleSidebar,
  setSidebarOpen,
  toggleSidebarCollapsed,
} from './state/layoutSlice';

/**
 * Convenience hook to consume layout state from Redux.
 */
export function useLayout() {
  const dispatch = useAppDispatch();
  const state = useAppSelector(selectLayout);

  return {
    state,
    toggleSidebar: () => dispatch(toggleSidebar()),
    setSidebarOpen: (open: boolean) => dispatch(setSidebarOpen(open)),
    toggleSidebarCollapsed: () => dispatch(toggleSidebarCollapsed()),
  };
}

// State
export {
  toggleSidebar,
  setSidebarOpen,
  toggleSidebarCollapsed,
  selectLayout,
  selectSidebarOpen,
  selectSidebarCollapsed,
} from './state/layoutSlice';

// UI
export { AppLayout, Header, Sidebar } from './ui';

// Domain
export * from './domain';
