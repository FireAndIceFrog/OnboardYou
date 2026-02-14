import { Handle, Position, type NodeProps } from '@xyflow/react';
import { businessLabel } from '@/shared/domain/types';
import styles from './nodes.module.scss';

export function TransformationNode({ data }: NodeProps) {
  const friendly = businessLabel(data.actionType as string);

  return (
    <div className={`${styles.pipelineNode} ${styles.logicNode}`}>
      <div className={styles.nodeHeader}>
        <span className={styles.nodeIcon}>⚙️</span>
        <span className={styles.nodeTitle}>Business Rule</span>
      </div>
      <div className={styles.nodeBody}>{data.label as string}</div>
      <span className={styles.nodeBadge}>{friendly}</span>
      <Handle type="target" position={Position.Left} />
      <Handle type="source" position={Position.Right} />
    </div>
  );
}
