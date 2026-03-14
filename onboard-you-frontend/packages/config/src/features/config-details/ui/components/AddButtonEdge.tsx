import { useState, useCallback } from 'react';
import {
  BaseEdge,
  EdgeLabelRenderer,
  getSmoothStepPath,
  type EdgeProps,
} from '@xyflow/react';
import { useAppDispatch, useAppSelector } from '@/store';
import { selectNodes, openAddStepAtIndex } from '../../state/configDetailsSlice';

export function AddButtonEdge({
  id,
  source,
  target,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  style,
  markerEnd,
}: EdgeProps) {
  const [hovered, setHovered] = useState(false);
  const dispatch = useAppDispatch();
  const nodes = useAppSelector(selectNodes);

  const [edgePath, labelX, labelY] = getSmoothStepPath({
    sourceX,
    sourceY,
    targetX,
    targetY,
    sourcePosition,
    targetPosition,
  });

  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      const targetIdx = nodes.findIndex((n) => n.id === target);
      if (targetIdx > 0) {
        dispatch(openAddStepAtIndex(targetIdx));
      }
    },
    [target, nodes, dispatch],
  );

  return (
    <>
      {/* Invisible wide hit area for hover detection */}
      <path
        d={edgePath}
        fill="none"
        stroke="transparent"
        strokeWidth={30}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
      />
      <BaseEdge path={edgePath} markerEnd={markerEnd} style={style} />
      <EdgeLabelRenderer>
        <button
          type="button"
          onClick={handleClick}
          onMouseEnter={() => setHovered(true)}
          onMouseLeave={() => setHovered(false)}
          style={{
            position: 'absolute',
            transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
            pointerEvents: 'all',
            opacity: hovered ? 1 : 0,
            transition: 'opacity 0.15s, transform 0.15s',
            width: 24,
            height: 24,
            borderRadius: '50%',
            background: '#1a365d',
            color: '#fff',
            border: 'none',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 16,
            fontWeight: 700,
            lineHeight: 1,
            cursor: 'pointer',
            boxShadow: '0 1px 4px rgba(0,0,0,0.2)',
          }}
          aria-label="Add step"
        >
          +
        </button>
      </EdgeLabelRenderer>
    </>
  );
}
