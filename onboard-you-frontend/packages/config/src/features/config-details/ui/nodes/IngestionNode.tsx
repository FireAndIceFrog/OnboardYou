import { Handle, Position, type NodeProps } from '@xyflow/react';
import styles from './nodes.module.scss';

export function IngestionNode({ data }: NodeProps) {
  return (
    <div className={`${styles.pipelineNode} ${styles.ingestionNode}`}>
      <div className={styles.nodeHeader}>
        <span className={styles.nodeIcon}>📥</span>
        <span className={styles.nodeTitle}>Ingestion</span>
      </div>
      <div className={styles.nodeBody}>{data.label as string}</div>
      <span className={styles.nodeBadge}>{data.stageType as string}</span>
      <Handle type="source" position={Position.Right} />
    </div>
  );
}
