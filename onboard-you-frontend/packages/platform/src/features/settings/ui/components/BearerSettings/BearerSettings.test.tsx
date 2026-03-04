import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { BearerSettings } from './BearerSettings';
import { DEFAULT_EGRESS_SETTINGS } from '../../../domain/types';
import { LoadingStatus } from '../../../state/settingsSlice';

describe('BearerSettings', () => {
  it('renders fields for bearer configuration', () => {
    const preloaded = {
      settings: {
        settings: DEFAULT_EGRESS_SETTINGS,
        saved: false,
        dirty: false,
        loadingStatus: LoadingStatus.Idle,
        isSaving: false,
        error: null,
        showAdvanced: false,
        wizardStep: 0,
      },
    };

    renderWithProviders(<BearerSettings showAdvanced={false} />, { preloadedState: preloaded });
    // destination url input should render with english placeholder
    expect(screen.getByPlaceholderText('https://api.example.com/employees')).toBeInTheDocument();
  });
});