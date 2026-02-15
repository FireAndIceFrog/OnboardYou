import { describe, it, expect } from 'vitest';
import { convertToFlow } from './pipelineLayoutService';
import type { Manifest } from '@/shared/domain/types';

describe('convertToFlow', () => {
  it('returns no nodes or edges for an empty pipeline', () => {
    const manifest: Manifest = { version: '1', actions: [] };
    const { nodes, edges } = convertToFlow(manifest);
    expect(nodes).toHaveLength(0);
    expect(edges).toHaveLength(0);
  });

  it('returns one node and no edges for a single action', () => {
    const manifest: Manifest = {
      version: '1',
      actions: [
        { id: 'step-1', actionType: 'csv_hris_connector', config: { name: 'Import CSV' } },
      ],
    };
    const { nodes, edges } = convertToFlow(manifest);
    expect(nodes).toHaveLength(1);
    expect(edges).toHaveLength(0);
    expect(nodes[0].id).toBe('step-1');
    expect(nodes[0].data.actionType).toBe('csv_hris_connector');
  });

  it('produces a chain of nodes with connecting edges for multiple actions', () => {
    const manifest: Manifest = {
      version: '1',
      actions: [
        { id: 'step-1', actionType: 'csv_hris_connector', config: { name: 'Import' } },
        { id: 'step-2', actionType: 'pii_masking', config: { name: 'Mask PII' } },
        { id: 'step-3', actionType: 'api_dispatch', config: { name: 'Send to API' } },
      ],
    };
    const { nodes, edges } = convertToFlow(manifest);

    expect(nodes).toHaveLength(3);
    expect(edges).toHaveLength(2);

    // Edges connect sequentially
    expect(edges[0].source).toBe('step-1');
    expect(edges[0].target).toBe('step-2');
    expect(edges[1].source).toBe('step-2');
    expect(edges[1].target).toBe('step-3');
  });

  it('positions nodes with correct horizontal spacing', () => {
    const manifest: Manifest = {
      version: '1',
      actions: [
        { id: 'a', actionType: 'csv_hris_connector', config: {} },
        { id: 'b', actionType: 'pii_masking', config: {} },
        { id: 'c', actionType: 'api_dispatch', config: {} },
      ],
    };
    const { nodes } = convertToFlow(manifest);

    // Nodes should be spaced 300px apart (NODE_GAP_X), starting at x=50 (START_X)
    expect(nodes[0].position.x).toBe(50);
    expect(nodes[1].position.x).toBe(350);
    expect(nodes[2].position.x).toBe(650);

    // All nodes at the same y position (START_Y = 100)
    expect(nodes[0].position.y).toBe(100);
    expect(nodes[1].position.y).toBe(100);
    expect(nodes[2].position.y).toBe(100);
  });

  it('assigns the correct node type based on action category', () => {
    const manifest: Manifest = {
      version: '1',
      actions: [
        { id: 'ing', actionType: 'csv_hris_connector', config: {} },
        { id: 'logic', actionType: 'pii_masking', config: {} },
        { id: 'eg', actionType: 'api_dispatcher', config: {} },
      ],
    };
    const { nodes } = convertToFlow(manifest);

    expect(nodes[0].type).toBe('ingestion');
    expect(nodes[1].type).toBe('logic');
    expect(nodes[2].type).toBe('egress');
  });
});
