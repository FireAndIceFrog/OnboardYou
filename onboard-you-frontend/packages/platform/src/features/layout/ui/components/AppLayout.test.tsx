import { describe, it, expect } from 'vitest';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { AppLayout } from './AppLayout';

describe('AppLayout', () => {
  it('renders without crashing', () => {
    const { container } = renderWithProviders(<AppLayout />);
    expect(container.querySelector('main')).toBeInTheDocument();
  });
});
