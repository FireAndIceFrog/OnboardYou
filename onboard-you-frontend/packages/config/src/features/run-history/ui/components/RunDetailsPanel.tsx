import { useState, useCallback, useMemo, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Box,
  Flex,
  Text,
  Heading,
  Badge,
  Button,
  Input,
  Table,
} from '@chakra-ui/react';

import { businessLabel } from '@/shared/domain/types';
import type { PipelineRun, PipelineWarning } from '@/generated/api';

/* ── Helpers ───────────────────────────────────────────────── */

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

const STATUS_COLORS: Record<string, string> = {
  completed: 'green',
  running: 'blue',
  failed: 'red',
  validation_failed: 'orange',
};

/* ── Warning grouping ──────────────────────────────────────── */

interface GroupedWarning {
  actionId: string;
  actionLabel: string;
  warnings: PipelineWarning[];
  totalCount: number;
}

function groupWarnings(warnings: PipelineWarning[]): GroupedWarning[] {
  const map = new Map<string, PipelineWarning[]>();
  for (const w of warnings) {
    const list = map.get(w.action_id) ?? [];
    list.push(w);
    map.set(w.action_id, list);
  }
  return Array.from(map.entries()).map(([actionId, wList]) => ({
    actionId,
    actionLabel: businessLabel(actionId),
    warnings: wList,
    totalCount: wList.reduce((sum, w) => sum + w.count, 0),
  }));
}

/* ── Resize hook ───────────────────────────────────────────── */

function useResize(initialWidth: number, minWidth: number, maxWidth: number) {
  const [width, setWidth] = useState(initialWidth);
  const dragging = useRef(false);
  const startX = useRef(0);
  const startWidth = useRef(initialWidth);

  const onMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      dragging.current = true;
      startX.current = e.clientX;
      startWidth.current = width;

      const onMouseMove = (ev: MouseEvent) => {
        if (!dragging.current) return;
        const delta = startX.current - ev.clientX;
        const next = Math.min(maxWidth, Math.max(minWidth, startWidth.current + delta));
        setWidth(next);
      };

      const onMouseUp = () => {
        dragging.current = false;
        document.removeEventListener('mousemove', onMouseMove);
        document.removeEventListener('mouseup', onMouseUp);
      };

      document.addEventListener('mousemove', onMouseMove);
      document.addEventListener('mouseup', onMouseUp);
    },
    [width, minWidth, maxWidth],
  );

  return { width, onMouseDown };
}

/* ── Main component ────────────────────────────────────────── */

export function RunDetailsPanel({
  run,
  onClose,
}: {
  run: PipelineRun;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const { width, onMouseDown } = useResize(420, 300, 800);
  const [search, setSearch] = useState('');

  const grouped = useMemo(() => groupWarnings(run.warnings), [run.warnings]);

  const filteredGroups = useMemo(() => {
    if (!search) return grouped;
    const q = search.toLowerCase();
    return grouped
      .map((g) => ({
        ...g,
        warnings: g.warnings.filter(
          (w) =>
            w.message.toLowerCase().includes(q) ||
            w.action_id.toLowerCase().includes(q) ||
            g.actionLabel.toLowerCase().includes(q),
        ),
      }))
      .filter((g) => g.warnings.length > 0);
  }, [grouped, search]);

  const totalWarnings = run.warnings.length;
  const hasError = !!run.errorMessage;

  return (
    <Flex
      position="absolute"
      top="0"
      right="0"
      bottom="0"
      w={`${width}px`}
      bg="white"
      borderLeft="1px solid"
      borderColor="gray.200"
      direction="column"
      shadow="lg"
      zIndex={10}
      data-testid="run-details-panel"
    >
      {/* Resize handle */}
      <Box
        position="absolute"
        top="0"
        left="0"
        bottom="0"
        w="4px"
        cursor="col-resize"
        bg="transparent"
        _hover={{ bg: 'blue.200' }}
        onMouseDown={onMouseDown}
        data-testid="resize-handle"
      />

      {/* Header */}
      <Flex
        align="center"
        justify="space-between"
        px="4"
        py="3"
        borderBottom="1px solid"
        borderColor="gray.200"
        flexShrink={0}
      >
        <Flex align="center" gap="2">
          <Heading size="sm">{t('runDetails.title', 'Run Details')}</Heading>
          <Badge colorPalette={STATUS_COLORS[run.status] ?? 'gray'} size="sm">
            {run.status}
          </Badge>
        </Flex>
        <Button variant="ghost" size="xs" onClick={onClose} data-testid="close-details">
          ✕
        </Button>
      </Flex>

      {/* Summary */}
      <Box px="4" py="3" borderBottom="1px solid" borderColor="gray.100" flexShrink={0}>
        <Flex gap="6" wrap="wrap" fontSize="xs" color="gray.600">
          <Box>
            <Text fontWeight="600">{t('runDetails.started', 'Started')}</Text>
            <Text>{formatDate(run.startedAt)}</Text>
          </Box>
          {run.finishedAt && (
            <Box>
              <Text fontWeight="600">{t('runDetails.finished', 'Finished')}</Text>
              <Text>{formatDate(run.finishedAt)}</Text>
            </Box>
          )}
          <Box>
            <Text fontWeight="600">{t('runDetails.rowsProcessed', 'Rows')}</Text>
            <Text>{run.rowsProcessed ?? '—'}</Text>
          </Box>
          <Box>
            <Text fontWeight="600">{t('runDetails.warningCount', 'Warnings')}</Text>
            <Text>{totalWarnings}</Text>
          </Box>
        </Flex>
      </Box>

      {/* Error section */}
      {hasError && (
        <Box
          px="4"
          py="3"
          bg="red.50"
          borderBottom="1px solid"
          borderColor="red.200"
          flexShrink={0}
          data-testid="run-error-section"
        >
          <Text fontSize="xs" fontWeight="600" color="red.700" mb="1">
            {t('runDetails.errorTitle', 'Error')}
            {run.errorActionId && ` — ${businessLabel(run.errorActionId)}`}
            {run.errorRow != null && ` (row ${run.errorRow})`}
          </Text>
          <Text fontSize="xs" color="red.600">
            {run.errorMessage}
          </Text>
        </Box>
      )}

      {/* Search */}
      <Box px="4" py="2" flexShrink={0}>
        <Input
          placeholder={t('runDetails.searchWarnings', 'Search warnings…')}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          size="sm"
          data-testid="warning-search"
        />
      </Box>

      {/* Warnings table */}
      <Box flex="1" overflow="auto" px="4" pb="4">
        {filteredGroups.length === 0 ? (
          <Flex align="center" justify="center" py="8" color="gray.500">
            <Text fontSize="sm">
              {totalWarnings === 0
                ? t('runDetails.noWarnings', 'No warnings')
                : t('runDetails.noMatch', 'No matching warnings')}
            </Text>
          </Flex>
        ) : (
          filteredGroups.map((group) => (
            <Box key={group.actionId} mb="4">
              <Flex align="center" gap="2" mb="2">
                <Text fontSize="xs" fontWeight="600">
                  {group.actionLabel}
                </Text>
                <Badge colorPalette="yellow" size="sm">
                  {group.totalCount} {t('runDetails.affected', 'affected')}
                </Badge>
              </Flex>
              <Table.Root size="sm" variant="outline">
                <Table.Header>
                  <Table.Row>
                    <Table.ColumnHeader>{t('runDetails.message', 'Message')}</Table.ColumnHeader>
                    <Table.ColumnHeader w="60px">{t('runDetails.count', 'Count')}</Table.ColumnHeader>
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  {group.warnings.map((w, i) => (
                    <Table.Row key={i}>
                      <Table.Cell fontSize="xs">
                        {w.message}
                        {w.detail && (
                          <Text as="span" color="gray.400" ml="1">
                            ({w.detail})
                          </Text>
                        )}
                      </Table.Cell>
                      <Table.Cell fontSize="xs" textAlign="center">
                        {w.count}
                      </Table.Cell>
                    </Table.Row>
                  ))}
                </Table.Body>
              </Table.Root>
            </Box>
          ))
        )}
      </Box>
    </Flex>
  );
}
