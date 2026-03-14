import { useMemo, useCallback, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  ReactFlow,
  Controls,
  Background,
  BackgroundVariant,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { Box, Flex, Text, Button } from '@chakra-ui/react';
import { ArrowLeftIcon } from '@/shared/ui';

import { useAppDispatch } from '@/store';
import type { PipelineRun } from '@/generated/api';
import { convertToFlow } from '@/features/config-details/services/pipelineLayoutService';
import { PipelineNode, AddButtonEdge } from '@/features/config-details/ui/components';
import { clearSelectedRun } from '../../state/runHistorySlice';
import { RunHistoryTab } from './RunHistoryTab';
import { RunDetailsPanel } from './RunDetailsPanel';

/* ── React Flow registrations ──────────────────────────────── */

const nodeTypes = {
  ingestion: PipelineNode,
  logic: PipelineNode,
  egress: PipelineNode,
};

const edgeTypes = {
  addButton: AddButtonEdge,
};

const defaultEdgeOptions = {
  animated: true,
  style: { strokeWidth: 2 },
};

/* ── Component ─────────────────────────────────────────────── */

export function RunHistoryView({ customerCompanyId }: { customerCompanyId: string }) {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();

  const [viewingRun, setViewingRun] = useState<PipelineRun | null>(null);

  // Build React Flow nodes/edges from a run's manifest snapshot,
  // highlighting the error action in red.
  const historyFlow = useMemo(() => {
    if (!viewingRun?.manifestSnapshot) return null;
    const { nodes: flowNodes, edges: flowEdges } = convertToFlow(viewingRun.manifestSnapshot);
    if (viewingRun.errorActionId) {
      for (const n of flowNodes) {
        if (n.id === viewingRun.errorActionId) {
          n.style = { ...n.style, border: '2px solid #E53E3E', boxShadow: '0 0 8px rgba(229,62,62,0.4)' };
          n.data = { ...n.data, hasError: true };
        }
      }
    }
    return { nodes: flowNodes, edges: flowEdges };
  }, [viewingRun]);

  const handleSelectRun = useCallback((run: PipelineRun) => {
    setViewingRun(run);
  }, []);

  const handleCloseRunDetails = useCallback(() => {
    setViewingRun(null);
    dispatch(clearSelectedRun());
  }, [dispatch]);

  const memoizedNodeTypes = useMemo(() => nodeTypes, []);
  const memoizedEdgeTypes = useMemo(() => edgeTypes, []);
  const memoizedEdgeOptions = useMemo(() => defaultEdgeOptions, []);

  if (viewingRun) {
    return (
      <Box flex="1" position="relative" role="application" aria-label="Pipeline run snapshot" data-testid="run-history-view">
        {/* Toolbar */}
        <Flex
          position="absolute"
          top="0"
          left="0"
          right="0"
          zIndex={5}
          px="4"
          py="2"
          bg="whiteAlpha.900"
          borderBottom="1px solid"
          borderColor="gray.200"
        >
          <Button variant="ghost" size="sm" onClick={handleCloseRunDetails} data-testid="back-to-runs">
            <ArrowLeftIcon size="0.85em" /> {t('runHistory.backToList', 'Back to runs')}
          </Button>
        </Flex>

        {historyFlow ? (
          <ReactFlow
            nodes={historyFlow.nodes}
            edges={historyFlow.edges}
            nodeTypes={memoizedNodeTypes}
            edgeTypes={memoizedEdgeTypes}
            defaultEdgeOptions={memoizedEdgeOptions}
            nodesDraggable={false}
            nodesConnectable={false}
            elementsSelectable={false}
            fitView
            fitViewOptions={{ padding: 0.2 }}
            proOptions={{ hideAttribution: true }}
          >
            <Controls />
            <Background variant={BackgroundVariant.Dots} gap={16} size={1} />
          </ReactFlow>
        ) : (
          <Flex align="center" justify="center" h="100%" color="gray.500">
            <Text fontSize="sm">{t('runHistory.noSnapshot', 'No manifest snapshot available for this run')}</Text>
          </Flex>
        )}

        <RunDetailsPanel run={viewingRun} onClose={handleCloseRunDetails} />
      </Box>
    );
  }

  return (
    <Box flex="1" overflow="auto" bg="white" data-testid="run-history-view">
      <RunHistoryTab
        customerCompanyId={customerCompanyId}
        onSelectRun={handleSelectRun}
      />
    </Box>
  );
}
