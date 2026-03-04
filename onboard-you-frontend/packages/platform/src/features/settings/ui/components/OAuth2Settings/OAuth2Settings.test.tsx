import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { OAuth2Settings } from './OAuth2Settings';
import { DEFAULT_EGRESS_SETTINGS } from '../../../domain/types';
import { LoadingStatus } from '../../../state/settingsSlice';

describe('OAuth2Settings', () => {
  it('renders fields for oauth2 configuration', () => {
    const preloaded = {
      settings: {
        settings: {
          ...DEFAULT_EGRESS_SETTINGS,
          authType: 'oauth2' as const,
        },
        saved: false,
        dirty: false,
        loadingStatus: LoadingStatus.Idle,
        isSaving: false,
        error: null,
        showAdvanced: false,
        wizardStep: 0,
      },
    };

    renderWithProviders(<OAuth2Settings />, { preloadedState: preloaded });
    expect(screen.getByPlaceholderText('https://api.example.com/employees')).toBeInTheDocument();
  });
});