import { useMemo, useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useParams, useNavigate, useLocation } from 'react-router-dom';
import {
  ReactFlow,
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
import type { RootState } from '@/store';
import { useGlobal } from '@/shared/hooks';
import { humanFrequency } from '@/shared/domain/types';
import { AlertTriangleIcon, PlusIcon, PlayIcon, ArrowLeftIcon } from '@/shared/ui';
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
  toggleAddStepPanel,
  setAddStepPanelOpen,
  addFlowAction,
  selectConfig,
  selectNodes,
  selectEdges,
  selectSelectedNode,
  selectAddStepPanelOpen,
  selectConfigDetailsLoading,
  selectConfigDetailsSaving,
  selectConfigDetailsDeleting,
  selectConfigDetailsError,
  fetchSettingsSchemaThunk,
} from '../../state/configDetailsSlice';
import { triggerRun as triggerRunService } from '../../services/configDetailsService';
import { fetchRunHistory, selectIsRunning } from '@/features/run-history/state';
import { ActionEditPanel, AddButtonEdge, AddStepPanel, PipelineNode } from '../components';
import { RunHistoryView } from '@/features/run-history/ui/components';

const nodeTypes = {
  ingestion: PipelineNode,
  logic: PipelineNode,
  egress: PipelineNode,
};

const edgeTypes = {
  addButton: AddButtonEdge,
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
  const addStepOpen = useAppSelector(selectAddStepPanelOpen);

  // Derive a stable boolean so we don't re-render on every new {} reference.
  const hasValidationErrors = useAppSelector(
    (state: RootState) => Object.keys(state.configDetails.validationErrors).length > 0,
  );

  // ── Current / History tab state ———————————————————————————
  const [activeTab, setActiveTab] = useState<'current' | 'history'>('current');
  const [isTriggering, setIsTriggering] = useState(false);
  const isRunning = useAppSelector(selectIsRunning);

  // ── Fetch existing config or initialise a blank one ———————
  useEffect(() => {
    dispatch(fetchSettingsSchemaThunk());
    if (isNewConfig && connectionForm) {
      dispatch(initNewConfig(connectionForm));
    } else {
      dispatch(fetchConfigDetails({ customerCompanyId }));
    }
  }, [dispatch, customerCompanyId, isNewConfig, connectionForm]);

  // ── Fetch run history on mount to know if a run is in progress ——
  useEffect(() => {
    if (!isNewConfig) {
      dispatch(fetchRunHistory({ customerCompanyId }));
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dispatch, customerCompanyId, isNewConfig]);

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

  const handleToggleAddStep = useCallback(() => {
    dispatch(toggleAddStepPanel());
  }, [dispatch]);

  const handleTriggerRun = useCallback(async () => {
    setIsTriggering(true);
    try {
      await triggerRunService(customerCompanyId);
      showNotification(t('configDetails.triggerRunSuccess', 'Pipeline run triggered'), 'success');
      // Refresh run list so the button immediately reflects the running state
      dispatch(fetchRunHistory({ customerCompanyId }));
    } catch (err) {
      const message = err && typeof err === 'object' && 'error' in err
        ? (err as { error: string }).error
        : t('configDetails.triggerRunFailed', 'Failed to trigger run');
      showNotification(message, 'error');
    } finally {
      setIsTriggering(false);
    }
  }, [customerCompanyId, dispatch, showNotification, t]);

  const handleCloseAddStep = useCallback(() => {
    dispatch(setAddStepPanelOpen(false));
  }, [dispatch]);

  const handleBack = useCallback(() => {
    navigate(-1);
  }, [navigate]);

  const handleSave = useCallback(async () => {
    if (!config) return;

    if (hasValidationErrors) {
      showNotification(
        t('configDetails.validationBlocksSave', 'Cannot save: fix validation errors first'),
        'error',
      );
      return;
    }

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
    } catch (err) {
      const message = typeof err === 'string' ? err : t('configDetails.saveFailed', 'Save failed');
      showNotification(message, 'error');
    }
  }, [config, isNewConfig, customerCompanyId, dispatch, navigate, showNotification, t, hasValidationErrors]);

  const handleDelete = useCallback(async () => {
    if (isNewConfig) return;
    const confirmed = window.confirm(t('configDetails.deleteConfirm'));
    if (!confirmed) return;

    try {
      await dispatch(deleteConfigThunk({ customerCompanyId })).unwrap();
      showNotification(t('configDetails.deleteSuccess'), 'success');
      navigate('/config', { replace: true });
    } catch (err) {
      const message = typeof err === 'string' ? err : t('configDetails.deleteFailed', 'Delete failed');
      showNotification(message, 'error');
    }
  }, [isNewConfig, customerCompanyId, dispatch, navigate, showNotification, t]);

  const memoizedNodeTypes = useMemo(() => nodeTypes, []);
  const memoizedEdgeTypes = useMemo(() => edgeTypes, []);

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
      <Flex direction="column" align="center" justify="center" h="100%" gap="3" color="tertiary.500">
        <Box color="tertiary.400"><AlertTriangleIcon size="2em" /></Box>
        <Text>{error}</Text>
        <Button variant="outline" borderColor="tertiary.300" color="tertiary.600" onClick={handleBack}>
          {t('configDetails.backToConfigurations')}
        </Button>
      </Flex>
    );
  }

  if (!config) return null;

  return (
    <Flex direction="column" h="100%" bg="tertiary.50">
      {/* Header */}
      <Flex as="header" align="center" px="6" py="3" bg="white" borderBottom="1px solid" borderColor="tertiary.200">
        {/* Left: navigation + title */}
        <Flex align="center" gap="3" flex="1">
          <Button variant="ghost" size="sm" color="tertiary.600" onClick={handleBack}>
            <ArrowLeftIcon size="1em" /> {t('configDetails.back')}
          </Button>
          <Heading size="md" fontWeight="600" color="primary.500">{config.name}</Heading>
          <Badge colorPalette="blue">{humanFrequency(config.cron)}</Badge>
        </Flex>

        {/* Center: tab toggle (fixed position) */}
        {!isNewConfig && (
          <Flex bg="tertiary.100" borderRadius="md" p="1" gap="1" data-testid="tab-toggle">
            <Button
              size="xs"
              variant={activeTab === 'current' ? 'solid' : 'ghost'}
              bg={activeTab === 'current' ? 'secondary.500' : undefined}
              color={activeTab === 'current' ? 'white' : 'tertiary.600'}
              _hover={activeTab === 'current' ? { bg: 'secondary.600' } : undefined}
              onClick={() => setActiveTab('current')}
              data-testid="tab-current"
            >
              {t('configDetails.tabCurrent', 'Current')}
            </Button>
            <Button
              size="xs"
              variant={activeTab === 'history' ? 'solid' : 'ghost'}
              bg={activeTab === 'history' ? 'secondary.500' : undefined}
              color={activeTab === 'history' ? 'white' : 'tertiary.600'}
              _hover={activeTab === 'history' ? { bg: 'secondary.600' } : undefined}
              onClick={() => setActiveTab('history')}
              data-testid="tab-history"
            >
              {t('configDetails.tabHistory', 'History')}
            </Button>
          </Flex>
        )}

        {/* Right: action buttons — use visibility to reserve space on both tabs */}
        <Flex align="center" gap="2" flex="1" justify="flex-end" visibility={activeTab === 'current' ? 'visible' : 'hidden'}>
            <>
              <Button variant="outline" size="sm" borderColor="tertiary.300" color="tertiary.600" onClick={handleToggleAddStep}>
                <PlusIcon size="0.875em" /> {t('configDetails.addStep')}
              </Button>
              {!isNewConfig && (
                <Button
                  variant="outline"
                  size="sm"
                  borderColor={isRunning ? 'tertiary.300' : 'secondary.500'}
                  color={isRunning ? 'tertiary.500' : 'secondary.500'}
                  onClick={handleTriggerRun}
                  disabled={isTriggering || isRunning}
                  data-testid="trigger-run"
                >
                  {isRunning
                    ? t('configDetails.running', 'Running…')
                    : isTriggering
                      ? t('configDetails.triggering', 'Triggering…')
                      : (<><PlayIcon size="0.75em" /> {t('configDetails.runNow', 'Run Now')}</>)}
                </Button>
              )}
              <Button bg="primary.500" color="white" _hover={{ bg: 'primary.600' }} size="sm" onClick={handleSave} disabled={isSaving}>
                {isSaving ? t('configDetails.saving') : t('configDetails.save')}
              </Button>
              {!isNewConfig && (
                <Button variant="outline" size="sm" borderColor="tertiary.300" color="red.500" _hover={{ bg: 'red.50' }} onClick={handleDelete} disabled={isDeleting}>
                  {isDeleting ? t('configDetails.deleting') : t('configDetails.delete')}
                </Button>
              )}
            </>
        </Flex>
      </Flex>

      {/* Body */}
      <Flex flex="1" overflow="hidden">
        {activeTab === 'current' ? (
          /* ── Current: editable pipeline flow ── */
          <Box flex="1" position="relative" role="application" aria-label="Pipeline flow editor">
            <ReactFlow
              nodes={nodes}
              edges={edges}
              nodeTypes={memoizedNodeTypes}
              edgeTypes={memoizedEdgeTypes}
              defaultEdgeOptions={defaultEdgeOptions}
              onNodesChange={handleNodesChange}
              onEdgesChange={handleEdgesChange}
              onNodeClick={handleNodeClick}
              onPaneClick={handlePaneClick}
              fitView
              fitViewOptions={{ padding: 0.2 }}
              proOptions={{ hideAttribution: true }}
            >
              <Controls />
              <Background variant={BackgroundVariant.Dots} gap={16} size={1} />
            </ReactFlow>

            {selectedNode && <ActionEditPanel />}
            {addStepOpen && <AddStepPanel onClose={handleCloseAddStep} />}
          </Box>
        ) : (
          /* ── History: run list + detail view ── */
          <RunHistoryView customerCompanyId={customerCompanyId} />
        )}
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
      <Flex direction="column" align="center" justify="center" h="100%" gap="3" color="tertiary.500">
        <Box color="tertiary.400"><AlertTriangleIcon size="2em" /></Box>
        <Text>{t('configDetails.noConnectionData')}</Text>
      </Flex>
    );
  }

  if (!customerCompanyId) {
    return (
      <Flex direction="column" align="center" justify="center" h="100%" gap="3" color="tertiary.500">
        <Box color="tertiary.400"><AlertTriangleIcon size="2em" /></Box>
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
