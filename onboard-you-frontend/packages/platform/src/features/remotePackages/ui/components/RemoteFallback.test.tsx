import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { RemoteLoadFallback } from './RemoteFallback';

describe('RemoteLoadFallback', () => {
  it('renders the error heading', () => {
    renderWithProviders(<RemoteLoadFallback reset={() => {}} />);
    expect(screen.getByRole('heading', { name: /failed to load module/i })).toBeInTheDocument();
  });

  it('renders a try again button that calls reset', async () => {
    const resetFn = vi.fn();
    const { getByRole } = renderWithProviders(
      <RemoteLoadFallback reset={resetFn} />,
    );
    await getByRole('button', { name: /try again/i }).click();
    expect(resetFn).toHaveBeenCalled();
  });

  it('has an alert role for accessibility', () => {
    renderWithProviders(<RemoteLoadFallback reset={() => {}} />);
    expect(screen.getByRole('alert')).toBeInTheDocument();
  });
});
