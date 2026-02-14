import type { Node, Edge } from '@xyflow/react';
import type { PipelineConfig } from '@/shared/domain/types';

export interface ConfigDetailsState {
  config: PipelineConfig | null;
  nodes: Node[];
  edges: Edge[];
  selectedNode: Node | null;
  isLoading: boolean;
  error: string | null;
  chatOpen: boolean;
}
