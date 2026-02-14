import { Handle, Position, type NodeProps } from '@xyflow/react';
import styles from './nodes.module.scss';

export function EgressNode({ data }: NodeProps) {
  const { label, stageType, destination } = data as {
    label: string;
    stageType: string;
    destination?: string;
  };

  return (
    <div className={`${styles.pipelineNode} ${styles.egressNode}`}>
      <div className={styles.nodeHeader}>
        <span className={styles.nodeIcon}>📤</span>
        <span className={styles.nodeLabel}>{label}</span>
      </div>
      <div className={styles.nodeBody}>
        {destination && <span className={styles.nodeDetail}>{destination}</span>}
        <span className={styles.nodeBadge}>{stageType}</span>
      </div>
      <Handle type="target" position={Position.Left} className={styles.handle} />
    </div>
  );
}
