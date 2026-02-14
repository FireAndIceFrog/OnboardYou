import { useState, useEffect, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import {
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
  type OnNodesChange,
  type OnEdgesChange,
} from '@xyflow/react';
import { useGlobal } from '@/hooks/useGlobal';
import type { PipelineConfig, PipelineDefinition } from '@/types';

const NODE_WIDTH = 220;
const NODE_HEIGHT = 100;
const HORIZONTAL_GAP = 280;
const VERTICAL_GAP = 140;

function buildNodesAndEdges(pipeline: PipelineDefinition): {
  nodes: Node[];
  edges: Edge[];
} {
  const nodes: Node[] = [];
  const edges: Edge[] = [];

  // Ingestion node — leftmost
  const ingestionNodeId = 'ingestion';
  nodes.push({
    id: ingestionNodeId,
    type: 'ingestion',
    position: { x: 0, y: 0 },
    data: {
      label: 'Ingestion',
      stageType: pipeline.ingestion.type,
      source: pipeline.ingestion.source,
      config: pipeline.ingestion.config,
    },
  });

  // Build a dependency graph for transformations
  const txMap = new Map(pipeline.transformations.map((t) => [t.id, t]));
  const depths = new Map<string, number>();

  function getDepth(txId: string): number {
    if (depths.has(txId)) return depths.get(txId)!;
    const tx = txMap.get(txId);
    if (!tx || tx.dependsOn.length === 0) {
      depths.set(txId, 0);
      return 0;
    }
    const maxParentDepth = Math.max(...tx.dependsOn.map((dep) => getDepth(dep)));
    const d = maxParentDepth + 1;
    depths.set(txId, d);
    return d;
  }

  pipeline.transformations.forEach((t) => getDepth(t.id));

  // Group transformations by depth level
  const depthBuckets = new Map<number, string[]>();
  pipeline.transformations.forEach((t) => {
    const d = depths.get(t.id) ?? 0;
    if (!depthBuckets.has(d)) depthBuckets.set(d, []);
    depthBuckets.get(d)!.push(t.id);
  });

  const sortedDepths = Array.from(depthBuckets.keys()).sort((a, b) => a - b);

  sortedDepths.forEach((depth, _colIdx) => {
    const ids = depthBuckets.get(depth)!;
    const x = (depth + 1) * HORIZONTAL_GAP;
    ids.forEach((txId, rowIdx) => {
      const tx = txMap.get(txId)!;
      const yOffset = rowIdx * VERTICAL_GAP - ((ids.length - 1) * VERTICAL_GAP) / 2;
      nodes.push({
        id: txId,
        type: 'transformation',
        position: { x, y: yOffset },
        data: {
          label: tx.name,
          stageType: tx.type,
          config: tx.config,
        },
      });

      // Edges from dependencies (or from ingestion if no deps)
      if (tx.dependsOn.length === 0) {
        edges.push({
          id: `e-${ingestionNodeId}-${txId}`,
          source: ingestionNodeId,
          target: txId,
          animated: true,
          style: { stroke: '#94A3B8', strokeWidth: 2 },
        });
      } else {
        tx.dependsOn.forEach((depId) => {
          edges.push({
            id: `e-${depId}-${txId}`,
            source: depId,
            target: txId,
            animated: true,
            style: { stroke: '#94A3B8', strokeWidth: 2 },
          });
        });
      }
    });
  });

  // Egress node — rightmost
  const maxDepth = sortedDepths.length > 0 ? sortedDepths[sortedDepths.length - 1] : -1;
  const egressX = (maxDepth + 2) * HORIZONTAL_GAP;
  const egressNodeId = 'egress';
  nodes.push({
    id: egressNodeId,
    type: 'egress',
    position: { x: egressX, y: 0 },
    data: {
      label: 'Egress',
      stageType: pipeline.egress.type,
      destination: pipeline.egress.destination,
      config: pipeline.egress.config,
    },
  });

  // Connect last transformations to egress
  const lastDepthIds = depthBuckets.get(maxDepth) ?? [];
  if (lastDepthIds.length === 0) {
    // No transformations — connect ingestion directly to egress
    edges.push({
      id: `e-${ingestionNodeId}-${egressNodeId}`,
      source: ingestionNodeId,
      target: egressNodeId,
      animated: true,
      style: { stroke: '#94A3B8', strokeWidth: 2 },
    });
  } else {
    lastDepthIds.forEach((txId) => {
      edges.push({
        id: `e-${txId}-${egressNodeId}`,
        source: txId,
        target: egressNodeId,
        animated: true,
        style: { stroke: '#94A3B8', strokeWidth: 2 },
      });
    });
  }

  // Center all nodes vertically by adding an offset so the bounding box is centered
  if (nodes.length > 0) {
    const minY = Math.min(...nodes.map((n) => n.position.y));
    const maxY = Math.max(...nodes.map((n) => n.position.y + NODE_HEIGHT));
    const centerOffset = -(minY + maxY) / 2;
    nodes.forEach((n) => {
      n.position.y += centerOffset + 200;
    });
  }

  return { nodes, edges };
}

interface UseConfigDetailsReturn {
  config: PipelineConfig | null;
  nodes: Node[];
  edges: Edge[];
  onNodesChange: OnNodesChange;
  onEdgesChange: OnEdgesChange;
  isLoading: boolean;
  error: string | null;
  selectedNode: Node | null;
  setSelectedNode: (node: Node | null) => void;
}

export function useConfigDetails(): UseConfigDetailsReturn {
  const { configId } = useParams<{ configId: string }>();
  const { apiClient, showNotification } = useGlobal();
  const [config, setConfig] = useState<PipelineConfig | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  const fetchConfig = useCallback(async () => {
    if (!configId) return;
    setIsLoading(true);
    setError(null);
    try {
      const data = await apiClient.get<PipelineConfig>(`/configs/${configId}`);
      setConfig(data);
      const { nodes: n, edges: e } = buildNodesAndEdges(data.pipeline);
      setNodes(n);
      setEdges(e);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to load configuration';
      setError(message);
      showNotification(message, 'error');
    } finally {
      setIsLoading(false);
    }
  }, [configId, apiClient, showNotification, setNodes, setEdges]);

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  return {
    config,
    nodes,
    edges,
    onNodesChange,
    onEdgesChange,
    isLoading,
    error,
    selectedNode,
    setSelectedNode,
  };
}
