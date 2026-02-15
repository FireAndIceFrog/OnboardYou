import { describe, it, expect } from 'vitest';
import reducer, {
  toggleSidebar,
  setSidebarOpen,
  toggleSidebarCollapsed,
} from './layoutSlice';

const initialState = {
  sidebarOpen: false,
  sidebarCollapsed: false,
};

describe('layoutSlice', () => {
  it('should return the initial state', () => {
    const state = reducer(undefined, { type: '@@INIT' });
    expect(state).toEqual(initialState);
  });

  it('toggleSidebar flips sidebarOpen', () => {
    const state = reducer(initialState, toggleSidebar());
    expect(state.sidebarOpen).toBe(true);

    const state2 = reducer(state, toggleSidebar());
    expect(state2.sidebarOpen).toBe(false);
  });

  it('setSidebarOpen sets sidebarOpen to the given value', () => {
    const state = reducer(initialState, setSidebarOpen(true));
    expect(state.sidebarOpen).toBe(true);

    const state2 = reducer(state, setSidebarOpen(false));
    expect(state2.sidebarOpen).toBe(false);
  });

  it('toggleSidebarCollapsed flips sidebarCollapsed', () => {
    const state = reducer(initialState, toggleSidebarCollapsed());
    expect(state.sidebarCollapsed).toBe(true);

    const state2 = reducer(state, toggleSidebarCollapsed());
    expect(state2.sidebarCollapsed).toBe(false);
  });
});
