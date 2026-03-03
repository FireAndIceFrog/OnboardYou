import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { SettingsPage } from './SettingsPage';

describe('SettingsPage', () => {
  it('renders header and footer components', () => {
    renderWithProviders(<SettingsPage />);
    // header title comes from translation
    expect(screen.getByText('My Systems')).toBeInTheDocument();
    // footer test connection button should render
    expect(screen.getByText(/test connection/i)).toBeInTheDocument();
  });
});