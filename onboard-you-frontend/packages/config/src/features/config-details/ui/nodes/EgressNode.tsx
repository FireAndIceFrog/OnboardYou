import { Handle, Position, type NodeProps } from '@xyflow/react';
import { businessLabel } from '@/shared/domain/types';
import styles from './nodes.module.scss';

export function EgressNode({ data }: NodeProps) {
  const friendly = businessLabel(data.actionType as string);

  return (
    <div className={`${styles.pipelineNode} ${styles.egressNode}`}>
      <div className={styles.nodeHeader}>
        <span className={styles.nodeIcon}>📤</span>
        <span className={styles.nodeTitle}>Destination</span>
      </div>
      <div className={styles.nodeBody}>{data.label as string}</div>
      <span className={styles.nodeBadge}>{friendly}</span>
      <Handle type="target" position={Position.Left} />
    </div>
  );
}
