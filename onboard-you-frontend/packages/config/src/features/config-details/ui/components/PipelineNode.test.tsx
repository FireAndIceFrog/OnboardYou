import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { renderWithProviders } from '@/shared/test/testWrapper';
import { PipelineNode } from './PipelineNode';

/**
 * PipelineNode is rendered inside React Flow which requires a ReactFlowProvider.
 * We also wrap with renderWithProviders to supply the Redux store that
 * PipelineNode uses for validationErrors selection.
 */
function renderNode(data: Record<string, unknown>) {
  return renderWithProviders(
    <ReactFlowProvider>
      <PipelineNode
        id="test-node"
        data={data}
        type="logic"
        // minimal NodeProps stubs
        {...({
          dragging: false,
          zIndex: 0,
          isConnectable: true,
          positionAbsoluteX: 0,
          positionAbsoluteY: 0,
          selected: false,
          deletable: false,
          selectable: true,
          parentId: undefined,
          sourcePosition: undefined,
          targetPosition: undefined,
        } as any)}
      />
    </ReactFlowProvider>,
    {
      preloadedState: {
        configDetails: {
          config: null,
          nodes: [],
          edges: [],
          selectedNode: null,
          isLoading: false,
          isSaving: false,
          isDeleting: false,
          isValidating: false,
          error: null,
          chatOpen: false,
          addStepPanelOpen: false,
          validationResult: null,
          validationErrors: {},
        } as any,
      },
    },
  );
}

describe('PipelineNode', () => {
  it('renders an ingestion node with green styling', () => {
    renderNode({ category: 'ingestion', actionType: 'csv_hris_connector', label: 'CSV Upload' });
    const node = screen.getByTestId('pipeline-node-ingestion');
    expect(node).toBeInTheDocument();
    expect(screen.getByText('CSV Upload')).toBeInTheDocument();
    expect(screen.getByText('📥')).toBeInTheDocument();
  });

  it('renders a logic node with blue styling', () => {
    renderNode({ category: 'logic', actionType: 'rename_column', label: 'Rename Fields' });
    const node = screen.getByTestId('pipeline-node-logic');
    expect(node).toBeInTheDocument();
    expect(screen.getAllByText('Rename Fields').length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('⚙️')).toBeInTheDocument();
  });

  it('renders an egress node with orange styling', () => {
    renderNode({ category: 'egress', actionType: 'api_dispatcher', label: 'Send to API' });
    const node = screen.getByTestId('pipeline-node-egress');
    expect(node).toBeInTheDocument();
    expect(screen.getAllByText('Send to API').length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('📤')).toBeInTheDocument();
  });

  it('falls back to logic styling for unknown category', () => {
    renderNode({ category: 'unknown', actionType: 'test', label: 'Test Node' });
    // Unknown category should use the default style
    const node = screen.getByTestId('pipeline-node-unknown');
    expect(node).toBeInTheDocument();
    expect(screen.getByText('🔧')).toBeInTheDocument();
  });

  it('displays the action type as a business label badge', () => {
    renderNode({ category: 'logic', actionType: 'pii_masking', label: 'Mask PII' });
    // businessLabel converts pii_masking → a human-friendly name
    expect(screen.getByText('Mask PII')).toBeInTheDocument();
  });
});
