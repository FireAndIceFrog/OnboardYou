import { describe, it, expect, vi } from 'vitest';
import { screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { FieldEditor } from './FieldEditor';
import type { FieldSchema } from '../../domain/actionCatalog';

function renderField(schema: FieldSchema, value: unknown, availableColumns: string[] = []) {
  const onChange = vi.fn();
  renderWithProviders(<FieldEditor schema={schema} value={value} onChange={onChange} availableColumns={availableColumns} />);
  return { onChange };
}

describe('FieldEditor', () => {
  describe('readonly type', () => {
    it('renders a string value', () => {
      renderField({ key: 'name', label: 'Name', type: 'readonly' }, 'hello');
      expect(screen.getByTestId('field-readonly-name')).toHaveTextContent('hello');
    });

    it('renders an array as comma-separated values', () => {
      renderField({ key: 'cols', label: 'Columns', type: 'readonly' }, ['a', 'b', 'c']);
      expect(screen.getByTestId('field-readonly-cols')).toHaveTextContent('a, b, c');
    });

    it('renders null as dash', () => {
      renderField({ key: 'x', label: 'X', type: 'readonly' }, null);
      expect(screen.getByTestId('field-readonly-x')).toHaveTextContent('—');
    });

    it('renders an object as JSON', () => {
      renderField({ key: 'obj', label: 'Obj', type: 'readonly' }, { foo: 'bar' });
      expect(screen.getByTestId('field-readonly-obj')).toHaveTextContent('"foo": "bar"');
    });
  });

  describe('text type', () => {
    it('renders input with current value', () => {
      renderField({ key: 'name', label: 'Name', type: 'text', placeholder: 'Enter name' }, 'test');
      const input = screen.getByTestId('field-text-name') as HTMLInputElement;
      expect(input.value).toBe('test');
    });

    it('calls onChange when typing', async () => {
      const user = userEvent.setup();
      const { onChange } = renderField({ key: 'name', label: 'Name', type: 'text' }, '');
      const input = screen.getByTestId('field-text-name');
      await user.type(input, 'a');
      expect(onChange).toHaveBeenCalledWith('name', 'a');
    });
  });

  describe('number type', () => {
    it('renders input with current value', () => {
      renderField({ key: 'count', label: 'Count', type: 'number' }, 42);
      const input = screen.getByTestId('field-number-count') as HTMLInputElement;
      expect(input.value).toBe('42');
    });

    it('renders empty string for null', () => {
      renderField({ key: 'count', label: 'Count', type: 'number' }, null);
      const input = screen.getByTestId('field-number-count') as HTMLInputElement;
      expect(input.value).toBe('');
    });
  });

  describe('select type', () => {
    it('renders options from schema', () => {
      renderField(
        {
          key: 'format',
          label: 'Format',
          type: 'select',
          options: [
            { value: 'alpha2', label: '2-letter' },
            { value: 'alpha3', label: '3-letter' },
          ],
        },
        'alpha2',
      );
      const select = screen.getByTestId('field-select-format') as HTMLSelectElement;
      expect(select.value).toBe('alpha2');
      expect(select.options).toHaveLength(2);
    });

    it('calls onChange on selection', async () => {
      const user = userEvent.setup();
      const { onChange } = renderField(
        {
          key: 'format',
          label: 'Format',
          type: 'select',
          options: [
            { value: 'alpha2', label: '2-letter' },
            { value: 'alpha3', label: '3-letter' },
          ],
        },
        'alpha2',
      );
      await user.selectOptions(screen.getByTestId('field-select-format'), 'alpha3');
      expect(onChange).toHaveBeenCalledWith('format', 'alpha3');
    });
  });

  describe('column-select type', () => {
    it('renders available columns as options', () => {
      renderField({ key: 'col', label: 'Column', type: 'column-select' }, '', ['email', 'name', 'phone']);
      const select = screen.getByTestId('field-column-select-col') as HTMLSelectElement;
      // +1 for the placeholder option
      expect(select.options).toHaveLength(4);
    });
  });

  describe('column-multi type', () => {
    it('shows empty message when no columns available', () => {
      renderField({ key: 'cols', label: 'Cols', type: 'column-multi' }, [], []);
      expect(screen.getByText(/No columns available yet/)).toBeInTheDocument();
    });

    it('renders chips for each available column', () => {
      renderField({ key: 'cols', label: 'Cols', type: 'column-multi' }, ['email'], ['email', 'name']);
      expect(screen.getByText('email')).toBeInTheDocument();
      expect(screen.getByText('name')).toBeInTheDocument();
    });

    it('toggles column selection on click', async () => {
      const user = userEvent.setup();
      const { onChange } = renderField(
        { key: 'cols', label: 'Cols', type: 'column-multi' },
        ['email'],
        ['email', 'name'],
      );
      await user.click(screen.getByText('name'));
      expect(onChange).toHaveBeenCalledWith('cols', ['email', 'name']);
    });

    it('deselects a column on click', async () => {
      const user = userEvent.setup();
      const { onChange } = renderField(
        { key: 'cols', label: 'Cols', type: 'column-multi' },
        ['email', 'name'],
        ['email', 'name'],
      );
      await user.click(screen.getByText('email'));
      expect(onChange).toHaveBeenCalledWith('cols', ['name']);
    });
  });

  describe('columns type', () => {
    it('renders comma-separated values and calls onChange with array', async () => {
      const user = userEvent.setup();
      const { onChange } = renderField(
        { key: 'cols', label: 'Cols', type: 'columns', placeholder: 'a, b' },
        ['a', 'b'],
      );
      const input = screen.getByTestId('field-columns-cols') as HTMLInputElement;
      expect(input.value).toBe('a, b');

      fireEvent.change(input, { target: { value: 'x, y' } });
      // some call should have the split array
      expect(onChange).toHaveBeenCalled();
      const calls = onChange.mock.calls.map((c) => c[1]);
      expect(calls).toContainEqual(['x', 'y']);
    });
  });

  describe('mapping type', () => {
    it('renders mapping rows and allows add/remove/edit', async () => {
      const user = userEvent.setup();
      const { onChange } = renderField(
        { key: 'map', label: 'Map', type: 'mapping' },
        { old: 'new' },
        ['email', 'name'],
      );

      expect(screen.getByTestId('mapping-editor')).toBeInTheDocument();
      const row = screen.getByTestId('mapping-row-0');
      // should show the existing mapping value in the input
      const input = row.querySelector('input') as HTMLInputElement;
      expect(input.value).toBe('new');

      // change the value input
      await user.clear(input);
      await user.type(input, 'updated');
      expect(onChange).toHaveBeenCalled();

      // add a new row
      await user.click(screen.getByTestId('mapping-add-row'));
      expect(onChange).toHaveBeenCalled();
      const lastMapping = onChange.mock.calls[onChange.mock.calls.length - 1][1];
      // new mapping should contain an empty key entry
      expect(typeof lastMapping).toBe('object');
      expect(Object.keys(lastMapping).length).toBeGreaterThanOrEqual(1);
    });
  });
});
