import { describe, it, expect } from 'vitest';
import reducer, { setLoading, setUser, setError, logout } from './authSlice';

const initialState = {
  user: null,
  isAuthenticated: false,
  isLoading: true,
  token: null,
  refreshToken: null,
  error: null,
};

const mockUser = {
  id: 'user-001',
  email: 'test@example.com',
  name: 'Test User',
  organizationId: 'org-001',
  role: 'admin' as const,
};

describe('authSlice', () => {
  it('should return the initial state', () => {
    const state = reducer(undefined, { type: '@@INIT' });
    expect(state).toEqual(initialState);
  });

  it('setLoading sets isLoading true and clears error', () => {
    const prev = { ...initialState, isLoading: false, error: 'old error' };
    const state = reducer(prev, setLoading());
    expect(state.isLoading).toBe(true);
    expect(state.error).toBeNull();
  });

  it('setUser sets user, token, and marks as authenticated', () => {
    const state = reducer(
      initialState,
      setUser({ user: mockUser, token: 'abc-token', refreshToken: 'ref-token' }),
    );
    expect(state.user).toEqual(mockUser);
    expect(state.token).toBe('abc-token');
    expect(state.refreshToken).toBe('ref-token');
    expect(state.isAuthenticated).toBe(true);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('setError clears user/token and sets error message', () => {
    const prev = {
      ...initialState,
      user: mockUser,
      token: 'abc',
      isAuthenticated: true,
    };
    const state = reducer(prev, setError('Something went wrong'));
    expect(state.user).toBeNull();
    expect(state.token).toBeNull();
    expect(state.refreshToken).toBeNull();
    expect(state.isAuthenticated).toBe(false);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBe('Something went wrong');
  });

  it('logout clears user and sets isAuthenticated false', () => {
    const prev = {
      ...initialState,
      user: mockUser,
      token: 'abc',
      refreshToken: 'ref',
      isAuthenticated: true,
      isLoading: false,
    };
    const state = reducer(prev, logout());
    expect(state.user).toBeNull();
    expect(state.token).toBeNull();
    expect(state.refreshToken).toBeNull();
    expect(state.isAuthenticated).toBe(false);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });
});
