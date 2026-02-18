import { useMemo, useCallback, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useParams, useNavigate, useLocation } from 'react-router-dom';
import {
  ReactFlow,
  MiniMap,
  Controls,
  Background,
  BackgroundVariant,
  type NodeMouseHandler,
  type NodeChange,
  type EdgeChange,
  type Node,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import { useAppDispatch, useAppSelector } from '@/store';
import { useGlobal } from '@/shared/hooks';
import { Button, Spinner, Badge } from '@/shared/ui';
import { humanFrequency } from '@/shared/domain/types';
import type { ConnectionForm } from '../domain/types';
import {
  fetchConfigDetails,
  initNewConfig,
  saveConfigThunk,
  createConfigThunk,
  deleteConfigThunk,
  validateConfigThunk,
  onNodesChange as onNodesChangeAction,
  onEdgesChange as onEdgesChangeAction,
  selectNode as selectNodeAction,
  deselectNode,
  toggleChat,
  toggleAddStepPanel,
  setAddStepPanelOpen,
  addFlowAction,
  selectConfig,
  selectNodes,
  selectEdges,
  selectSelectedNode,
  selectIsChatOpen,
  selectAddStepPanelOpen,
  selectConfigDetailsLoading,
  selectConfigDetailsSaving,
  selectConfigDetailsDeleting,
  selectConfigDetailsError,
} from '../state/configDetailsSlice';
import { selectLastFlowAction } from '@/features/chat/state/chatSlice';
import { ActionEditPanel } from './ActionEditPanel';
import { AddStepPanel } from './AddStepPanel';
import { IngestionNode, TransformationNode, EgressNode } from './nodes';
import { ChatWindow } from '@/features/chat/ui';
import styles from './ConfigDetailsPage.module.scss';

const nodeTypes = {
  ingestion: IngestionNode,
  logic: TransformationNode,
  egress: EgressNode,
};

function ConfigDetailsContent({
  customerCompanyId,
  isNewConfig,
  connectionForm,
}: {
  customerCompanyId: string;
  isNewConfig: boolean;
  connectionForm?: ConnectionForm;
}) {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { showNotification } = useGlobal();
  const navigate = useNavigate();

  const config = useAppSelector(selectConfig);
  const nodes = useAppSelector(selectNodes);
  const edges = useAppSelector(selectEdges);
  const selectedNode = useAppSelector(selectSelectedNode);
  const isLoading = useAppSelector(selectConfigDetailsLoading);
  const isSaving = useAppSelector(selectConfigDetailsSaving);
  const isDeleting = useAppSelector(selectConfigDetailsDeleting);
  const error = useAppSelector(selectConfigDetailsError);
  const chatOpen = useAppSelector(selectIsChatOpen);
  const addStepOpen = useAppSelector(selectAddStepPanelOpen);
  const lastFlowAction = useAppSelector(selectLastFlowAction);

  // ── Fetch existing config or initialise a blank one ───────
  useEffect(() => {
    if (isNewConfig && connectionForm) {
      dispatch(initNewConfig(connectionForm));
    } else {
      dispatch(fetchConfigDetails({ customerCompanyId }));
    }
  }, [dispatch, customerCompanyId, isNewConfig, connectionForm]);

  // ── Show error notifications ──────────────────────────────
  useEffect(() => {
    if (error) showNotification(error, 'error');
  }, [error, showNotification]);

  // ── Auto-validate to get per-step column snapshots ────────
  const validateTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const pipelineActions = config?.pipeline?.actions;

  useEffect(() => {
    if (!config || !pipelineActions?.length) return;
    clearTimeout(validateTimerRef.current);
    validateTimerRef.current = setTimeout(() => {
      dispatch(validateConfigThunk({ customerCompanyId, data: config }));
    }, 400);                 // debounce 400 ms so rapid edits don't spam the API
    return () => clearTimeout(validateTimerRef.current);
  }, [dispatch, customerCompanyId, config, pipelineActions]);

  // ── Real-time flow updates from chat ──────────────────────
  const processedActionsRef = useRef(new Set<string>());

  useEffect(() => {
    if (lastFlowAction && !processedActionsRef.current.has(lastFlowAction.id)) {
      processedActionsRef.current.add(lastFlowAction.id);
      dispatch(addFlowAction(lastFlowAction));
    }
  }, [lastFlowAction, dispatch]);

  const handleNodeClick: NodeMouseHandler = useCallback(
    (_event, node) => {
      dispatch(selectNodeAction(node));
    },
    [dispatch],
  );

  const handlePaneClick = useCallback(() => {
    dispatch(deselectNode());
  }, [dispatch]);

  const handleNodesChange = useCallback(
    (changes: NodeChange[]) => {
      dispatch(onNodesChangeAction(changes));
    },
    [dispatch],
  );

  const handleEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      dispatch(onEdgesChangeAction(changes));
    },
    [dispatch],
  );

  const handleToggleChat = useCallback(() => {
    dispatch(toggleChat());
  }, [dispatch]);

  const handleToggleAddStep = useCallback(() => {
    dispatch(toggleAddStepPanel());
  }, [dispatch]);

  const handleCloseAddStep = useCallback(() => {
    dispatch(setAddStepPanelOpen(false));
  }, [dispatch]);

  const handleBack = useCallback(() => {
    navigate(-1);
  }, [navigate]);

  const handleSave = useCallback(async () => {
    if (!config) return;

    try {
      if (isNewConfig) {
        // Derive a customerCompanyId from the display name
        const newId = config.name
          .toLowerCase()
          .replace(/[^a-z0-9]+/g, '-')
          .replace(/^-|-$/g, '')
          || 'new-config';

        const result = await dispatch(
          createConfigThunk({ customerCompanyId: newId, data: config }),
        ).unwrap();

        showNotification(t('configDetails.createSuccess'), 'success');
        // Navigate to the real config URL so subsequent saves use PUT
        navigate(`/config/${result.customerCompanyId}`, { replace: true });
      } else {
        await dispatch(
          saveConfigThunk({ customerCompanyId, data: config }),
        ).unwrap();

        showNotification(t('configDetails.saveSuccess'), 'success');
      }
    } catch {
      // Error is already set in Redux state by the rejected thunk
    }
  }, [config, isNewConfig, customerCompanyId, dispatch, navigate, showNotification, t]);

  const handleDelete = useCallback(async () => {
    if (isNewConfig) return;
    const confirmed = window.confirm(t('configDetails.deleteConfirm'));
    if (!confirmed) return;

    try {
      await dispatch(deleteConfigThunk({ customerCompanyId })).unwrap();
      showNotification(t('configDetails.deleteSuccess'), 'success');
      navigate('/config', { replace: true });
    } catch {
      // Error is already set in Redux state by the rejected thunk
    }
  }, [isNewConfig, customerCompanyId, dispatch, navigate, showNotification, t]);

  const memoizedNodeTypes = useMemo(() => nodeTypes, []);

  const defaultEdgeOptions = useMemo(
    () => ({
      animated: true,
      style: { strokeWidth: 2 },
    }),
    [],
  );

  if (isLoading) {
    return (
      <div className={styles.loadingState}>
        <Spinner size="lg" />
        <span>{t('configDetails.loading')}</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.errorState}>
        <span className={styles.errorIcon}>⚠️</span>
        <p>{error}</p>
        <Button variant="secondary" onClick={handleBack}>
          {t('configDetails.backToConfigurations')}
        </Button>
      </div>
    );
  }

  if (!config) return null;

  return (
    <div className={styles.detailsPage}>
      {/* Step indicator bar */}
      <nav className={styles.stepBar} aria-label="Configuration steps">
        <div className={styles.stepItem}>
          <span className={styles.stepDot} data-completed="">✓</span>
          <span className={styles.stepText}>{t('configDetails.steps.connectionDetails')}</span>
        </div>
        <div className={styles.stepLine} data-active="" />
        <div className={styles.stepItem} aria-current="step">
          <span className={styles.stepDot} data-active="">2</span>
          <span className={styles.stepTextActive}>{t('configDetails.steps.flowCustomization')}</span>
        </div>
      </nav>

      {/* Header */}
      <header className={styles.detailHeader}>
        <div className={styles.headerLeft}>
          <button type="button" className={styles.backBtn} onClick={handleBack}>
            {t('configDetails.back')}
          </button>
          <h1 className={styles.configName}>{config.name}</h1>
          <Badge variant="info">{humanFrequency(config.cron)}</Badge>
        </div>
        <div className={styles.headerRight}>
          <button type="button" className={styles.addStepBtn} onClick={handleToggleAddStep}>
            ➕ {t('configDetails.addStep')}
          </button>
          <button type="button" className={styles.chatToggle} onClick={handleToggleChat}>
            💬 {chatOpen ? t('configDetails.closeChat') : t('configDetails.openChat')}
          </button>
          <Button variant="primary" size="sm" onClick={handleSave} disabled={isSaving}>
            {isSaving ? t('configDetails.saving') : t('configDetails.save')}
          </Button>
          {!isNewConfig && (
            <Button variant="danger" size="sm" onClick={handleDelete} disabled={isDeleting}>
              {isDeleting ? t('configDetails.deleting') : t('configDetails.delete')}
            </Button>
          )}
        </div>
      </header>

      {/* Body */}
      <div className={styles.body}>
        {/* Canvas */}
        <div className={styles.canvasArea} role="application" aria-label="Pipeline flow editor">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            nodeTypes={memoizedNodeTypes}
            defaultEdgeOptions={defaultEdgeOptions}
            onNodesChange={handleNodesChange}
            onEdgesChange={handleEdgesChange}
            onNodeClick={handleNodeClick}
            onPaneClick={handlePaneClick}
            fitView
            fitViewOptions={{ padding: 0.2 }}
            proOptions={{ hideAttribution: true }}
          >
            <MiniMap
              nodeStrokeWidth={3}
              zoomable
              pannable
            />
            <Controls />
            <Background variant={BackgroundVariant.Dots} gap={16} size={1} />
          </ReactFlow>

          {selectedNode && <ActionEditPanel />}
          {addStepOpen && <AddStepPanel onClose={handleCloseAddStep} />}
        </div>

        {/* Chat Panel */}
        <aside
          className={`${styles.chatPanel} ${!chatOpen ? styles.chatPanelHidden : ''}`}
        >
          {chatOpen && (
            <ChatWindow onClose={handleToggleChat} />
          )}
        </aside>
      </div>
    </div>
  );
}

export function ConfigDetailsPage() {
  const { t } = useTranslation();
  const { customerCompanyId } = useParams<{ customerCompanyId: string }>();
  const location = useLocation();

  const isNewConfig = customerCompanyId === 'new';
  const connectionForm = (location.state as { connection?: ConnectionForm } | null)
    ?.connection;

  // If navigating to /new without connection form data, redirect back
  if (isNewConfig && !connectionForm) {
    return (
      <div className={styles.errorState}>
        <span className={styles.errorIcon}>⚠️</span>
        <p>{t('configDetails.noConnectionData')}</p>
      </div>
    );
  }

  if (!customerCompanyId) {
    return (
      <div className={styles.errorState}>
        <span className={styles.errorIcon}>⚠️</span>
        <p>{t('configDetails.noConfigId')}</p>
      </div>
    );
  }

  return (
    <ConfigDetailsContent
      customerCompanyId={customerCompanyId}
      isNewConfig={isNewConfig}
      connectionForm={connectionForm}
    />
  );
}
