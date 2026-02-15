import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import type { LayoutState } from '@/features/layout/domain/types';
import type { RootState } from '@/store';

/* ── Initial state ────────────────────────────────────────── */

const initialState: LayoutState = {
  sidebarOpen: false,
  sidebarCollapsed: false,
};

/* ── Slice ────────────────────────────────────────────────── */

const layoutSlice = createSlice({
  name: 'layout',
  initialState,
  reducers: {
    toggleSidebar(state) {
      state.sidebarOpen = !state.sidebarOpen;
    },
    setSidebarOpen(state, action: PayloadAction<boolean>) {
      state.sidebarOpen = action.payload;
    },
    toggleSidebarCollapsed(state) {
      state.sidebarCollapsed = !state.sidebarCollapsed;
    },
  },
});

export const { toggleSidebar, setSidebarOpen, toggleSidebarCollapsed } = layoutSlice.actions;

/* ── Selectors ────────────────────────────────────────────── */

export const selectLayout = (state: RootState) => state.layout;
export const selectSidebarOpen = (state: RootState) => state.layout.sidebarOpen;
export const selectSidebarCollapsed = (state: RootState) => state.layout.sidebarCollapsed;

export default layoutSlice.reducer;
