import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { PiiMaskingPanel } from './PiiMaskingPanel';

function renderPanel(
  config: Record<string, unknown>,
  availableColumns: string[] = [],
) {
  const onChange = vi.fn();
  renderWithProviders(
    <PiiMaskingPanel config={config} onChange={onChange} availableColumns={availableColumns} />,
  );
  return { onChange };
}

describe('PiiMaskingPanel', () => {
  it('renders the panel with title', () => {
    renderPanel({ columns: [] });
    expect(screen.getByTestId('pii-masking-panel')).toBeInTheDocument();
    expect(screen.getByText('Sensitive Columns')).toBeInTheDocument();
  });

  it('renders existing PII column rows', () => {
    renderPanel({
      columns: [
        { name: 'ssn', strategy: { Redact: { keep_last: 4, mask_prefix: '***-**-' } } },
        { name: 'salary', strategy: 'Zero' },
      ],
    });
    expect(screen.getByTestId('pii-row-0')).toBeInTheDocument();
    expect(screen.getByTestId('pii-row-1')).toBeInTheDocument();
  });

  it('adds a new column row when clicking add', async () => {
    const user = userEvent.setup();
    const { onChange } = renderPanel({ columns: [] }, ['ssn', 'email']);
    await user.click(screen.getByTestId('pii-add-column'));
    expect(onChange).toHaveBeenCalledWith('columns', [
      { name: '', strategy: { Redact: { keep_last: 4, mask_prefix: '***-**-' } } },
    ]);
  });

  it('removes a column row when clicking remove', async () => {
    const user = userEvent.setup();
    const { onChange } = renderPanel({
      columns: [{ name: 'ssn', strategy: 'Zero' }],
    });
    const removeBtn = screen.getByRole('button', { name: /remove/i });
    await user.click(removeBtn);
    expect(onChange).toHaveBeenCalledWith('columns', []);
  });

  it('handles missing columns gracefully', () => {
    renderPanel({});
    expect(screen.getByTestId('pii-masking-panel')).toBeInTheDocument();
  });
});
