import {
  createSlice,
  createAsyncThunk,
  current,
  type PayloadAction,
} from '@reduxjs/toolkit';
import {
  applyNodeChanges,
  applyEdgeChanges,
  type Node,
  type Edge,
  type NodeChange,
  type EdgeChange,
} from '@xyflow/react';
import type { RootState } from '@/store';
import type { PipelineConfig, ActionConfig, ValidationResult } from '@/shared/domain/types';
import { actionCategory } from '@/shared/domain/types';
import type { ApiClient } from '@/shared/services';
import type { ConfigDetailsState } from '../domain/types';
import { fetchConfig, saveConfig as saveConfigService, validateConfig as validateConfigService } from '../services/configDetailsService';
import { convertToFlow } from '../services/pipelineLayoutService';

/* ── Layout constants ──────────────────────────────────────── */

const NODE_GAP_X = 300;
const START_X = 50;
const START_Y = 100;

const CATEGORY_STYLES: Record<string, { stroke: string }> = {
  ingestion: { stroke: '#2563EB' },
  logic: { stroke: '#7C3AED' },
  egress: { stroke: '#10B981' },
};

/* ── Initial state ─────────────────────────────────────────── */

const initialState: ConfigDetailsState = {
  config: null,
  nodes: [],
  edges: [],
  selectedNode: null,
  isLoading: false,
  error: null,
  chatOpen: false,
};

/* ── Async thunks ──────────────────────────────────────────── */

export const fetchConfigDetails = createAsyncThunk<
  { config: PipelineConfig; nodes: Node[]; edges: Edge[] },
  { apiClient: ApiClient; customerCompanyId: string }
>(
  'configDetails/fetchConfigDetails',
  async ({ apiClient, customerCompanyId }, { rejectWithValue }) => {
    try {
      const config = await fetchConfig(apiClient, customerCompanyId);
      const { nodes, edges } = convertToFlow(config.pipeline);
      return { config, nodes, edges };
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to load configuration';
      return rejectWithValue(message);
    }
  },
);

export const saveConfigThunk = createAsyncThunk<
  PipelineConfig,
  { apiClient: ApiClient; customerCompanyId: string; data: PipelineConfig }
>(
  'configDetails/saveConfig',
  async ({ apiClient, customerCompanyId, data }, { rejectWithValue }) => {
    try {
      return await saveConfigService(apiClient, customerCompanyId, data);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to save configuration';
      return rejectWithValue(message);
    }
  },
);

export const validateConfigThunk = createAsyncThunk<
  ValidationResult,
  { apiClient: ApiClient; customerCompanyId: string; data: PipelineConfig }
>(
  'configDetails/validateConfig',
  async ({ apiClient, customerCompanyId, data }, { rejectWithValue }) => {
    try {
      return await validateConfigService(apiClient, customerCompanyId, data);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to validate configuration';
      return rejectWithValue(message);
    }
  },
);

/* ── Slice ─────────────────────────────────────────────────── */

const configDetailsSlice = createSlice({
  name: 'configDetails',
  initialState,
  reducers: {
    setNodes(state, action: PayloadAction<Node[]>) {
      state.nodes = action.payload as typeof state.nodes;
    },
    setEdges(state, action: PayloadAction<Edge[]>) {
      state.edges = action.payload as typeof state.edges;
    },
    onNodesChange(state, action: PayloadAction<NodeChange[]>) {
      const plain = current(state).nodes;
      state.nodes = applyNodeChanges(action.payload, plain) as typeof state.nodes;
    },
    onEdgesChange(state, action: PayloadAction<EdgeChange[]>) {
      const plain = current(state).edges;
      state.edges = applyEdgeChanges(action.payload, plain) as typeof state.edges;
    },
    selectNode(state, action: PayloadAction<Node>) {
      state.selectedNode = action.payload as typeof state.selectedNode;
    },
    deselectNode(state) {
      state.selectedNode = null;
    },
    toggleChat(state) {
      state.chatOpen = !state.chatOpen;
    },
    setChatOpen(state, action: PayloadAction<boolean>) {
      state.chatOpen = action.payload;
    },
    addFlowAction(state, action: PayloadAction<ActionConfig>) {
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

      if (state.nodes.length > 0) {
        const prevNode = state.nodes[state.nodes.length - 1];
        const style = CATEGORY_STYLES[category] ?? CATEGORY_STYLES.logic;
        state.edges.push({
          id: `edge-${prevNode.id}-${actionCfg.id}`,
          source: prevNode.id,
          target: actionCfg.id,
          animated: true,
          style: { ...style, strokeWidth: 2 },
        } as typeof state.edges[number]);
      }

      state.nodes.push(newNode as typeof state.nodes[number]);
    },
    resetConfigDetails() {
      return initialState;
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(fetchConfigDetails.pending, (state) => {
        state.isLoading = true;
        state.error = null;
      })
      .addCase(fetchConfigDetails.fulfilled, (state, action) => {
        state.isLoading = false;
        state.config = action.payload.config;
        state.nodes = action.payload.nodes as typeof state.nodes;
        state.edges = action.payload.edges as typeof state.edges;
        state.error = null;
      })
      .addCase(fetchConfigDetails.rejected, (state, action) => {
        state.isLoading = false;
        state.error = (action.payload as string) ?? 'Failed to load configuration';
      })
      .addCase(saveConfigThunk.fulfilled, (state, action) => {
        state.config = action.payload;
      })
      .addCase(validateConfigThunk.rejected, (state, action) => {
        state.error = (action.payload as string) ?? 'Validation failed';
      });
  },
});

export const {
  setNodes,
  setEdges,
  onNodesChange,
  onEdgesChange,
  selectNode,
  deselectNode,
  toggleChat,
  setChatOpen,
  addFlowAction,
  resetConfigDetails,
} = configDetailsSlice.actions;

/* ── Selectors ─────────────────────────────────────────────── */

export const selectConfigDetails = (state: RootState) => state.configDetails;
export const selectConfig = (state: RootState) => state.configDetails.config;
export const selectNodes = (state: RootState) => state.configDetails.nodes;
export const selectEdges = (state: RootState) => state.configDetails.edges;
export const selectSelectedNode = (state: RootState) => state.configDetails.selectedNode;
export const selectIsChatOpen = (state: RootState) => state.configDetails.chatOpen;
export const selectConfigDetailsLoading = (state: RootState) => state.configDetails.isLoading;
export const selectConfigDetailsError = (state: RootState) => state.configDetails.error;

export default configDetailsSlice.reducer;
