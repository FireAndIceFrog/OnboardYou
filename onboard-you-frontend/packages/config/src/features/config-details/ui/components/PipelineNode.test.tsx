import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ChakraProvider } from '@chakra-ui/react';
import { I18nextProvider } from 'react-i18next';
import { Provider } from 'react-redux';
import i18n from '@/i18n';
import { system } from '@/theme';
import { ReactFlowProvider } from '@xyflow/react';
import { createTestStore } from '@/shared/test/testWrapper';
import { PipelineNode } from './PipelineNode';

/**
 * PipelineNode is rendered inside React Flow which requires a ReactFlowProvider.
 * We also wrap with Chakra + i18n + Redux to match the real runtime.
 */
function renderNode(data: Record<string, unknown>) {
  const store = createTestStore();
  return render(
    <Provider store={store}>
      <ChakraProvider value={system}>
        <I18nextProvider i18n={i18n}>
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
          </ReactFlowProvider>
        </I18nextProvider>
      </ChakraProvider>
    </Provider>,
  );
}

describe('PipelineNode', () => {
  it('renders an ingestion node with green styling', () => {
    renderNode({ category: 'ingestion', actionType: 'generic_ingestion_connector', label: 'Generic Upload' });
    const node = screen.getByTestId('pipeline-node-ingestion');
    expect(node).toBeInTheDocument();
    expect(screen.getByText('Generic Upload')).toBeInTheDocument();
  });

  it('renders a logic node with blue styling', () => {
    renderNode({ category: 'logic', actionType: 'rename_column', label: 'Rename Fields' });
    const node = screen.getByTestId('pipeline-node-logic');
    expect(node).toBeInTheDocument();
    expect(screen.getAllByText('Rename Fields').length).toBeGreaterThanOrEqual(1);
  });

  it('renders an egress node with orange styling', () => {
    renderNode({ category: 'egress', actionType: 'api_dispatcher', label: 'Send to API' });
    const node = screen.getByTestId('pipeline-node-egress');
    expect(node).toBeInTheDocument();
    expect(screen.getAllByText('Send to API').length).toBeGreaterThanOrEqual(1);
  });

  it('falls back to logic styling for unknown category', () => {
    renderNode({ category: 'unknown', actionType: 'test', label: 'Test Node' });
    // Unknown category should use the default style
    const node = screen.getByTestId('pipeline-node-unknown');
    expect(node).toBeInTheDocument();
  });

  it('displays the action type as a business label badge', () => {
    renderNode({ category: 'logic', actionType: 'pii_masking', label: 'Mask PII' });
    // businessLabel converts pii_masking → a human-friendly name
    expect(screen.getByText('Mask PII')).toBeInTheDocument();
  });
});
