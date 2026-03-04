import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { ActionEditPanel } from './ActionEditPanel';
import type { ConfigDetailsState } from '../../domain/types';

/** Minimal state with a selected node to render the panel. */
function makeState(overrides: Partial<ConfigDetailsState> = {}): {
  configDetails: ConfigDetailsState;
} {
  return {
    configDetails: {
      config: null,
      nodes: [],
      edges: [],
      selectedNode: {
        id: 'step-1',
        position: { x: 0, y: 0 },
        data: {
          actionId: 'step-1',
          actionType: 'rename_column',
          category: 'logic',
          label: 'Rename Fields',
          config: { mapping: { old_col: 'new_col' } },
        },
      },
      isLoading: false,
      isSaving: false,
      isDeleting: false,
      isValidating: false,
      error: null,
      chatOpen: false,
      addStepPanelOpen: false,
      validationResult: null,
      planSummary: null,
      isGeneratingPlan: false,
      viewMode: 'advanced',
      planStale: false,
      ...overrides,
    },
  };
}

describe('ActionEditPanel', () => {
  it('renders nothing when no node is selected', () => {
    renderWithProviders(<ActionEditPanel />, {
      preloadedState: makeState({ selectedNode: null }),
    });
    expect(screen.queryByTestId('action-edit-panel')).not.toBeInTheDocument();
  });

  it('renders the panel when a node is selected', () => {
    renderWithProviders(<ActionEditPanel />, {
      preloadedState: makeState(),
    });
    expect(screen.getByTestId('action-edit-panel')).toBeInTheDocument();
    expect(screen.getByText('Rename Fields')).toBeInTheDocument();
  });

  it('shows catalog description for known action types', () => {
    renderWithProviders(<ActionEditPanel />, {
      preloadedState: makeState(),
    });
    expect(
      screen.getByText(/Change column names to match the format/),
    ).toBeInTheDocument();
  });

  it('renders remove button for non-ingestion actions', () => {
    renderWithProviders(<ActionEditPanel />, {
      preloadedState: makeState(),
    });
    expect(screen.getByTestId('action-edit-remove')).toBeInTheDocument();
  });

  it('hides remove button for ingestion actions', () => {
    const state = makeState();
    (state.configDetails.selectedNode!.data as Record<string, unknown>).category = 'ingestion';
    renderWithProviders(<ActionEditPanel />, { preloadedState: state });
    expect(screen.queryByTestId('action-edit-remove')).not.toBeInTheDocument();
  });

  it('requires two clicks to remove (confirmation pattern)', async () => {
    const user = userEvent.setup();
    const { store } = renderWithProviders(<ActionEditPanel />, {
      preloadedState: makeState(),
    });

    const removeBtn = screen.getByTestId('action-edit-remove');
    expect(removeBtn).toHaveTextContent(/Remove this step/);

    // First click shows confirmation
    await user.click(removeBtn);
    expect(screen.getByTestId('action-edit-remove')).toHaveTextContent(/Click again to confirm/);
  });

  it('dispatches deselectNode when close is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ActionEditPanel />, {
      preloadedState: makeState(),
    });

    await user.click(screen.getByTestId('action-edit-close'));
    // After close, panel should disappear (selectedNode becomes null)
    expect(screen.queryByTestId('action-edit-panel')).not.toBeInTheDocument();
  });

  it('renders a custom panel for pii_masking action type', () => {
    const state = makeState();
    const node = state.configDetails.selectedNode!;
    (node.data as Record<string, unknown>).actionType = 'pii_masking';
    (node.data as Record<string, unknown>).label = 'Mask Sensitive Data';
    (node.data as Record<string, unknown>).config = {
      columns: [{ name: 'ssn', strategy: 'Zero' }],
    };
    renderWithProviders(<ActionEditPanel />, { preloadedState: state });
    expect(screen.getByTestId('pii-masking-panel')).toBeInTheDocument();
  });

  it('renders generic FieldEditor for actions without a custom panel', () => {
    renderWithProviders(<ActionEditPanel />, {
      preloadedState: makeState(),
    });
    // rename_column uses 'mapping' field type → MappingEditor should render
    expect(screen.getByTestId('mapping-editor')).toBeInTheDocument();
  });
});
