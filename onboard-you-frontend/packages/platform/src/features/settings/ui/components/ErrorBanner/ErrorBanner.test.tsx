import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { ErrorBanner } from './ErrorBanner';

describe('ErrorBanner', () => {
  it('renders nothing when no message', () => {
    const { container } = renderWithProviders(<ErrorBanner message="" onDismiss={() => {}} />);
    expect(container.firstChild).toBeNull();
  });

  it('shows message and calls dismiss', () => {
    const dismiss = vi.fn();
    renderWithProviders(<ErrorBanner message="Oops" onDismiss={dismiss} />);
    expect(screen.getByText('Oops')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button'));
    expect(dismiss).toHaveBeenCalled();
  });
});