import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { LoginPage } from './LoginPage';

const unauthState = {
  auth: {
    user: null,
    isAuthenticated: false,
    isLoading: false,
    token: null,
    refreshToken: null,
    error: null,
  },
} as const;

describe('LoginPage', () => {
  it('renders the login form with email and password inputs', () => {
    renderWithProviders(<LoginPage />, { preloadedState: { ...unauthState } });
    // email field has placeholder "you@company.com"
    expect(screen.getByPlaceholderText('you@company.com')).toBeInTheDocument();
    // password field has placeholder "••••••••"
    expect(screen.getByPlaceholderText('••••••••')).toBeInTheDocument();
  });

  it('renders a sign in button', () => {
    renderWithProviders(<LoginPage />, { preloadedState: { ...unauthState } });
    expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument();
  });

  it('displays an error message when auth error exists', () => {
    renderWithProviders(<LoginPage />, {
      preloadedState: {
        auth: {
          user: null,
          isAuthenticated: false,
          isLoading: false,
          token: null,
          refreshToken: null,
          error: 'Invalid credentials',
        },
      },
    });
    expect(screen.getByText('Invalid credentials')).toBeInTheDocument();
  });
});
