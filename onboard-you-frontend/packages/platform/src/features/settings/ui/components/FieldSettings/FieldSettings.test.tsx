import { describe, it, expect } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { FieldSettings } from './FieldSettings';
import { DEFAULT_EGRESS_SETTINGS } from '../../../domain/types';
import { LoadingStatus } from '../../../state/settingsSlice';
import type { RootState } from '@/store';

function preloaded(overrides: Record<string, unknown> = {}): Partial<RootState> {
  return {
    settings: {
      settings: DEFAULT_EGRESS_SETTINGS,
      saved: false,
      dirty: false,
      loadingStatus: LoadingStatus.Succeeded,
      isSaving: false,
      error: null,
      showAdvanced: false,
      wizardStep: 1,
      ...overrides,
    },
  } as unknown as Partial<RootState>;
}

describe('FieldSettings', () => {
  it('renders the add button and empty state message', () => {
    renderWithProviders(<FieldSettings />, { preloadedState: preloaded() });
    expect(screen.getByRole('button', { name: /add field/i })).toBeInTheDocument();
    expect(screen.getByText(/no fields defined yet/i)).toBeInTheDocument();
  });

  it('clicking Add Field creates a new row with default type string', async () => {
    const user = userEvent.setup();
    const { store } = renderWithProviders(<FieldSettings />, { preloadedState: preloaded() });

    await user.click(screen.getByRole('button', { name: /add field/i }));

    const state = (store.getState() as RootState).settings.settings;
    expect(state.bearer.schema).toHaveProperty('');
    expect(state.bearer.schema['']).toBe('string');
    // Row should now be visible
    expect(screen.queryByText(/no fields defined yet/i)).not.toBeInTheDocument();
  });

  it('changing field name updates the schema key', async () => {
    const settings = {
      ...DEFAULT_EGRESS_SETTINGS,
      bearer: { ...DEFAULT_EGRESS_SETTINGS.bearer, schema: { name: 'string' } },
    };
    const { store } = renderWithProviders(<FieldSettings />, {
      preloadedState: preloaded({ settings }),
    });

    const nameInput = screen.getByDisplayValue('name');
    fireEvent.change(nameInput, { target: { value: 'email' } });
    fireEvent.blur(nameInput);

    await waitFor(() => {
      const schema = (store.getState() as RootState).settings.settings.bearer.schema;
      expect(schema).toHaveProperty('email');
      expect(schema).not.toHaveProperty('name');
    });
  });

  it('changing field type updates the schema value', async () => {
    const user = userEvent.setup();
    const settings = {
      ...DEFAULT_EGRESS_SETTINGS,
      bearer: { ...DEFAULT_EGRESS_SETTINGS.bearer, schema: { age: 'string' } },
    };
    const { store } = renderWithProviders(<FieldSettings />, {
      preloadedState: preloaded({ settings }),
    });

    const select = screen.getByDisplayValue('String');
    await user.selectOptions(select, 'number');

    const schema = (store.getState() as RootState).settings.settings.bearer.schema;
    expect(schema.age).toBe('number');
  });

  it('clicking delete removes the field', async () => {
    const user = userEvent.setup();
    const settings = {
      ...DEFAULT_EGRESS_SETTINGS,
      bearer: {
        ...DEFAULT_EGRESS_SETTINGS.bearer,
        schema: { foo: 'string', bar: 'number' },
      },
    };
    const { store } = renderWithProviders(<FieldSettings />, {
      preloadedState: preloaded({ settings }),
    });

    // Delete the first field (foo)
    const deleteButtons = screen.getAllByRole('button', { name: /delete/i });
    await user.click(deleteButtons[0]);

    const schema = (store.getState() as RootState).settings.settings.bearer.schema;
    expect(schema).not.toHaveProperty('foo');
    expect(schema).toHaveProperty('bar');
  });

  it('renders the advanced body path input when showAdvanced is true', () => {
    renderWithProviders(<FieldSettings />, {
      preloadedState: preloaded({ showAdvanced: true }),
    });
    expect(screen.getByPlaceholderText('e.g. data.items')).toBeInTheDocument();
  });

  it('updates body path in store', async () => {
    const user = userEvent.setup();
    const { store } = renderWithProviders(<FieldSettings />, {
      preloadedState: preloaded({ showAdvanced: true }),
    });

    const bodyInput = screen.getByPlaceholderText('e.g. data.items');
    await user.type(bodyInput, 'data.foo');

    expect((store.getState() as RootState).settings.settings.bearer.bodyPath).toBe('data.foo');
  });
});