import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { SettingsPage } from './SettingsPage';
import { LoadingStatus } from '../../state/settingsSlice';
import { RootState } from '@/store';

describe('SettingsPage', () => {
  it('renders header and footer components', () => {
    const preloaded = {
      settings: {
        settings: { authType: 'bearer', bearer: { destinationUrl: '', token: '', placement: 'authorization_header', placementKey: '', extraHeaders: {}, schema: {}, bodyPath: '' }, oauth2: { destinationUrl: '', clientId: '', clientSecret: '', tokenUrl: '', scopes: '', grantType: 'client_credentials', refreshToken: '', schema: {}, bodyPath: '' }, retryPolicy: { maxAttempts: 3, initialBackoffMs: 1000, retryableStatusCodes: [429, 502, 503, 504] } },
        saved: false,
        dirty: false,
        loadingStatus: LoadingStatus.Succeeded,
        isSaving: false,
        error: null,
      },
    } as unknown as RootState;
    renderWithProviders(<SettingsPage />, { preloadedState: preloaded });
    // header title comes from translation
    expect(screen.getByText('My Systems')).toBeInTheDocument();
    // footer test connection button should render
    expect(screen.getByText(/test connection/i)).toBeInTheDocument();
  });
});