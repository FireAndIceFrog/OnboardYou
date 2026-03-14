import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { StatCard } from './StatCard';
import type { StatCardData } from '@/features/home/domain/types';

const baseData: StatCardData = {
  label: 'Connected Systems',
  value: '42',
  iconName: 'connections',
};

describe('StatCard', () => {
  it('renders the icon, value and label', () => {
    renderWithProviders(<StatCard data={baseData} />);
    expect(screen.getByText('42')).toBeInTheDocument();
    expect(screen.getByText('Connected Systems')).toBeInTheDocument();
  });

  it('does not render a change badge when no change is provided', () => {
    renderWithProviders(<StatCard data={baseData} />);
    // No trend text should appear
    expect(screen.queryByText(/↑|↓|→/)).toBeNull();
  });

  it('renders a change badge with up trend', () => {
    const data: StatCardData = { ...baseData, trend: 'up', change: '+12%' };
    renderWithProviders(<StatCard data={data} />);
    expect(screen.getByText(/↑ \+12%/)).toBeInTheDocument();
  });

  it('renders a change badge with down trend', () => {
    const data: StatCardData = { ...baseData, trend: 'down', change: '-5%' };
    renderWithProviders(<StatCard data={data} />);
    expect(screen.getByText(/↓ -5%/)).toBeInTheDocument();
  });
});
