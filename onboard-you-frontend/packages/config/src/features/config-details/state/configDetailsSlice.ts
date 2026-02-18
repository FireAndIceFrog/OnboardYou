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
import type { RootState, ThunkExtra } from '@/store';
import type { PipelineConfig, ActionConfig, ValidationResult, ActionType, WorkdayConfig, CsvHrisConnectorConfig } from '@/generated/api';
import { actionCategory, businessLabel } from '@/shared/domain/types';
import type { ConfigDetailsState, ConnectionForm } from '../domain/types';
import { buildResponseGroup } from '../domain/types';
import { fetchConfig, createConfig as createConfigService, saveConfig as saveConfigService, validateConfig as validateConfigService } from '../services/configDetailsService';
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
  isSaving: false,
  error: null,
  chatOpen: false,
};

/* ── Async thunks ──────────────────────────────────────────── */

export const fetchConfigDetails = createAsyncThunk<
  { config: PipelineConfig; nodes: Node[]; edges: Edge[] },
  { customerCompanyId: string },
  { extra: ThunkExtra }
>(
  'configDetails/fetchConfigDetails',
  async ({ customerCompanyId }, { rejectWithValue }) => {
    try {
      const config = await fetchConfig(customerCompanyId);
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
  { customerCompanyId: string; data: PipelineConfig },
  { extra: ThunkExtra }
>(
  'configDetails/saveConfig',
  async ({ customerCompanyId, data }, { rejectWithValue }) => {
    try {
      return await saveConfigService(customerCompanyId, {
        name: data.name,
        cron: data.cron,
        pipeline: data.pipeline,
        image: data.image,
      });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to save configuration';
      return rejectWithValue(message);
    }
  },
);

export const createConfigThunk = createAsyncThunk<
  PipelineConfig,
  { customerCompanyId: string; data: PipelineConfig },
  { extra: ThunkExtra }
>(
  'configDetails/createConfig',
  async ({ customerCompanyId, data }, { rejectWithValue }) => {
    try {
      return await createConfigService(customerCompanyId, {
        name: data.name,
        cron: data.cron,
        pipeline: data.pipeline,
        image: data.image,
      });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to create configuration';
      return rejectWithValue(message);
    }
  },
);

export const validateConfigThunk = createAsyncThunk<
  ValidationResult,
  { customerCompanyId: string; data: PipelineConfig },
  { extra: ThunkExtra }
>(
  'configDetails/validateConfig',
  async ({ customerCompanyId, data }, { rejectWithValue }) => {
    try {
      return await validateConfigService(customerCompanyId, {
        name: data.name,
        cron: data.cron,
        pipeline: data.pipeline,
        image: data.image,
      });
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
      const category = actionCategory(actionCfg.action_type);
      const idx = state.nodes.length;

      const newNode: Node = {
        id: actionCfg.id,
        type: category,
        position: { x: START_X + idx * NODE_GAP_X, y: START_Y },
        data: {
          label: businessLabel(actionCfg.action_type),
          actionType: actionCfg.action_type,
          category,
          config: actionCfg.config,
        },
      };

      if (state.nodes.length > 0) {
        const prevNode = state.nodes[state.nodes.length - 1];
        const edgeColor = CATEGORY_STYLES[category] ?? CATEGORY_STYLES.logic;
        state.edges.push({
          id: `edge-${prevNode.id}-${actionCfg.id}`,
          source: prevNode.id,
          target: actionCfg.id,
          type: 'default',
          animated: true,
          style: { stroke: edgeColor.stroke, strokeWidth: 2 },
        } as typeof state.edges[number]);
      }

      state.nodes.push(newNode as typeof state.nodes[number]);
    },
    resetConfigDetails() {
      return initialState;
    },
    /**
     * Initialise the slice for a brand-new config created via the
     * connection wizard.  Builds a PipelineConfig with a single
     * ingestion action derived from the ConnectionForm.
     */
    initNewConfig(state, action: PayloadAction<ConnectionForm>) {
      const form = action.payload;

      // Map the wizard system id to the generated ActionType
      const actionType: ActionType =
        form.system === 'workday' ? 'workday_hris_connector' : 'csv_hris_connector';

      // Build connector-specific config payload
      const connectorConfig: WorkdayConfig | CsvHrisConnectorConfig =
        form.system === 'workday'
          ? {
              tenant_url: form.workday.tenantUrl,
              tenant_id: form.workday.tenantId,
              username: form.workday.username,
              password: form.workday.password,
              worker_count_limit: Number(form.workday.workerCountLimit) || 200,
              response_group: buildResponseGroup(form.workday.responseGroup),
            }
          : {
              filename: form.csv.filename,
              columns: form.csv.columns,
            };

      const ingestionAction: ActionConfig = {
        id: 'ingest',
        action_type: actionType,
        config: connectorConfig,
      };

      const newConfig: PipelineConfig = {
        name: form.displayName || 'New Configuration',
        cron: 'rate(1 day)',
        organizationId: '',
        customerCompanyId: '',
        pipeline: {
          version: '1.0',
          actions: [ingestionAction],
        },
      };

      const { nodes, edges } = convertToFlow(newConfig.pipeline);

      state.config = newConfig;
      state.nodes = nodes as typeof state.nodes;
      state.edges = edges as typeof state.edges;
      state.isLoading = false;
      state.error = null;
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
      .addCase(saveConfigThunk.pending, (state) => {
        state.isSaving = true;
        state.error = null;
      })
      .addCase(saveConfigThunk.fulfilled, (state, action) => {
        state.isSaving = false;
        state.config = action.payload;
      })
      .addCase(saveConfigThunk.rejected, (state, action) => {
        state.isSaving = false;
        state.error = (action.payload as string) ?? 'Failed to save configuration';
      })
      .addCase(createConfigThunk.pending, (state) => {
        state.isSaving = true;
        state.error = null;
      })
      .addCase(createConfigThunk.fulfilled, (state, action) => {
        state.isSaving = false;
        state.config = action.payload;
      })
      .addCase(createConfigThunk.rejected, (state, action) => {
        state.isSaving = false;
        state.error = (action.payload as string) ?? 'Failed to create configuration';
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
  initNewConfig,
} = configDetailsSlice.actions;

/* ── Selectors ─────────────────────────────────────────────── */

export const selectConfigDetails = (state: RootState) => state.configDetails;
export const selectConfig = (state: RootState) => state.configDetails.config;
export const selectNodes = (state: RootState) => state.configDetails.nodes;
export const selectEdges = (state: RootState) => state.configDetails.edges;
export const selectSelectedNode = (state: RootState) => state.configDetails.selectedNode;
export const selectIsChatOpen = (state: RootState) => state.configDetails.chatOpen;
export const selectConfigDetailsLoading = (state: RootState) => state.configDetails.isLoading;
export const selectConfigDetailsSaving = (state: RootState) => state.configDetails.isSaving;
export const selectConfigDetailsError = (state: RootState) => state.configDetails.error;

export default configDetailsSlice.reducer;
