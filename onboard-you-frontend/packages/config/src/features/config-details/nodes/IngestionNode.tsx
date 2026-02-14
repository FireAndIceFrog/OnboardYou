import { Handle, Position, type NodeProps } from '@xyflow/react';
import styles from './nodes.module.scss';

export function IngestionNode({ data }: NodeProps) {
  const { label, stageType, source } = data as {
    label: string;
    stageType: string;
    source?: string;
  };

  return (
    <div className={`${styles.pipelineNode} ${styles.ingestionNode}`}>
      <div className={styles.nodeHeader}>
        <span className={styles.nodeIcon}>📥</span>
        <span className={styles.nodeLabel}>{label}</span>
      </div>
      <div className={styles.nodeBody}>
        {source && <span className={styles.nodeDetail}>{source}</span>}
        <span className={styles.nodeBadge}>{stageType}</span>
      </div>
      <Handle type="source" position={Position.Right} className={styles.handle} />
    </div>
  );
}
