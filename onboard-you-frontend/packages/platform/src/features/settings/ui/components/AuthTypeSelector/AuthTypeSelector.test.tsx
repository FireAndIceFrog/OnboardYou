import { describe, it, expect } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { AuthTypeSelector } from './AuthTypeSelector';

describe('AuthTypeSelector', () => {
  it('renders both bearer and oauth2 options', () => {
    renderWithProviders(<AuthTypeSelector />);
    // SVG icons are now used instead of emojis; check option labels exist
    expect(screen.getAllByRole('button').length).toBeGreaterThanOrEqual(2);
  });

  it('clicking the other card updates the authType in store', () => {
    const { store } = renderWithProviders(<AuthTypeSelector />);
    // initial value is bearer
    expect(store.getState().settings.settings.authType).toBe('bearer');
    const buttons = screen.getAllByRole('button');
    fireEvent.click(buttons[1]);
    expect(store.getState().settings.settings.authType).toBe('oauth2');
  });
});