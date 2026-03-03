import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { SettingsPage } from './SettingsPage';
import { LoadingStatus } from '../../state/settingsSlice';
import { RootState } from '@/store';

const baseSettings = {
  authType: 'bearer' as const,
  bearer: {
    destinationUrl: '',
    token: '',
    placement: 'authorization_header' as const,
    placementKey: '',
    extraHeaders: {},
    schema: {},
    bodyPath: '',
  },
  oauth2: {
    destinationUrl: '',
    clientId: '',
    clientSecret: '',
    tokenUrl: '',
    scopes: '',
    grantType: 'client_credentials' as const,
    refreshToken: '',
    schema: {},
    bodyPath: '',
  },
  retryPolicy: {
    maxAttempts: 3,
    initialBackoffMs: 1000,
    retryableStatusCodes: [429, 502, 503, 504],
  },
};

function preloaded(overrides: Record<string, unknown> = {}): Partial<RootState> {
  return {
    settings: {
      settings: baseSettings,
      saved: false,
      dirty: false,
      loadingStatus: LoadingStatus.Succeeded,
      isSaving: false,
      error: null,
      showAdvanced: false,
      wizardStep: 0,
      ...overrides,
    },
  } as unknown as Partial<RootState>;
}

describe('SettingsPage', () => {
  it('renders header and footer components', () => {
    renderWithProviders(<SettingsPage />, { preloadedState: preloaded() });
    expect(screen.getByText('My Systems')).toBeInTheDocument();
    expect(screen.getByText(/test connection/i)).toBeInTheDocument();
  });

  it('shows auth type selector on step 0', () => {
    renderWithProviders(<SettingsPage />, { preloadedState: preloaded({ wizardStep: 0 }) });
    expect(screen.getByText(/authentication type/i)).toBeInTheDocument();
  });

  it('shows dynamic payload on step 1', () => {
    renderWithProviders(<SettingsPage />, { preloadedState: preloaded({ wizardStep: 1 }) });
    expect(screen.getByText(/dynamic payload/i)).toBeInTheDocument();
  });

  it('shows retry policy on step 2 when advanced', () => {
    renderWithProviders(<SettingsPage />, {
      preloadedState: preloaded({ wizardStep: 2, showAdvanced: true }),
    });
    expect(screen.getByText(/retry policy/i)).toBeInTheDocument();
  });

  it('renders wizard navigation with Previous and Next', () => {
    renderWithProviders(<SettingsPage />, { preloadedState: preloaded() });
    expect(screen.getByRole('button', { name: /previous/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /next/i })).toBeInTheDocument();
  });
});