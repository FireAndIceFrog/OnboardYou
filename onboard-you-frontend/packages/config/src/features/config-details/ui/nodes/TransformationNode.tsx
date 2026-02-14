import { Handle, Position, type NodeProps } from '@xyflow/react';
import styles from './nodes.module.scss';

export function TransformationNode({ data }: NodeProps) {
  return (
    <div className={`${styles.pipelineNode} ${styles.transformationNode}`}>
      <div className={styles.nodeHeader}>
        <span className={styles.nodeIcon}>⚙️</span>
        <span className={styles.nodeTitle}>Transform</span>
      </div>
      <div className={styles.nodeBody}>{data.label as string}</div>
      <span className={styles.nodeBadge}>{data.stageType as string}</span>
      <Handle type="target" position={Position.Left} />
      <Handle type="source" position={Position.Right} />
    </div>
  );
}
