import { useCallback, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  ReactFlow,
  MiniMap,
  Controls,
  Background,
  BackgroundVariant,
  type NodeMouseHandler,
  type Node,
} from '@xyflow/react';
import { useConfigDetails } from './useConfigDetails';
import { IngestionNode } from './nodes/IngestionNode';
import { TransformationNode } from './nodes/TransformationNode';
import { EgressNode } from './nodes/EgressNode';
import { ConfigDetailsForm } from './ConfigDetailsForm';
import { ChatWindow } from '@/features/chat/ChatWindow';
import styles from './ConfigDetailsPage.module.scss';

const nodeTypes = {
  ingestion: IngestionNode,
  transformation: TransformationNode,
  egress: EgressNode,
};

export function ConfigDetailsPage() {
  const navigate = useNavigate();
  const {
    config,
    nodes,
    edges,
    onNodesChange,
    onEdgesChange,
    isLoading,
    error,
    selectedNode,
    setSelectedNode,
  } = useConfigDetails();

  const [chatOpen, setChatOpen] = useState(true);

  const onNodeClick: NodeMouseHandler = useCallback(
    (_event, node: Node) => {
      setSelectedNode(node);
    },
    [setSelectedNode],
  );

  const onPaneClick = useCallback(() => {
    setSelectedNode(null);
  }, [setSelectedNode]);

  if (isLoading) {
    return (
      <div className={styles.loadingState}>
        <div className={styles.spinner} />
        <p>Loading configuration…</p>
      </div>
    );
  }

  if (error || !config) {
    return (
      <div className={styles.errorState}>
        <span className={styles.errorIcon}>⚠️</span>
        <h2>Failed to load configuration</h2>
        <p>{error ?? 'Configuration not found'}</p>
        <button className={styles.backBtn} onClick={() => navigate('/')}>
          ← Back to Configurations
        </button>
      </div>
    );
  }

  const statusClass =
    config.status === 'active'
      ? styles.statusActive
      : config.status === 'paused'
        ? styles.statusPaused
        : config.status === 'error'
          ? styles.statusError
          : styles.statusDraft;

  return (
    <div className={styles.page}>
      <header className={styles.detailHeader}>
        <div className={styles.headerLeft}>
          <button className={styles.backBtn} onClick={() => navigate('/')}>
            ← Back
          </button>
          <h1 className={styles.configName}>{config.name}</h1>
          <span className={`${styles.statusBadge} ${statusClass}`}>{config.status}</span>
        </div>
        <div className={styles.headerRight}>
          <button
            className={styles.chatToggle}
            onClick={() => setChatOpen((v) => !v)}
            aria-label={chatOpen ? 'Hide chat' : 'Show chat'}
          >
            {chatOpen ? '💬 Hide Chat' : '💬 Show Chat'}
          </button>
          <button className={styles.saveBtn}>Save</button>
        </div>
      </header>

      <div className={styles.body}>
        <div className={styles.canvasArea}>
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onNodeClick={onNodeClick}
            onPaneClick={onPaneClick}
            nodeTypes={nodeTypes}
            fitView
            fitViewOptions={{ padding: 0.3 }}
            proOptions={{ hideAttribution: true }}
          >
            <Background variant={BackgroundVariant.Dots} gap={20} size={1} color="#CBD5E1" />
            <MiniMap
              nodeStrokeWidth={2}
              pannable
              zoomable
              style={{ border: '1px solid #E2E8F0', borderRadius: 8 }}
            />
            <Controls showInteractive={false} />
          </ReactFlow>
        </div>

        {chatOpen && (
          <div className={styles.chatPanel}>
            <ChatWindow config={config} onClose={() => setChatOpen(false)} />
          </div>
        )}
      </div>

      {selectedNode && (
        <ConfigDetailsForm node={selectedNode} onClose={() => setSelectedNode(null)} />
      )}
    </div>
  );
}
