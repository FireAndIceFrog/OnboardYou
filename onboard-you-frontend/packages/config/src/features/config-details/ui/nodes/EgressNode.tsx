import { Handle, Position, type NodeProps } from '@xyflow/react';
import styles from './nodes.module.scss';

export function EgressNode({ data }: NodeProps) {
  return (
    <div className={`${styles.pipelineNode} ${styles.egressNode}`}>
      <div className={styles.nodeHeader}>
        <span className={styles.nodeIcon}>📤</span>
        <span className={styles.nodeTitle}>Egress</span>
      </div>
      <div className={styles.nodeBody}>{data.label as string}</div>
      <span className={styles.nodeBadge}>{data.stageType as string}</span>
      <Handle type="target" position={Position.Left} />
    </div>
  );
}
