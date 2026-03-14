import { useCallback, useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Text, Heading, chakra } from '@chakra-ui/react';
import { useAppDispatch, useAppSelector, store } from '@/store';
import type { RootState } from '@/store';
import type { ActionConfigPayload, ActionType } from '@/generated/api';
import { businessLabel } from '@/shared/domain/types';
import { ACTION_FIELD_SCHEMAS, ACTION_CATALOG } from '../../domain/actionCatalog';
import {
  selectSelectedNode,
  selectAvailableColumnsForAction,
  deselectNode,
  removeFlowAction,
  updateFlowActionConfig,
} from '../../state/configDetailsSlice';
import { FieldEditor } from './FieldEditor';
import { getActionPanel } from './action-panels/registry';
import {
  ImportIcon,
  CogIcon,
  ExportIcon,
  WrenchIcon,
  CloseIcon,
  TrashIcon,
  AlertTriangleIcon,
} from '@/shared/ui';

const StyledButton = chakra('button');

/* ── Category icons ────────────────────────────────────────── */
const CATEGORY_ICONS: Record<string, React.ComponentType<{ size?: number | string }>> = {
  ingestion: ImportIcon,
  logic: CogIcon,
  egress: ExportIcon,
};

/* ── helpers ───────────────────────────────────────────────── */

function getField(config: Record<string, unknown>, key: string): unknown {
  return config[key];
}

function setField(
  config: Record<string, unknown>,
  key: string,
  value: unknown,
): Record<string, unknown> {
  return { ...config, [key]: value };
}

/* ── Main panel ────────────────────────────────────────────── */

export function ActionEditPanel() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const selectedNode = useAppSelector(selectSelectedNode);
  const [confirmRemove, setConfirmRemove] = useState(false);

  const handleClose = useCallback(() => {
    dispatch(deselectNode());
    setConfirmRemove(false);
  }, [dispatch]);

  const nodeData = selectedNode?.data as Record<string, unknown> | undefined;
  const actionId = (nodeData?.actionId as string) ?? selectedNode?.id;
  const actionType = (nodeData?.actionType as ActionType) ?? '';
  const category = (nodeData?.category as string) ?? 'logic';
  const config = nodeData?.config as ActionConfigPayload | undefined;
  const label = (nodeData?.label as string) ?? businessLabel(actionType);

  const availableColumns = useAppSelector(
    (state: RootState) => selectAvailableColumnsForAction(state, actionId ?? ''),
  );

  const validationError = useAppSelector(
    (state: RootState) => actionId ? state.configDetails.validationErrors[actionId] : undefined,
  );

  const catalogEntry = useMemo(
    () => ACTION_CATALOG.find((a) => a.actionType === actionType),
    [actionType],
  );

  const fieldSchemas = useMemo(
    () => ACTION_FIELD_SCHEMAS[actionType as ActionType] ?? [],
    [actionType],
  );

  const isIngestion = category === 'ingestion';

  const handleFieldChange = useCallback(
    (key: string, value: unknown) => {
      if (!actionId) return;
      // Read the current config from the store to avoid stale closure issues
      // when multiple onChange calls happen in the same tick.
      const state = store.getState();
      const action = state.configDetails.config?.pipeline.actions.find((a) => a.id === actionId);
      const freshConfig = action?.config;
      const configObj =
        typeof freshConfig === 'object' && freshConfig !== null
          ? (freshConfig as Record<string, unknown>)
          : {};
      const updated = setField(configObj, key, value);
      dispatch(updateFlowActionConfig({ actionId, config: updated as ActionConfigPayload }));
    },
    [actionId, dispatch],
  );

  const handleRemove = useCallback(() => {
    if (!actionId) return;
    if (!confirmRemove) {
      setConfirmRemove(true);
      return;
    }
    dispatch(removeFlowAction(actionId));
    setConfirmRemove(false);
  }, [actionId, confirmRemove, dispatch]);

  if (!selectedNode) return null;

  const configObj =
    typeof config === 'object' && config !== null
      ? (config as Record<string, unknown>)
      : null;

  /* ── Check for a custom per-action panel ────────────────── */
  const CustomPanel = getActionPanel(actionType as ActionType);

  return (
    <Box
      position="absolute"
      top="4"
      right="4"
      w="380px"
      bg="white"
      borderRadius="lg"
      border="1px solid"
      borderColor="tertiary.200"
      shadow="xl"
      zIndex="10"
      display="flex"
      flexDirection="column"
      maxH="calc(100% - 200px)"
      data-testid="action-edit-panel"
    >
      {/* Header */}
      <Flex
        align="center"
        justify="space-between"
        px="4"
        py="3"
        borderBottom="1px solid"
        borderColor="tertiary.100"
        bg="tertiary.50"
        borderTopRadius="lg"
      >
        <Flex align="center" gap="2">
          <Box color="secondary.500">
            {(() => { const CatIcon = CATEGORY_ICONS[category] ?? WrenchIcon; return <CatIcon size="1.25em" />; })()}
          </Box>
          <Box>
            <Heading size="sm" color="primary.500">{label}</Heading>
            <Text fontSize="xs" color="tertiary.500">
              {t(`configDetails.form.categoryLabels.${category}`, category)}
            </Text>
          </Box>
        </Flex>
        <StyledButton
          onClick={handleClose}
          aria-label={t('common.close', 'Close')}
          cursor="pointer"
          color="tertiary.400"
          _hover={{ color: 'tertiary.600' }}
          bg="transparent"
          border="none"
          p="0"
          display="flex"
          data-testid="action-edit-close"
        >
          <CloseIcon size="1em" />
        </StyledButton>
      </Flex>

      {/* Description */}
      {catalogEntry?.description && (
        <Text
          fontSize="sm"
          color="secondary.700"
          px="4"
          py="2"
          borderBottom="1px solid"
          borderColor="secondary.50"
          bg="secondary.50"
        >
          {catalogEntry.description}
        </Text>
      )}

      {/* Validation error */}
      {validationError && (
        <Box
          px="4"
          py="2"
          bg="red.50"
          borderBottom="1px solid"
          borderColor="red.200"
          data-testid="action-validation-error"
        >
          <Flex align="center" gap="1" mb="0.5">
            <Box color="red.600"><AlertTriangleIcon size="0.875em" /></Box>
            <Text fontSize="xs" fontWeight="600" color="red.700">
              Validation Error
            </Text>
          </Flex>
          <Text fontSize="xs" color="red.600" whiteSpace="pre-wrap">
            {validationError}
          </Text>
        </Box>
      )}

      {/* Fields — either a custom panel, generic schema-driven fields, or raw fallback */}
      <Box flex="1" overflowY="auto" p="4" display="flex" flexDirection="column" gap="4">
        {CustomPanel && configObj ? (
          <CustomPanel
            config={configObj}
            onChange={handleFieldChange}
            availableColumns={availableColumns}
          />
        ) : fieldSchemas.length > 0 && configObj ? (
          fieldSchemas.map((schema) => (
            <Box as="section" key={schema.key}>
              <Heading as="h3" fontSize="sm" fontWeight="600" mb="1">
                {schema.label}
              </Heading>
              {schema.hint && (
                <Text fontSize="xs" color="gray.500" mb="2">
                  {schema.hint}
                </Text>
              )}
              <FieldEditor
                schema={schema}
                value={getField(configObj, schema.key)}
                onChange={handleFieldChange}
                availableColumns={availableColumns}
              />
            </Box>
          ))
        ) : configObj ? (
          Object.entries(configObj).map(([key, value]) => (
            <Box as="section" key={key}>
              <Heading as="h3" fontSize="sm" fontWeight="600" mb="1">
                {key}
              </Heading>
              <Text fontSize="sm" color="gray.600" whiteSpace="pre-wrap">
                {typeof value === 'string' || typeof value === 'number'
                  ? String(value)
                  : JSON.stringify(value, null, 2)}
              </Text>
            </Box>
          ))
        ) : (
          <Box as="section">
            <Heading as="h3" fontSize="sm" fontWeight="600" mb="1">
              {t('flow.edit.configuration', 'Configuration')}
            </Heading>
            <Text fontSize="sm" color="gray.600">
              {String(config ?? '—')}
            </Text>
          </Box>
        )}
      </Box>

      {/* Footer */}
      {!isIngestion && (
        <Box px="4" py="3" borderTop="1px solid" borderColor="tertiary.100">
          <StyledButton
            w="full"
            py="2"
            borderRadius="md"
            border="1px solid"
            borderColor={confirmRemove ? 'red.300' : 'tertiary.300'}
            bg={confirmRemove ? 'red.50' : 'white'}
            color={confirmRemove ? 'red.600' : 'tertiary.600'}
            cursor="pointer"
            fontSize="sm"
            fontWeight="500"
            transition="all 0.15s"
            display="flex"
            alignItems="center"
            justifyContent="center"
            gap="2"
            _hover={{ borderColor: 'red.300', bg: 'red.50', color: 'red.600' }}
            onClick={handleRemove}
            data-testid="action-edit-remove"
          >
            {confirmRemove
              ? (<><AlertTriangleIcon size="0.875em" /> {t('flow.edit.confirmRemove', 'Click again to confirm removal')}</>)
              : (<><TrashIcon size="0.875em" /> {t('flow.edit.removeStep', 'Remove this step')}</>)}
          </StyledButton>
        </Box>
      )}
    </Box>
  );
}
