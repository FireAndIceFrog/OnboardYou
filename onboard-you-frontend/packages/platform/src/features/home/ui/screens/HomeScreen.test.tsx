import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { HomeScreen } from './HomeScreen';

describe('HomeScreen', () => {
  it('renders a welcome heading', () => {
    renderWithProviders(<HomeScreen />, {
      preloadedState: {
        auth: {
          user: { id: 'u1', email: 'test@test.com', name: 'Alice', organizationId: 'org1', role: 'admin' as const },
          isAuthenticated: true,
          isLoading: false,
          token: 'tok',
          refreshToken: null,
          error: null,
        },
      },
    });
    expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument();
  });

  it('renders a dashboard overview section', () => {
    renderWithProviders(<HomeScreen />);
    expect(screen.getByLabelText('Dashboard overview')).toBeInTheDocument();
  });
});
