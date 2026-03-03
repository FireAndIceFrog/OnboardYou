import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { SettingsHeader } from './SettingsHeader';

describe('SettingsHeader', () => {
  it('renders title and badges', () => {
    renderWithProviders(<SettingsHeader />);
    expect(screen.getByText('My Systems')).toBeInTheDocument();
  });
});