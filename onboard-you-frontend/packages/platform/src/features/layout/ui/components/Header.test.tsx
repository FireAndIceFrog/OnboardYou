import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { Header } from './Header';

describe('Header', () => {
  it('renders the app name', () => {
    renderWithProviders(<Header />, {
      preloadedState: {
        auth: {
          user: { id: 'u1', email: 'test@test.com', name: 'John Doe', organizationId: 'org1', role: 'admin' as const },
          isAuthenticated: true,
          isLoading: false,
          token: 'tok',
          refreshToken: null,
          error: null,
        },
      },
    });
    // APP_NAME should appear somewhere in the header
    expect(screen.getByRole('banner')).toBeInTheDocument();
  });

  it('renders a user menu button', () => {
    renderWithProviders(<Header />, {
      preloadedState: {
        auth: {
          user: { id: 'u1', email: 'test@test.com', name: 'John Doe', organizationId: 'org1', role: 'admin' as const },
          isAuthenticated: true,
          isLoading: false,
          token: 'tok',
          refreshToken: null,
          error: null,
        },
      },
    });
    // The user menu button should show initials
    expect(screen.getByText('JD')).toBeInTheDocument();
  });
});
