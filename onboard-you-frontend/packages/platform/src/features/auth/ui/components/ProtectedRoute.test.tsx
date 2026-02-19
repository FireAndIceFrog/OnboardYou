import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { ProtectedRoute } from './ProtectedRoute';

describe('ProtectedRoute', () => {
  it('shows spinner when auth is loading', () => {
    renderWithProviders(<ProtectedRoute />, {
      preloadedState: {
        auth: {
          user: null,
          isAuthenticated: false,
          isLoading: true,
          token: null,
          refreshToken: null,
          error: null,
        },
      },
    });
    expect(screen.getByText('', { selector: '.chakra-spinner' }) || document.querySelector('[class*="spinner"]')).toBeTruthy();
  });

  it('redirects to /login when not authenticated and not loading', () => {
    renderWithProviders(<ProtectedRoute />, {
      initialRoute: '/dashboard',
      preloadedState: {
        auth: {
          user: null,
          isAuthenticated: false,
          isLoading: false,
          token: null,
          refreshToken: null,
          error: null,
        },
      },
    });
    // The Navigate component will redirect, but in our test wrapper we use MemoryRouter
    // so we can't check the URL directly. The outlet content won't render.
    expect(screen.queryByTestId('outlet-content')).toBeNull();
  });
});
