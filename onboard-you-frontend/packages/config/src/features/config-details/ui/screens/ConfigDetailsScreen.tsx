import { useMemo, useCallback, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useParams, useNavigate, useLocation } from 'react-router-dom';
import {
  ReactFlow,
  MiniMap,
  Controls,
  Background,
  BackgroundVariant,
  type NodeMouseHandler,
  type NodeChange,
  type EdgeChange,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { Box, Flex, Heading, Text, Button, Spinner, Badge } from '@chakra-ui/react';

import { useAppDispatch, useAppSelector } from '@/store';
import { useGlobal } from '@/shared/hooks';
import { humanFrequency } from '@/shared/domain/types';
import type { ConnectionForm } from '../../domain/types';
import {
  fetchConfigDetails,
  initNewConfig,
  saveConfigThunk,
  createConfigThunk,
  deleteConfigThunk,
  validateConfigThunk,
  onNodesChange as onNodesChangeAction,
  onEdgesChange as onEdgesChangeAction,
  selectNode as selectNodeAction,
  deselectNode,
  toggleChat,
  toggleAddStepPanel,
  setAddStepPanelOpen,
  addFlowAction,
  selectConfig,
  selectNodes,
  selectEdges,
  selectSelectedNode,
  selectIsChatOpen,
  selectAddStepPanelOpen,
  selectConfigDetailsLoading,
  selectConfigDetailsSaving,
  selectConfigDetailsDeleting,
  selectConfigDetailsError,
} from '../../state/configDetailsSlice';
import { selectLastFlowAction } from '@/features/chat/state/chatSlice';
import { ActionEditPanel, AddStepPanel, PipelineNode } from '../components';
import { ChatWindow } from '@/features/chat/ui';

const nodeTypes = {
  ingestion: PipelineNode,
  logic: PipelineNode,
  egress: PipelineNode,
};

function ConfigDetailsContent({
  customerCompanyId,
  isNewConfig,
  connectionForm,
}: {
  customerCompanyId: string;
  isNewConfig: boolean;
  connectionForm?: ConnectionForm;
}) {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const { showNotification } = useGlobal();
  const navigate = useNavigate();

  const config = useAppSelector(selectConfig);
  const nodes = useAppSelector(selectNodes);
  const edges = useAppSelector(selectEdges);
  const selectedNode = useAppSelector(selectSelectedNode);
  const isLoading = useAppSelector(selectConfigDetailsLoading);
  const isSaving = useAppSelector(selectConfigDetailsSaving);
  const isDeleting = useAppSelector(selectConfigDetailsDeleting);
  const error = useAppSelector(selectConfigDetailsError);
  const chatOpen = useAppSelector(selectIsChatOpen);
  const addStepOpen = useAppSelector(selectAddStepPanelOpen);
  const lastFlowAction = useAppSelector(selectLastFlowAction);

  // ── Fetch existing config or initialise a blank one ───────
  useEffect(() => {
    if (isNewConfig && connectionForm) {
      dispatch(initNewConfig(connectionForm));
    } else {
      dispatch(fetchConfigDetails({ customerCompanyId }));
    }
  }, [dispatch, customerCompanyId, isNewConfig, connectionForm]);

  // ── Show error notifications ──────────────────────────────
  useEffect(() => {
    if (error) showNotification(error, 'error');
  }, [error, showNotification]);

  // ── Auto-validate to get per-step column snapshots ────────
  const validateTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const pipelineActions = config?.pipeline?.actions;

  useEffect(() => {
    if (!config || !pipelineActions?.length) return;
    clearTimeout(validateTimerRef.current);
    validateTimerRef.current = setTimeout(() => {
      dispatch(validateConfigThunk({ customerCompanyId, data: config }));
    }, 400);
    return () => clearTimeout(validateTimerRef.current);
  }, [dispatch, customerCompanyId, config, pipelineActions]);

  // ── Real-time flow updates from chat ──────────────────────
  const processedActionsRef = useRef(new Set<string>());

  useEffect(() => {
    if (lastFlowAction && !processedActionsRef.current.has(lastFlowAction.id)) {
      processedActionsRef.current.add(lastFlowAction.id);
      dispatch(addFlowAction(lastFlowAction));
    }
  }, [lastFlowAction, dispatch]);

  const handleNodeClick: NodeMouseHandler = useCallback(
    (_event, node) => {
      dispatch(selectNodeAction(node));
    },
    [dispatch],
  );

  const handlePaneClick = useCallback(() => {
    dispatch(deselectNode());
  }, [dispatch]);

  const handleNodesChange = useCallback(
    (changes: NodeChange[]) => {
      dispatch(onNodesChangeAction(changes));
    },
    [dispatch],
  );

  const handleEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      dispatch(onEdgesChangeAction(changes));
    },
    [dispatch],
  );

  const handleToggleChat = useCallback(() => {
    dispatch(toggleChat());
  }, [dispatch]);

  const handleToggleAddStep = useCallback(() => {
    dispatch(toggleAddStepPanel());
  }, [dispatch]);

  const handleCloseAddStep = useCallback(() => {
    dispatch(setAddStepPanelOpen(false));
  }, [dispatch]);

  const handleBack = useCallback(() => {
    navigate(-1);
  }, [navigate]);

  const handleSave = useCallback(async () => {
    if (!config) return;

    try {
      if (isNewConfig) {
        const newId = config.name
          .toLowerCase()
          .replace(/[^a-z0-9]+/g, '-')
          .replace(/^-|-$/g, '')
          || 'new-config';

        const result = await dispatch(
          createConfigThunk({ customerCompanyId: newId, data: config }),
        ).unwrap();

        showNotification(t('configDetails.createSuccess'), 'success');
        navigate(`/config/${result.customerCompanyId}`, { replace: true });
      } else {
        await dispatch(
          saveConfigThunk({ customerCompanyId, data: config }),
        ).unwrap();

        showNotification(t('configDetails.saveSuccess'), 'success');
      }
    } catch {
      // Error is already set in Redux state by the rejected thunk
    }
  }, [config, isNewConfig, customerCompanyId, dispatch, navigate, showNotification, t]);

  const handleDelete = useCallback(async () => {
    if (isNewConfig) return;
    const confirmed = window.confirm(t('configDetails.deleteConfirm'));
    if (!confirmed) return;

    try {
      await dispatch(deleteConfigThunk({ customerCompanyId })).unwrap();
      showNotification(t('configDetails.deleteSuccess'), 'success');
      navigate('/config', { replace: true });
    } catch {
      // Error is already set in Redux state by the rejected thunk
    }
  }, [isNewConfig, customerCompanyId, dispatch, navigate, showNotification, t]);

  const memoizedNodeTypes = useMemo(() => nodeTypes, []);

  const defaultEdgeOptions = useMemo(
    () => ({
      animated: true,
      style: { strokeWidth: 2 },
    }),
    [],
  );

  if (isLoading) {
    return (
      <Flex align="center" justify="center" h="100%" gap="3" color="gray.500">
        <Spinner size="lg" />
        <Text>{t('configDetails.loading')}</Text>
      </Flex>
    );
  }

  if (error) {
    return (
      <Flex direction="column" align="center" justify="center" h="100%" gap="3" color="gray.500">
        <Text fontSize="2xl">⚠️</Text>
        <Text>{error}</Text>
        <Button variant="outline" onClick={handleBack}>
          {t('configDetails.backToConfigurations')}
        </Button>
      </Flex>
    );
  }

  if (!config) return null;

  return (
    <Flex direction="column" h="100%" bg="gray.50">
      {/* Header */}
      <Flex as="header" align="center" justify="space-between" px="6" py="3" bg="white" borderBottom="1px solid" borderColor="gray.200">
        <Flex align="center" gap="3">
          <Button variant="ghost" size="sm" onClick={handleBack}>
            {t('configDetails.back')}
          </Button>
          <Heading size="md" fontWeight="600">{config.name}</Heading>
          <Badge colorPalette="blue">{humanFrequency(config.cron)}</Badge>
        </Flex>
        <Flex align="center" gap="2">
          <Button variant="ghost" size="sm" onClick={handleToggleAddStep}>
            ➕ {t('configDetails.addStep')}
          </Button>
          <Button variant="ghost" size="sm" onClick={handleToggleChat}>
            💬 {chatOpen ? t('configDetails.closeChat') : t('configDetails.openChat')}
          </Button>
          <Button colorPalette="blue" size="sm" onClick={handleSave} disabled={isSaving}>
            {isSaving ? t('configDetails.saving') : t('configDetails.save')}
          </Button>
          {!isNewConfig && (
            <Button colorPalette="red" variant="outline" size="sm" onClick={handleDelete} disabled={isDeleting}>
              {isDeleting ? t('configDetails.deleting') : t('configDetails.delete')}
            </Button>
          )}
        </Flex>
      </Flex>

      {/* Body */}
      <Flex flex="1" overflow="hidden">
        {/* Canvas */}
        <Box flex="1" position="relative" role="application" aria-label="Pipeline flow editor">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            nodeTypes={memoizedNodeTypes}
            defaultEdgeOptions={defaultEdgeOptions}
            onNodesChange={handleNodesChange}
            onEdgesChange={handleEdgesChange}
            onNodeClick={handleNodeClick}
            onPaneClick={handlePaneClick}
            fitView
            fitViewOptions={{ padding: 0.2 }}
            proOptions={{ hideAttribution: true }}
          >
            <MiniMap nodeStrokeWidth={3} zoomable pannable />
            <Controls />
            <Background variant={BackgroundVariant.Dots} gap={16} size={1} />
          </ReactFlow>

          {selectedNode && <ActionEditPanel />}
          {addStepOpen && <AddStepPanel onClose={handleCloseAddStep} />}
        </Box>

        {/* Chat Panel */}
        <Box
          as="aside"
          w={chatOpen ? '380px' : '0px'}
          overflow="hidden"
          transition="width 0.2s ease"
          borderLeft={chatOpen ? '1px solid' : 'none'}
          borderColor="gray.200"
          bg="white"
          flexShrink={0}
        >
          {chatOpen && <ChatWindow onClose={handleToggleChat} />}
        </Box>
      </Flex>
    </Flex>
  );
}

export function ConfigDetailsScreen() {
  const { t } = useTranslation();
  const { customerCompanyId } = useParams<{ customerCompanyId: string }>();
  const location = useLocation();

  const isNewConfig = customerCompanyId === 'new';
  const connectionForm = (location.state as { connection?: ConnectionForm } | null)
    ?.connection;

  if (isNewConfig && !connectionForm) {
    return (
      <Flex direction="column" align="center" justify="center" h="100%" gap="3" color="gray.500">
        <Text fontSize="2xl">⚠️</Text>
        <Text>{t('configDetails.noConnectionData')}</Text>
      </Flex>
    );
  }

  if (!customerCompanyId) {
    return (
      <Flex direction="column" align="center" justify="center" h="100%" gap="3" color="gray.500">
        <Text fontSize="2xl">⚠️</Text>
        <Text>{t('configDetails.noConfigId')}</Text>
      </Flex>
    );
  }

  return (
    <ConfigDetailsContent
      customerCompanyId={customerCompanyId}
      isNewConfig={isNewConfig}
      connectionForm={connectionForm}
    />
  );
}

/** @deprecated Use `ConfigDetailsScreen` instead. Kept for backward compatibility during migration. */
export const ConfigDetailsPage = ConfigDetailsScreen;
