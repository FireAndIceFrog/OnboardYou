import { describe, it, expect } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { AuthTypeSelector } from './AuthTypeSelector';

describe('AuthTypeSelector', () => {
  it('renders both bearer and oauth2 options', () => {
    renderWithProviders(<AuthTypeSelector />);
    expect(screen.getByText('🔑')).toBeInTheDocument();
    expect(screen.getByText('🛡️')).toBeInTheDocument();
  });

  it('clicking the other card updates the authType in store', () => {
    const { store } = renderWithProviders(<AuthTypeSelector />);
    // initial value is bearer
    expect(store.getState().settings.settings.authType).toBe('bearer');
    fireEvent.click(screen.getByText('🛡️'));
    expect(store.getState().settings.settings.authType).toBe('oauth2');
  });
});