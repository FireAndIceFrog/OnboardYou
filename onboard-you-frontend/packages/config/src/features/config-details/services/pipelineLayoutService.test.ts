import { describe, it, expect } from 'vitest';
import { convertToFlow } from './pipelineLayoutService';
import type { Manifest } from '@/generated/api';

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
        { id: 'step-1', action_type: 'csv_hris_connector', config: { csv_path: '/tmp/test.csv' } },
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
        { id: 'step-1', action_type: 'csv_hris_connector', config: { csv_path: '/tmp/test.csv' } },
        { id: 'step-2', action_type: 'pii_masking', config: { columns: [] } },
        { id: 'step-3', action_type: 'api_dispatcher', config: { Bearer: { destination_url: 'https://example.com' } } },
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
        { id: 'a', action_type: 'csv_hris_connector', config: { csv_path: '/tmp/a.csv' } },
        { id: 'b', action_type: 'pii_masking', config: { columns: [] } },
        { id: 'c', action_type: 'api_dispatcher', config: { Bearer: { destination_url: 'https://example.com' } } },
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
        { id: 'ing', action_type: 'csv_hris_connector', config: { csv_path: '/tmp/test.csv' } },
        { id: 'logic', action_type: 'pii_masking', config: { columns: [] } },
        { id: 'eg', action_type: 'api_dispatcher', config: { Bearer: { destination_url: 'https://example.com' } } },
      ],
    };
    const { nodes } = convertToFlow(manifest);

    expect(nodes[0].type).toBe('ingestion');
    expect(nodes[1].type).toBe('logic');
    expect(nodes[2].type).toBe('egress');
  });
});
