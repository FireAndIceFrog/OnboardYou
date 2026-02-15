import { useMemo, useCallback, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useParams, useNavigate } from 'react-router-dom';
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
import {
  fetchConfigDetails,
  onNodesChange as onNodesChangeAction,
  onEdgesChange as onEdgesChangeAction,
  selectNode as selectNodeAction,
  deselectNode,
  toggleChat,
  addFlowAction,
  selectConfig,
  selectNodes,
  selectEdges,
  selectSelectedNode,
  selectIsChatOpen,
  selectConfigDetailsLoading,
  selectConfigDetailsError,
} from '../state/configDetailsSlice';
import { selectLastFlowAction } from '@/features/chat/state/chatSlice';
import { ConfigDetailsForm } from './ConfigDetailsForm';
import { IngestionNode, TransformationNode, EgressNode } from './nodes';
import { ChatWindow } from '@/features/chat/ui';
import styles from './ConfigDetailsPage.module.scss';

const nodeTypes = {
  ingestion: IngestionNode,
  logic: TransformationNode,
  egress: EgressNode,
};

function ConfigDetailsContent({ customerCompanyId }: { customerCompanyId: string }) {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { showNotification } = useGlobal();
  const navigate = useNavigate();

  const config = useAppSelector(selectConfig);
  const nodes = useAppSelector(selectNodes);
  const edges = useAppSelector(selectEdges);
  const selectedNode = useAppSelector(selectSelectedNode);
  const isLoading = useAppSelector(selectConfigDetailsLoading);
  const error = useAppSelector(selectConfigDetailsError);
  const chatOpen = useAppSelector(selectIsChatOpen);
  const lastFlowAction = useAppSelector(selectLastFlowAction);

  // ── Fetch config on mount ─────────────────────────────────
  useEffect(() => {
    dispatch(fetchConfigDetails({ customerCompanyId }));
  }, [dispatch, customerCompanyId]);

  // ── Show error notifications ──────────────────────────────
  useEffect(() => {
    if (error) showNotification(error, 'error');
  }, [error, showNotification]);

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

  const handleBack = useCallback(() => {
    navigate(-1);
  }, [navigate]);

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
          <button type="button" className={styles.chatToggle} onClick={handleToggleChat}>
            💬 {chatOpen ? t('configDetails.closeChat') : t('configDetails.openChat')}
          </button>
          <Button variant="primary" size="sm">
            {t('configDetails.save')}
          </Button>
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

          {selectedNode && <ConfigDetailsForm />}
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

  if (!customerCompanyId) {
    return (
      <div className={styles.errorState}>
        <span className={styles.errorIcon}>⚠️</span>
        <p>{t('configDetails.noConfigId')}</p>
      </div>
    );
  }

  return (
    <ConfigDetailsContent customerCompanyId={customerCompanyId} />
  );
}
