import { Handle, Position, type NodeProps } from '@xyflow/react';
import styles from './nodes.module.scss';

export function TransformationNode({ data }: NodeProps) {
  const { label, stageType } = data as {
    label: string;
    stageType: string;
  };

  return (
    <div className={`${styles.pipelineNode} ${styles.transformationNode}`}>
      <div className={styles.nodeHeader}>
        <span className={styles.nodeIcon}>⚙️</span>
        <span className={styles.nodeLabel}>{label}</span>
      </div>
      <div className={styles.nodeBody}>
        <span className={styles.nodeBadge}>{stageType}</span>
      </div>
      <Handle type="target" position={Position.Left} className={styles.handle} />
      <Handle type="source" position={Position.Right} className={styles.handle} />
    </div>
  );
}
