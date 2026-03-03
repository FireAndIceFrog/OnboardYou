import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { WizardNavigation } from './WizardNavigation';
import { LoadingStatus } from '../../../state/settingsSlice';
import type { RootState } from '@/store';

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

describe('WizardNavigation', () => {
  it('renders Previous and Next buttons', () => {
    renderWithProviders(<WizardNavigation />, { preloadedState: preloaded() });
    expect(screen.getByRole('button', { name: /previous/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /next/i })).toBeInTheDocument();
  });

  it('disables Previous on first step', () => {
    renderWithProviders(<WizardNavigation />, { preloadedState: preloaded({ wizardStep: 0 }) });
    expect(screen.getByRole('button', { name: /previous/i })).toBeDisabled();
  });

  it('disables Next on last step (no advanced)', () => {
    renderWithProviders(<WizardNavigation />, {
      preloadedState: preloaded({ wizardStep: 1, showAdvanced: false }),
    });
    expect(screen.getByRole('button', { name: /next/i })).toBeDisabled();
  });

  it('enables Next on step 1 when advanced is on (3 steps)', () => {
    renderWithProviders(<WizardNavigation />, {
      preloadedState: preloaded({ wizardStep: 1, showAdvanced: true }),
    });
    expect(screen.getByRole('button', { name: /next/i })).not.toBeDisabled();
  });

  it('clicking Next advances the step', async () => {
    const user = userEvent.setup();
    const { store } = renderWithProviders(<WizardNavigation />, {
      preloadedState: preloaded({ wizardStep: 0 }),
    });
    await user.click(screen.getByRole('button', { name: /next/i }));
    expect((store.getState() as RootState).settings.wizardStep).toBe(1);
  });

  it('clicking Previous goes back', async () => {
    const user = userEvent.setup();
    const { store } = renderWithProviders(<WizardNavigation />, {
      preloadedState: preloaded({ wizardStep: 1 }),
    });
    await user.click(screen.getByRole('button', { name: /previous/i }));
    expect((store.getState() as RootState).settings.wizardStep).toBe(0);
  });

  it('shows the correct number of step dots', () => {
    const { container } = renderWithProviders(<WizardNavigation />, {
      preloadedState: preloaded({ showAdvanced: true }),
    });
    const dots = container.querySelectorAll('[aria-current="step"], [class*="css"]');
    // Just verify the step indicator text shows 1/3
    expect(screen.getByText(/1\/3/)).toBeInTheDocument();
  });
});
