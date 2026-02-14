import type { Node, Edge } from '@xyflow/react';
import type { PipelineDefinition } from '@/shared/domain/types';

const NODE_WIDTH = 240;
const NODE_GAP_X = 300;
const NODE_GAP_Y = 120;
const START_X = 50;
const START_Y = 100;

/**
 * Converts a PipelineDefinition into React Flow nodes and edges
 * using topological level assignment for layout.
 */
export function convertToFlow(pipeline: PipelineDefinition): { nodes: Node[]; edges: Edge[] } {
  const nodes: Node[] = [];
  const edges: Edge[] = [];

  // 1. Ingestion node — leftmost
  const ingestionNode: Node = {
    id: 'ingestion',
    type: 'ingestion',
    position: { x: START_X, y: START_Y },
    data: {
      label: pipeline.ingestion.source,
      stageType: pipeline.ingestion.type,
      config: pipeline.ingestion.config,
    },
  };
  nodes.push(ingestionNode);

  // 2. Build dependency levels for transformations (topological sort)
  const transforms = pipeline.transformations;
  const levels = assignLevels(transforms);
  const maxLevel = Math.max(...Object.values(levels), 0);

  // Group transforms by level
  const levelGroups: Map<number, typeof transforms> = new Map();
  for (const tx of transforms) {
    const level = levels[tx.id] ?? 0;
    if (!levelGroups.has(level)) levelGroups.set(level, []);
    levelGroups.get(level)!.push(tx);
  }

  // Position transformation nodes
  for (const [level, group] of levelGroups) {
    const x = START_X + (level + 1) * NODE_GAP_X;
    const groupHeight = group.length * NODE_GAP_Y;
    const startY = START_Y - groupHeight / 2 + NODE_GAP_Y / 2;

    group.forEach((tx, idx) => {
      const txNode: Node = {
        id: tx.id,
        type: 'transformation',
        position: { x, y: startY + idx * NODE_GAP_Y },
        data: {
          label: tx.name,
          stageType: tx.type,
          config: tx.config,
        },
      };
      nodes.push(txNode);

      // Create edges from dependencies
      if (tx.dependsOn.length === 0) {
        // Connect from ingestion
        edges.push({
          id: `edge-ingestion-${tx.id}`,
          source: 'ingestion',
          target: tx.id,
          animated: true,
          style: { stroke: '#2563EB', strokeWidth: 2 },
        });
      } else {
        tx.dependsOn.forEach((depId) => {
          edges.push({
            id: `edge-${depId}-${tx.id}`,
            source: depId,
            target: tx.id,
            animated: true,
            style: { stroke: '#7C3AED', strokeWidth: 2 },
          });
        });
      }
    });
  }

  // 3. Egress node — rightmost
  const egressX = START_X + (maxLevel + 2) * NODE_GAP_X;
  const egressNode: Node = {
    id: 'egress',
    type: 'egress',
    position: { x: egressX, y: START_Y },
    data: {
      label: pipeline.egress.destination,
      stageType: pipeline.egress.type,
      config: pipeline.egress.config,
    },
  };
  nodes.push(egressNode);

  // Connect last-level transforms to egress
  const lastLevelTransforms = levelGroups.get(maxLevel) ?? [];
  if (lastLevelTransforms.length > 0) {
    lastLevelTransforms.forEach((tx) => {
      edges.push({
        id: `edge-${tx.id}-egress`,
        source: tx.id,
        target: 'egress',
        animated: true,
        style: { stroke: '#10B981', strokeWidth: 2 },
      });
    });
  } else {
    // No transforms — connect ingestion directly to egress
    edges.push({
      id: 'edge-ingestion-egress',
      source: 'ingestion',
      target: 'egress',
      animated: true,
      style: { stroke: '#10B981', strokeWidth: 2 },
    });
  }

  return { nodes, edges };
}

/** Topological level assignment via DFS */
function assignLevels(
  transforms: { id: string; dependsOn: string[] }[],
): Record<string, number> {
  const levels: Record<string, number> = {};
  const visited = new Set<string>();

  function visit(id: string): number {
    if (visited.has(id)) return levels[id] ?? 0;
    visited.add(id);

    const tx = transforms.find((t) => t.id === id);
    if (!tx) return 0;

    if (tx.dependsOn.length === 0) {
      levels[id] = 0;
      return 0;
    }

    const maxDep = Math.max(...tx.dependsOn.map((depId) => visit(depId)));
    levels[id] = maxDep + 1;
    return levels[id];
  }

  transforms.forEach((tx) => visit(tx.id));
  return levels;
}

// Re-export NODE_WIDTH for potential external use
export { NODE_WIDTH, NODE_GAP_X, NODE_GAP_Y };
