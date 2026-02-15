import { describe, it, expect } from 'vitest';
import reducer, {
  setTheme,
  toggleTheme,
  addNotification,
  dismissNotification,
} from './globalSlice';
import type { GlobalState } from './globalSlice';

// Provide a deterministic initial state for tests (avoids localStorage/matchMedia)
const initialState: GlobalState = {
  organization: null,
  notifications: [],
  theme: 'light',
};

describe('globalSlice', () => {
  it('should return the initial state with a theme', () => {
    const state = reducer(undefined, { type: '@@INIT' });
    expect(state.organization).toBeNull();
    expect(state.notifications).toEqual([]);
    expect(['light', 'dark']).toContain(state.theme);
  });

  it('setTheme changes the theme', () => {
    const state = reducer(initialState, setTheme('dark'));
    expect(state.theme).toBe('dark');
  });

  it('toggleTheme flips light to dark', () => {
    const state = reducer(initialState, toggleTheme());
    expect(state.theme).toBe('dark');
  });

  it('toggleTheme flips dark to light', () => {
    const darkState: GlobalState = { ...initialState, theme: 'dark' };
    const state = reducer(darkState, toggleTheme());
    expect(state.theme).toBe('light');
  });

  it('addNotification adds to the notifications array', () => {
    const action = addNotification('Hello', 'success');
    const state = reducer(initialState, action);
    expect(state.notifications).toHaveLength(1);
    expect(state.notifications[0].message).toBe('Hello');
    expect(state.notifications[0].type).toBe('success');
    expect(state.notifications[0].id).toBeDefined();
  });

  it('dismissNotification removes by id', () => {
    // Add a notification first
    const action = addNotification('To remove', 'info');
    const withNotification = reducer(initialState, action);
    const id = withNotification.notifications[0].id;

    const state = reducer(withNotification, dismissNotification(id));
    expect(state.notifications).toHaveLength(0);
  });

  it('dismissNotification only removes the targeted notification', () => {
    const first = addNotification('First', 'info');
    const second = addNotification('Second', 'error');

    let state = reducer(initialState, first);
    state = reducer(state, second);
    expect(state.notifications).toHaveLength(2);

    const idToRemove = state.notifications[0].id;
    state = reducer(state, dismissNotification(idToRemove));
    expect(state.notifications).toHaveLength(1);
    expect(state.notifications[0].message).toBe('Second');
  });
});
