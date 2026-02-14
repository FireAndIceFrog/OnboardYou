import type { Node, Edge } from '@xyflow/react';
import type { PipelineConfig } from '@/shared/domain/types';
import type { ConfigDetailsState } from '../domain/types';

export type ConfigDetailsAction =
  | { type: 'FETCH_START' }
  | { type: 'FETCH_SUCCESS'; payload: { config: PipelineConfig; nodes: Node[]; edges: Edge[] } }
  | { type: 'FETCH_ERROR'; payload: string }
  | { type: 'SET_NODES'; payload: Node[] }
  | { type: 'SET_EDGES'; payload: Edge[] }
  | { type: 'SELECT_NODE'; payload: Node }
  | { type: 'DESELECT_NODE' }
  | { type: 'TOGGLE_CHAT' }
  | { type: 'SET_CHAT_OPEN'; payload: boolean };

export const configDetailsInitialState: ConfigDetailsState = {
  config: null,
  nodes: [],
  edges: [],
  selectedNode: null,
  isLoading: false,
  error: null,
  chatOpen: false,
};

export function configDetailsReducer(
  state: ConfigDetailsState,
  action: ConfigDetailsAction,
): ConfigDetailsState {
  switch (action.type) {
    case 'FETCH_START':
      return { ...state, isLoading: true, error: null };

    case 'FETCH_SUCCESS':
      return {
        ...state,
        isLoading: false,
        config: action.payload.config,
        nodes: action.payload.nodes,
        edges: action.payload.edges,
        error: null,
      };

    case 'FETCH_ERROR':
      return { ...state, isLoading: false, error: action.payload };

    case 'SET_NODES':
      return { ...state, nodes: action.payload };

    case 'SET_EDGES':
      return { ...state, edges: action.payload };

    case 'SELECT_NODE':
      return { ...state, selectedNode: action.payload };

    case 'DESELECT_NODE':
      return { ...state, selectedNode: null };

    case 'TOGGLE_CHAT':
      return { ...state, chatOpen: !state.chatOpen };

    case 'SET_CHAT_OPEN':
      return { ...state, chatOpen: action.payload };

    default:
      return state;
  }
}
