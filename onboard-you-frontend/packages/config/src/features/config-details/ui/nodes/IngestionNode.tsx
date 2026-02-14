import { Handle, Position, type NodeProps } from '@xyflow/react';
import { businessLabel } from '@/shared/domain/types';
import styles from './nodes.module.scss';

export function IngestionNode({ data }: NodeProps) {
  const friendly = businessLabel(data.actionType as string);

  return (
    <div className={`${styles.pipelineNode} ${styles.ingestionNode}`}>
      <div className={styles.nodeHeader}>
        <span className={styles.nodeIcon}>📥</span>
        <span className={styles.nodeTitle}>Data Source</span>
      </div>
      <div className={styles.nodeBody}>{data.label as string}</div>
      <span className={styles.nodeBadge}>{friendly}</span>
      <Handle type="source" position={Position.Right} />
    </div>
  );
}
