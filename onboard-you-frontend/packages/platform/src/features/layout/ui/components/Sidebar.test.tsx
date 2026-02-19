import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { Sidebar } from './Sidebar';

describe('Sidebar', () => {
  it('renders a nav element with main navigation label', () => {
    renderWithProviders(<Sidebar />);
    expect(screen.getByRole('navigation', { name: /main navigation/i })).toBeInTheDocument();
  });
});
