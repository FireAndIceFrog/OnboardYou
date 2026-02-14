import type { Node, Edge } from '@xyflow/react';
import type { PipelineConfig, ActionConfig } from '@/shared/domain/types';
import type { ConfigDetailsState } from '../domain/types';
import { actionCategory } from '@/shared/domain/types';

const NODE_GAP_X = 300;
const START_X = 50;
const START_Y = 100;

const CATEGORY_STYLES: Record<string, { stroke: string }> = {
  ingestion: { stroke: '#2563EB' },
  logic: { stroke: '#7C3AED' },
  egress: { stroke: '#10B981' },
};

export type ConfigDetailsAction =
  | { type: 'FETCH_START' }
  | { type: 'FETCH_SUCCESS'; payload: { config: PipelineConfig; nodes: Node[]; edges: Edge[] } }
  | { type: 'FETCH_ERROR'; payload: string }
  | { type: 'SET_NODES'; payload: Node[] }
  | { type: 'SET_EDGES'; payload: Edge[] }
  | { type: 'SELECT_NODE'; payload: Node }
  | { type: 'DESELECT_NODE' }
  | { type: 'TOGGLE_CHAT' }
  | { type: 'SET_CHAT_OPEN'; payload: boolean }
  | { type: 'ADD_FLOW_ACTION'; payload: ActionConfig };

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

    case 'ADD_FLOW_ACTION': {
      const actionCfg = action.payload;
      const category = actionCategory(actionCfg.actionType);
      const idx = state.nodes.length;

      const newNode: Node = {
        id: actionCfg.id,
        type: category,
        position: { x: START_X + idx * NODE_GAP_X, y: START_Y },
        data: {
          label: (actionCfg.config.name as string) ?? actionCfg.id,
          actionType: actionCfg.actionType,
          category,
          config: actionCfg.config,
        },
      };

      const newEdges = [...state.edges];
      if (state.nodes.length > 0) {
        const prevNode = state.nodes[state.nodes.length - 1];
        const style = CATEGORY_STYLES[category] ?? CATEGORY_STYLES.logic;
        newEdges.push({
          id: `edge-${prevNode.id}-${actionCfg.id}`,
          source: prevNode.id,
          target: actionCfg.id,
          animated: true,
          style: { ...style, strokeWidth: 2 },
        });
      }

      return {
        ...state,
        nodes: [...state.nodes, newNode],
        edges: newEdges,
      };
    }

    default:
      return state;
  }
}
