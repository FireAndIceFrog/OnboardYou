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
import type { PipelineConfig, ActionConfig, ActionConfigPayload, ValidationResult, ActionType, WorkdayConfig, CsvHrisConnectorConfig } from '@/generated/api';
import { actionCategory, businessLabel } from '@/shared/domain/types';
import type { ConfigDetailsState, ConnectionForm } from '../domain/types';
import { buildResponseGroup } from '../domain/types';
import { fetchConfig, createConfig as createConfigService, saveConfig as saveConfigService, deleteConfig as deleteConfigService, validateConfig as validateConfigService } from '../services/configDetailsService';
import { triggerPlanGeneration, pollForPlanCompletion } from '../services/planService';
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
  isDeleting: false,
  isValidating: false,
  error: null,
  chatOpen: false,
  addStepPanelOpen: false,
  validationResult: null,
  planSummary: null,
  isGeneratingPlan: false,
  viewMode: 'normal',
  planStale: false,
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

export const deleteConfigThunk = createAsyncThunk<
  void,
  { customerCompanyId: string },
  { extra: ThunkExtra }
>(
  'configDetails/deleteConfig',
  async ({ customerCompanyId }, { rejectWithValue }) => {
    try {
      await deleteConfigService(customerCompanyId);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to delete configuration';
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

/**
 * Trigger async plan generation: POST generate-plan, then poll
 * GET /config/{id} until generationStatus is completed or failed.
 */
export const generatePlanThunk = createAsyncThunk<
  PipelineConfig,
  { customerCompanyId: string; sourceSystem: string },
  { extra: ThunkExtra }
>(
  'configDetails/generatePlan',
  async ({ customerCompanyId, sourceSystem }, { rejectWithValue }) => {
    try {
      await triggerPlanGeneration(customerCompanyId, sourceSystem);
      const config = await pollForPlanCompletion(customerCompanyId);
      return config;
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to generate plan';
      return rejectWithValue(message);
    }
  },
);

/**
 * Save the manifest (with current disabled flags) as the pipeline config.
 * Used by "Looks Good, Start Syncing" in Normal mode.
 */
export const applyPlanThunk = createAsyncThunk<
  PipelineConfig,
  { customerCompanyId: string },
  { state: RootState; extra: ThunkExtra }
>(
  'configDetails/applyPlan',
  async ({ customerCompanyId }, { getState, rejectWithValue }) => {
    try {
      const config = getState().configDetails.config;
      if (!config) throw new Error('No config to save');
      return await saveConfigService(customerCompanyId, {
        name: config.name,
        cron: config.cron,
        pipeline: config.pipeline,
        image: config.image,
      });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to apply plan';
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

      // Keep pipeline config in sync so the action is included in save/validate
      if (state.config) {
        state.config.pipeline.actions.push(actionCfg);
      }

      // Plan is now out of sync
      if (state.planSummary) state.planStale = true;

      const newNode: Node = {
        id: actionCfg.id,
        type: category,
        position: { x: START_X + idx * NODE_GAP_X, y: START_Y },
        data: {
          actionId: actionCfg.id,
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
    toggleAddStepPanel(state) {
      state.addStepPanelOpen = !state.addStepPanelOpen;
    },
    setAddStepPanelOpen(state, action: PayloadAction<boolean>) {
      state.addStepPanelOpen = action.payload;
    },
    removeFlowAction(state, action: PayloadAction<string>) {
      const actionId = action.payload;

      // Remove from pipeline config
      if (state.config) {
        state.config.pipeline.actions = state.config.pipeline.actions.filter(
          (a) => a.id !== actionId,
        );
      }

      // Remove the node
      state.nodes = state.nodes.filter((n) => n.id !== actionId) as typeof state.nodes;

      // Remove connected edges
      state.edges = state.edges.filter(
        (e) => e.source !== actionId && e.target !== actionId,
      ) as typeof state.edges;

      // Re-link the chain: if node B was between A and C, connect A→C
      const plainNodes = current(state).nodes;
      const newEdges: typeof state.edges = [];
      for (let i = 1; i < plainNodes.length; i++) {
        const prev = plainNodes[i - 1];
        const curr = plainNodes[i];
        const cat = (curr.data as Record<string, unknown>).category as string;
        const edgeColor = CATEGORY_STYLES[cat] ?? CATEGORY_STYLES.logic;
        newEdges.push({
          id: `edge-${prev.id}-${curr.id}`,
          source: prev.id,
          target: curr.id,
          type: 'default',
          animated: true,
          style: { stroke: edgeColor.stroke, strokeWidth: 2 },
        } as typeof state.edges[number]);
      }
      state.edges = newEdges;

      // Deselect if the removed node was selected
      if (state.selectedNode?.id === actionId) {
        state.selectedNode = null;
      }

      // Plan is now out of sync
      if (state.planSummary) state.planStale = true;
    },
    updateFlowActionConfig(
      state,
      action: PayloadAction<{ actionId: string; config: ActionConfigPayload }>,
    ) {
      const { actionId, config: newConfig } = action.payload;

      // Update the pipeline config
      if (state.config) {
        const pipelineAction = state.config.pipeline.actions.find((a) => a.id === actionId);
        if (pipelineAction) {
          pipelineAction.config = newConfig;
        }
      }

      // Update the node's data.config
      const node = state.nodes.find((n) => n.id === actionId);
      if (node) {
        (node.data as Record<string, unknown>).config = newConfig;
      }

      // Update selectedNode if it matches
      if (state.selectedNode?.id === actionId) {
        (state.selectedNode.data as Record<string, unknown>).config = newConfig;
      }

      // Plan is now out of sync
      if (state.planSummary) state.planStale = true;
    },
    setViewMode(state, action: PayloadAction<'normal' | 'advanced'>) {
      state.viewMode = action.payload;
    },
    /**
     * Toggle a plan feature on/off. Looks up the feature's action_ids
     * and flips `disabled` on those manifest actions. No actions added/removed.
     */
    toggleFeature(state, action: PayloadAction<string>) {
      const featureId = action.payload;
      const feature = state.planSummary?.features.find((f) => f.id === featureId);
      if (!feature || !state.config) return;

      // Determine current enabled state by checking if ALL linked actions are NOT disabled
      const linkedActions = state.config.pipeline.actions.filter((a) =>
        feature.actionIds.includes(a.id),
      );
      const isCurrentlyEnabled = linkedActions.every((a) => !a.disabled);

      // Toggle: if currently enabled → disable, if disabled → enable
      const newDisabled = isCurrentlyEnabled;

      for (const a of state.config.pipeline.actions) {
        if (feature.actionIds.includes(a.id)) {
          a.disabled = newDisabled;
        }
      }
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
        // Populate plan summary & default to Normal mode if plan exists
        state.planSummary = action.payload.config.planSummary ?? null;
        if (action.payload.config.planSummary) {
          state.viewMode = 'normal';
        }
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
      .addCase(deleteConfigThunk.pending, (state) => {
        state.isDeleting = true;
        state.error = null;
      })
      .addCase(deleteConfigThunk.fulfilled, () => {
        return initialState;
      })
      .addCase(deleteConfigThunk.rejected, (state, action) => {
        state.isDeleting = false;
        state.error = (action.payload as string) ?? 'Failed to delete configuration';
      })
      .addCase(validateConfigThunk.pending, (state) => {
        state.isValidating = true;
      })
      .addCase(validateConfigThunk.fulfilled, (state, action) => {
        state.isValidating = false;
        state.validationResult = action.payload;
      })
      .addCase(validateConfigThunk.rejected, (state) => {
        state.isValidating = false;
        // Don't set state.error — validation failure is non-blocking.
        // The user can still retry via the Validate button.
      })
      .addCase(generatePlanThunk.pending, (state) => {
        state.isGeneratingPlan = true;
        state.error = null;
      })
      .addCase(generatePlanThunk.fulfilled, (state, action) => {
        state.isGeneratingPlan = false;
        state.config = action.payload;
        state.planSummary = action.payload.planSummary ?? null;
        state.planStale = false;
        // Switch to Normal mode now that the plan is ready
        if (action.payload.planSummary) {
          state.viewMode = 'normal';
        }
        // Re-sync flow nodes/edges from the new manifest
        const { nodes, edges } = convertToFlow(action.payload.pipeline);
        state.nodes = nodes as typeof state.nodes;
        state.edges = edges as typeof state.edges;
      })
      .addCase(generatePlanThunk.rejected, (state, action) => {
        state.isGeneratingPlan = false;
        state.error = (action.payload as string) ?? 'Failed to generate plan';
      })
      .addCase(applyPlanThunk.pending, (state) => {
        state.isSaving = true;
        state.error = null;
      })
      .addCase(applyPlanThunk.fulfilled, (state, action) => {
        state.isSaving = false;
        state.config = action.payload;
      })
      .addCase(applyPlanThunk.rejected, (state, action) => {
        state.isSaving = false;
        state.error = (action.payload as string) ?? 'Failed to apply plan';
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
  removeFlowAction,
  updateFlowActionConfig,
  toggleAddStepPanel,
  setAddStepPanelOpen,
  resetConfigDetails,
  initNewConfig,
  setViewMode,
  toggleFeature,
} = configDetailsSlice.actions;

/* ── Selectors ─────────────────────────────────────────────── */

export const selectConfigDetails = (state: RootState) => state.configDetails;
export const selectConfig = (state: RootState) => state.configDetails.config;
export const selectNodes = (state: RootState) => state.configDetails.nodes;
export const selectEdges = (state: RootState) => state.configDetails.edges;
export const selectSelectedNode = (state: RootState) => state.configDetails.selectedNode;
export const selectIsChatOpen = (state: RootState) => state.configDetails.chatOpen;
export const selectAddStepPanelOpen = (state: RootState) => state.configDetails.addStepPanelOpen;
export const selectConfigDetailsLoading = (state: RootState) => state.configDetails.isLoading;
export const selectConfigDetailsSaving = (state: RootState) => state.configDetails.isSaving;
export const selectConfigDetailsDeleting = (state: RootState) => state.configDetails.isDeleting;
export const selectConfigDetailsError = (state: RootState) => state.configDetails.error;
export const selectValidationResult = (state: RootState) => state.configDetails.validationResult;
export const selectIsValidating = (state: RootState) => state.configDetails.isValidating;
export const selectPlanSummary = (state: RootState) => state.configDetails.planSummary;
export const selectIsGeneratingPlan = (state: RootState) => state.configDetails.isGeneratingPlan;
export const selectViewMode = (state: RootState) => state.configDetails.viewMode;
export const selectPlanStale = (state: RootState) => state.configDetails.planStale;

/**
 * Returns the columns available as input to a given action.
 * That is the `columns_after` of the *preceding* step in the pipeline.
 * For the very first step (ingestion), returns an empty array (columns are sourced by the connector itself).
 */
export const selectAvailableColumnsForAction = (state: RootState, actionId: string): string[] => {
  const vr = state.configDetails.validationResult;
  if (!vr?.steps?.length) return [];

  const idx = vr.steps.findIndex((s) => s.action_id === actionId);

  // First step (ingestion) — columns are sourced by the connector itself
  if (idx === 0) return [];

  // Found in validation result — return preceding step's output columns
  if (idx > 0) return vr.steps[idx - 1].columns_after;

  // Not found (e.g. newly added step not yet re-validated) —
  // fall back to final_columns (output of the last validated step)
  return vr.final_columns ?? [];
};

export default configDetailsSlice.reducer;
