import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { MappingEditor } from '../MappingEditor';

function renderMapping(value: unknown, availableColumns: string[] = []) {
  const onChange = vi.fn();
  renderWithProviders(
    <MappingEditor value={value} onChange={onChange} availableColumns={availableColumns} />,
  );
  return { onChange };
}

describe('MappingEditor', () => {
  it('renders empty state with add button', () => {
    renderMapping({});
    expect(screen.getByTestId('mapping-add-row')).toBeInTheDocument();
    expect(screen.queryByTestId('mapping-row-0')).not.toBeInTheDocument();
  });

  it('renders existing entries', () => {
    renderMapping({ old_name: 'new_name' }, ['old_name', 'other']);
    expect(screen.getByTestId('mapping-row-0')).toBeInTheDocument();
  });

  it('adds a new row when clicking add', async () => {
    const user = userEvent.setup();
    const { onChange } = renderMapping({}, ['col1']);
    await user.click(screen.getByTestId('mapping-add-row'));
    expect(onChange).toHaveBeenCalledWith({ '': '' });
  });

  it('removes a row when clicking remove', async () => {
    const user = userEvent.setup();
    const { onChange } = renderMapping({ old_name: 'new_name' }, ['old_name']);
    const removeBtn = screen.getByRole('button', { name: /remove/i });
    await user.click(removeBtn);
    expect(onChange).toHaveBeenCalledWith({});
  });

  it('handles non-object values gracefully', () => {
    renderMapping(null);
    expect(screen.getByTestId('mapping-editor')).toBeInTheDocument();
  });
});
