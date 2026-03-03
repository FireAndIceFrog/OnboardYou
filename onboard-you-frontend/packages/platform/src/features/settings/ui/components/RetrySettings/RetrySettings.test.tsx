import { describe, it, expect } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { RetrySettings } from './RetrySettings';
import { DEFAULT_EGRESS_SETTINGS } from '../../../domain/types';

describe('RetrySettings', () => {
  it('renders retry inputs and updates store', () => {
    const { store } = renderWithProviders(<RetrySettings />);

    const maxInput = screen.getByLabelText('Max Attempts');
    const backoffInput = screen.getByLabelText('Initial Back-off (ms)');
    const codesInput = screen.getByPlaceholderText('429, 502, 503, 504');

    expect(maxInput).toBeInTheDocument();
    expect(backoffInput).toBeInTheDocument();
    expect(codesInput).toBeInTheDocument();

    fireEvent.change(maxInput, { target: { value: '5' } });
    expect(store.getState().settings.settings.retryPolicy.maxAttempts).toBe(5);

    fireEvent.change(backoffInput, { target: { value: '2000' } });
    expect(store.getState().settings.settings.retryPolicy.initialBackoffMs).toBe(2000);

    fireEvent.change(codesInput, { target: { value: '400,401' } });
    expect(store.getState().settings.settings.retryPolicy.retryableStatusCodes).toEqual([400, 401]);
  });
});