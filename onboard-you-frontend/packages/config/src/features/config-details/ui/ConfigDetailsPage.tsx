import { useMemo, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  ReactFlow,
  MiniMap,
  Controls,
  Background,
  BackgroundVariant,
  type NodeMouseHandler,
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
  transformation: TransformationNode,
  egress: EgressNode,
};

function statusToBadgeVariant(status: string): 'active' | 'draft' | 'paused' | 'error' | 'info' {
  switch (status) {
    case 'active':
      return 'active';
    case 'draft':
      return 'draft';
    case 'paused':
      return 'paused';
    case 'error':
      return 'error';
    default:
      return 'info';
  }
}

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
          <Badge variant={statusToBadgeVariant(config.status)}>{config.status}</Badge>
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
  const { configId } = useParams<{ configId: string }>();

  if (!configId) {
    return (
      <div className={styles.errorState}>
        <span className={styles.errorIcon}>⚠️</span>
        <p>No configuration ID provided.</p>
      </div>
    );
  }

  return (
    <ConfigDetailsProvider configId={configId}>
      <ConfigDetailsContent />
    </ConfigDetailsProvider>
  );
}
