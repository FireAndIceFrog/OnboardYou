import { useMemo, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  ReactFlow,
  MiniMap,
  Controls,
  Background,
  BackgroundVariant,
  applyNodeChanges,
  applyEdgeChanges,
  type NodeMouseHandler,
  type NodeChange,
  type EdgeChange,
  type Node,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import { Button, Spinner, Badge } from '@/shared/ui';
import { useConfigDetails } from '../state/ConfigDetailsContext';
import { ConfigDetailsProvider } from '../state/ConfigDetailsProvider';
import { ConfigDetailsForm } from './ConfigDetailsForm';
import { IngestionNode, TransformationNode, EgressNode } from './nodes';
import styles from './ConfigDetailsPage.module.scss';

const nodeTypes = {
  ingestion: IngestionNode,
  logic: TransformationNode,
  egress: EgressNode,
};

function ConfigDetailsContent() {
  const { state, dispatch } = useConfigDetails();
  const navigate = useNavigate();

  const { config, nodes, edges, selectedNode, isLoading, error, chatOpen } = state;

  const handleNodeClick: NodeMouseHandler = useCallback(
    (_event, node) => {
      dispatch({ type: 'SELECT_NODE', payload: node });
    },
    [dispatch],
  );

  const handlePaneClick = useCallback(() => {
    dispatch({ type: 'DESELECT_NODE' });
  }, [dispatch]);

  const handleNodesChange = useCallback(
    (changes: NodeChange[]) => {
      dispatch({ type: 'SET_NODES', payload: applyNodeChanges(changes, nodes) });
    },
    [dispatch, nodes],
  );

  const handleEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      dispatch({ type: 'SET_EDGES', payload: applyEdgeChanges(changes, edges) });
    },
    [dispatch, edges],
  );

  const handleToggleChat = useCallback(() => {
    dispatch({ type: 'TOGGLE_CHAT' });
  }, [dispatch]);

  const handleBack = useCallback(() => {
    navigate(-1);
  }, [navigate]);

  const memoizedNodeTypes = useMemo(() => nodeTypes, []);

  if (isLoading) {
    return (
      <div className={styles.loadingState}>
        <Spinner size="lg" />
        <span>Loading configuration…</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.errorState}>
        <span className={styles.errorIcon}>⚠️</span>
        <p>{error}</p>
        <Button variant="secondary" onClick={handleBack}>
          ← Back to Configurations
        </Button>
      </div>
    );
  }

  if (!config) return null;

  return (
    <div className={styles.detailsPage}>
      {/* Header */}
      <header className={styles.detailHeader}>
        <div className={styles.headerLeft}>
          <button type="button" className={styles.backBtn} onClick={handleBack}>
            ← Back
          </button>
          <h1 className={styles.configName}>{config.name}</h1>
          <Badge variant="info">{config.cron}</Badge>
          <Badge variant="active">{config.pipeline.actions.length} steps</Badge>
        </div>
        <div className={styles.headerRight}>
          <button type="button" className={styles.chatToggle} onClick={handleToggleChat}>
            💬 {chatOpen ? 'Close Chat' : 'Open Chat'}
          </button>
          <Button variant="primary" size="sm">
            Save
          </Button>
        </div>
      </header>

      {/* Body */}
      <div className={styles.body}>
        {/* Canvas */}
        <div className={styles.canvasArea}>
          <ReactFlow
            nodes={nodes}
            edges={edges}
            nodeTypes={memoizedNodeTypes}
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
        <div
          className={`${styles.chatPanel} ${!chatOpen ? styles.chatPanelHidden : ''}`}
        >
          {chatOpen && (
            <div style={{ height: '100%', display: 'flex', flexDirection: 'column', padding: '1rem' }}>
              <p style={{ color: '#64748B', textAlign: 'center', marginTop: '2rem' }}>
                Chat assistant coming soon…
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export function ConfigDetailsPage() {
  const { customerCompanyId } = useParams<{ customerCompanyId: string }>();

  if (!customerCompanyId) {
    return (
      <div className={styles.errorState}>
        <span className={styles.errorIcon}>⚠️</span>
        <p>No configuration ID provided.</p>
      </div>
    );
  }

  return (
    <ConfigDetailsProvider customerCompanyId={customerCompanyId}>
      <ConfigDetailsContent />
    </ConfigDetailsProvider>
  );
}
