import { describe, it, expect } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { FieldSettings } from './FieldSettings';
import { DEFAULT_EGRESS_SETTINGS } from '../../../domain/types';

describe('FieldSettings', () => {
  it('renders the schema textarea and body path input for bearer auth', () => {
    const { store } = renderWithProviders(<FieldSettings />);
    const textarea = screen.getByRole('textbox');
    expect(textarea).toBeInTheDocument();
    // default schema should be an empty object
    expect(textarea).toHaveValue('{}');

    // update the schema to valid JSON
    fireEvent.change(textarea, { target: { value: '{"foo":"bar"}' } });
    expect(store.getState().settings.settings.bearer.schema).toEqual({ foo: 'bar' });

    const bodyInput = screen.getByPlaceholderText('e.g. data.items');
    fireEvent.change(bodyInput, { target: { value: 'data.foo' } });
    expect(store.getState().settings.settings.bearer.bodyPath).toBe('data.foo');
  });
});