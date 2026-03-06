import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { BearerSettings } from './BearerSettings';
import { DEFAULT_EGRESS_SETTINGS } from '../../../domain/types';
import { RootState } from '@/store';

describe('BearerSettings', () => {
  it('renders fields for bearer configuration', () => {
    const preloaded = {
      settings: {
        settings: DEFAULT_EGRESS_SETTINGS,
        saved: false,
        dirty: false,
        isLoading: false,
        isSaving: false,
        error: null,
      },
    };

    renderWithProviders(<BearerSettings showAdvanced={false} />, { preloadedState: preloaded as unknown as RootState});
    // destination url input should render with english placeholder
    expect(screen.getByPlaceholderText('https://api.example.com/employees')).toBeInTheDocument();
  });
});