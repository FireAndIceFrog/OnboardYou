import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ReactFlowProvider, Position, type EdgeProps } from '@xyflow/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { AddButtonEdge } from './AddButtonEdge';

// EdgeLabelRenderer uses a portal that doesn't mount in jsdom — render inline.
vi.mock('@xyflow/react', async () => {
  const actual = await vi.importActual<typeof import('@xyflow/react')>('@xyflow/react');
  return {
    ...actual,
    EdgeLabelRenderer: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  };
});

const BASE_EDGE_PROPS: EdgeProps = {
  id: 'edge-step-1-step-2',
  source: 'step-1',
  target: 'step-2',
  sourceX: 0,
  sourceY: 100,
  targetX: 300,
  targetY: 100,
  sourcePosition: Position.Right,
  targetPosition: Position.Left,
  data: {},
  selected: false,
  animated: true,
  markerEnd: undefined,
  markerStart: undefined,
  interactionWidth: 20,
  sourceHandleId: null,
  targetHandleId: null,
  pathOptions: undefined,
  style: { stroke: '#2563EB', strokeWidth: 2 },
  label: undefined,
  labelStyle: undefined,
  labelShowBg: undefined,
  labelBgStyle: undefined,
  labelBgPadding: undefined,
  labelBgBorderRadius: undefined,
  deletable: undefined,
  selectable: undefined,
  focusable: undefined,
};

/** Nodes needed in Redux state so the edge can find its target index. */
const PRELOADED_STATE = {
  configDetails: {
    config: null,
    nodes: [
      { id: 'step-1', type: 'ingestion', position: { x: 50, y: 100 }, data: {} },
      { id: 'step-2', type: 'logic', position: { x: 350, y: 100 }, data: {} },
      { id: 'step-3', type: 'egress', position: { x: 650, y: 100 }, data: {} },
    ],
    edges: [],
    selectedNode: null,
    isLoading: false,
    isSaving: false,
    isDeleting: false,
    isValidating: false,
    error: null,
    chatOpen: false,
    addStepPanelOpen: false,
    insertIndex: null,
    validationResult: null,
    validationErrors: {},
  },
};

function renderEdge(
  overrides: Partial<EdgeProps> = {},
  preloadedState = PRELOADED_STATE,
) {
  return renderWithProviders(
    <ReactFlowProvider>
      <AddButtonEdge {...BASE_EDGE_PROPS} {...overrides} />
    </ReactFlowProvider>,
    { preloadedState: preloadedState as any },
  );
}

describe('AddButtonEdge', () => {
  it('renders the add-step button', () => {
    renderEdge();
    expect(screen.getByRole('button', { name: 'Add step' })).toBeInTheDocument();
  });

  it('starts with the button hidden (opacity 0)', () => {
    renderEdge();
    const btn = screen.getByRole('button', { name: 'Add step' });
    expect(btn.style.opacity).toBe('0');
  });

  it('shows the button on mouse enter and hides on mouse leave', () => {
    renderEdge();
    const btn = screen.getByRole('button', { name: 'Add step' });

    fireEvent.mouseEnter(btn);
    expect(btn.style.opacity).toBe('1');

    fireEvent.mouseLeave(btn);
    expect(btn.style.opacity).toBe('0');
  });

  it('dispatches openAddStepAtIndex on click', async () => {
    const user = userEvent.setup();
    const { store } = renderEdge();

    await user.click(screen.getByRole('button', { name: 'Add step' }));

    const state = (store.getState() as any).configDetails;
    expect(state.addStepPanelOpen).toBe(true);
    expect(state.insertIndex).toBe(1); // step-2 is at index 1
  });

  it('dispatches with correct index for a later edge', async () => {
    const user = userEvent.setup();
    const { store } = renderEdge({ id: 'edge-step-2-step-3', source: 'step-2', target: 'step-3' });

    await user.click(screen.getByRole('button', { name: 'Add step' }));

    const state = (store.getState() as any).configDetails;
    expect(state.addStepPanelOpen).toBe(true);
    expect(state.insertIndex).toBe(2); // step-3 is at index 2
  });

  it('does not dispatch when target is the first node (index 0)', async () => {
    const user = userEvent.setup();
    // Edge pointing at step-1 which is at index 0
    const { store } = renderEdge({ id: 'edge-phantom-step-1', source: 'phantom', target: 'step-1' });

    await user.click(screen.getByRole('button', { name: 'Add step' }));

    const state = (store.getState() as any).configDetails;
    expect(state.addStepPanelOpen).toBe(false);
    expect(state.insertIndex).toBeNull();
  });

  it('deselects any selected node when opening the add panel', async () => {
    const user = userEvent.setup();
    const stateWithSelection = {
      ...PRELOADED_STATE,
      configDetails: {
        ...PRELOADED_STATE.configDetails,
        selectedNode: PRELOADED_STATE.configDetails.nodes[0],
      },
    };
    const { store } = renderEdge({}, stateWithSelection);

    await user.click(screen.getByRole('button', { name: 'Add step' }));

    const state = (store.getState() as any).configDetails;
    expect(state.selectedNode).toBeNull();
  });
});
