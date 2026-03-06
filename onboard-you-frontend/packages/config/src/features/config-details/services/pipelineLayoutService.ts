import type { Node, Edge } from '@xyflow/react';
import type { Manifest } from '@/shared/domain/types';
import { actionCategory, businessLabel } from '@/shared/domain/types';

const NODE_GAP_X = 300;
const START_X = 50;
const START_Y = 100;

const CATEGORY_STYLES: Record<string, { stroke: string }> = {
  ingestion: { stroke: '#2563EB' },
  logic: { stroke: '#7C3AED' },
  egress: { stroke: '#10B981' },
};

/**
 * Converts a Manifest (flat ordered list of actions) into
 * React Flow nodes and edges — laid out as a simple sequential chain.
 */
export function convertToFlow(manifest: Manifest): { nodes: Node[]; edges: Edge[] } {
  const nodes: Node[] = [];
  const edges: Edge[] = [];

  manifest.actions.forEach((action, idx) => {
    const category = actionCategory(action.action_type);

    const node: Node = {
      id: action.id,
      type: category,
      position: { x: START_X + idx * NODE_GAP_X, y: START_Y },
      data: {
        actionId: action.id,
        label: businessLabel(action.action_type),
        actionType: action.action_type,
        category,
        config: action.config,
      },
    };
    nodes.push(node);

    // Connect sequentially to the previous node
    if (idx > 0) {
      const prev = manifest.actions[idx - 1];
      const edgeColor = CATEGORY_STYLES[category] ?? CATEGORY_STYLES.logic;
      edges.push({
        id: `edge-${prev.id}-${action.id}`,
        source: prev.id,
        target: action.id,
        type: 'addButton',
        animated: true,
        style: { stroke: edgeColor.stroke, strokeWidth: 2 },
      });
    }
  });

  return { nodes, edges };
}
