import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { SettingsFooter } from './SettingsFooter';

describe('SettingsFooter', () => {
  it('triggers handlers', () => {
    const testFn = vi.fn();
    const saveFn = vi.fn();
    renderWithProviders(
      <SettingsFooter onTest={testFn} onSave={saveFn} disabledSave={false} isSaving={false} />,
    );

    fireEvent.click(screen.getByText(/test connection/i));
    expect(testFn).toHaveBeenCalled();

    fireEvent.click(screen.getByText(/save settings/i));
    expect(saveFn).toHaveBeenCalled();
  });

  it('disables save button appropriately', () => {
    const { rerender } = renderWithProviders(
      <SettingsFooter onTest={() => {}} onSave={() => {}} disabledSave={true} isSaving={false} />,
    );
    expect(screen.getByText(/save settings/i)).toBeDisabled();

    rerender(
      <SettingsFooter onTest={() => {}} onSave={() => {}} disabledSave={false} isSaving={true} />,
    );
    expect(screen.getByText(/save settings/i)).toHaveAttribute('data-loading');
  });
});