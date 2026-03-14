import { useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Box,
  Flex,
  Text,
  Spinner,
  Badge,
  Button,
  Input,
  Table,
} from '@chakra-ui/react';
import { AlertTriangleIcon, ArrowLeftIcon } from '@/shared/ui';

import { useAppDispatch, useAppSelector } from '@/store';
import { businessLabel } from '@/shared/domain/types';
import {
  fetchRunHistory,
  fetchRunDetail,
  setSearchQuery,
  setSort,
  selectFilteredRuns,
  selectRunHistoryLoading,
  selectRunDetailLoading,
  selectRunHistoryError,
  selectCurrentPage,
  selectLastPage,
  selectSelectedRun,
} from '../../state/runHistorySlice';
import type { RunHistoryState } from '../../state/runHistorySlice';
import type { PipelineRun } from '@/generated/api';

/* ── Status badge helpers ──────────────────────────────────── */

const STATUS_COLORS: Record<string, string> = {
  completed: 'green',
  running: 'blue',
  failed: 'red',
  validation_failed: 'orange',
};

function statusColor(status: string): string {
  return STATUS_COLORS[status] ?? 'gray';
}

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

function duration(startedAt: string, finishedAt?: string | null): string {
  if (!finishedAt) return '—';
  const ms = new Date(finishedAt).getTime() - new Date(startedAt).getTime();
  if (ms < 1000) return `${ms}ms`;
  const s = Math.floor(ms / 1000);
  if (s < 60) return `${s}s`;
  return `${Math.floor(s / 60)}m ${s % 60}s`;
}

/* ── Sort header helper ────────────────────────────────────── */

function SortHeader({
  field,
  label,
  sortField,
  sortDirection,
  onSort,
}: {
  field: 'startedAt' | 'status';
  label: string;
  sortField: string;
  sortDirection: string;
  onSort: (field: 'startedAt' | 'status') => void;
}) {
  const arrow = sortField === field ? (sortDirection === 'asc' ? ' ▲' : ' ▼') : '';
  return (
    <Table.ColumnHeader
      cursor="pointer"
      onClick={() => onSort(field)}
      userSelect="none"
      data-testid={`sort-${field}`}
    >
      {label}{arrow}
    </Table.ColumnHeader>
  );
}

/* ── Main component ────────────────────────────────────────── */

export function RunHistoryTab({
  customerCompanyId,
  onSelectRun,
}: {
  customerCompanyId: string;
  onSelectRun: (run: PipelineRun) => void;
}) {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();

  const runs = useAppSelector(selectFilteredRuns);
  const isLoading = useAppSelector(selectRunHistoryLoading);
  const isLoadingDetail = useAppSelector(selectRunDetailLoading);
  const error = useAppSelector(selectRunHistoryError);
  const currentPage = useAppSelector(selectCurrentPage);
  const lastPage = useAppSelector(selectLastPage);
  const selectedRun = useAppSelector(selectSelectedRun);
  const sortField = useAppSelector((s) => s.runHistory.sortField);
  const sortDirection = useAppSelector((s) => s.runHistory.sortDirection);
  const searchQuery = useAppSelector((s) => s.runHistory.searchQuery);

  useEffect(() => {
    dispatch(fetchRunHistory({ customerCompanyId }));
  }, [dispatch, customerCompanyId]);

  const handleSearch = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      dispatch(setSearchQuery(e.target.value));
    },
    [dispatch],
  );

  const handleSort = useCallback(
    (field: 'startedAt' | 'status') => {
      const direction =
        sortField === field && sortDirection === 'asc' ? 'desc' : 'asc';
      dispatch(setSort({ field, direction }));
    },
    [dispatch, sortField, sortDirection],
  );

  const handleSelectRun = useCallback(
    (run: PipelineRun) => {
      dispatch(
        fetchRunDetail({ customerCompanyId, runId: run.id }),
      ).unwrap().then(onSelectRun);
    },
    [dispatch, customerCompanyId, onSelectRun],
  );

  const handlePageChange = useCallback(
    (page: number) => {
      dispatch(fetchRunHistory({ customerCompanyId, page }));
    },
    [dispatch, customerCompanyId],
  );

  if (isLoading && runs.length === 0) {
    return (
      <Flex align="center" justify="center" py="16" gap="3" color="gray.500">
        <Spinner size="lg" />
        <Text>{t('runHistory.loading', 'Loading run history…')}</Text>
      </Flex>
    );
  }

  if (error) {
    return (
      <Flex direction="column" align="center" justify="center" py="16" gap="3" color="gray.500">
        <AlertTriangleIcon size="1.5em" />
        <Text>{error}</Text>
      </Flex>
    );
  }

  return (
    <Box data-testid="run-history-tab">
      {/* Search */}
      <Flex px="4" py="3" gap="3" align="center">
        <Input
          placeholder={t('runHistory.search', 'Search runs…')}
          value={searchQuery}
          onChange={handleSearch}
          size="sm"
          maxW="320px"
          data-testid="run-history-search"
        />
        {isLoading && <Spinner size="sm" />}
      </Flex>

      {/* Table */}
      {runs.length === 0 ? (
        <Flex align="center" justify="center" py="16" color="gray.500">
          <Text>{t('runHistory.empty', 'No runs found')}</Text>
        </Flex>
      ) : (
        <>
          <Table.Root size="sm" variant="outline">
            <Table.Header>
              <Table.Row>
                <SortHeader
                  field="startedAt"
                  label={t('runHistory.time', 'Time')}
                  sortField={sortField}
                  sortDirection={sortDirection}
                  onSort={handleSort}
                />
                <SortHeader
                  field="status"
                  label={t('runHistory.status', 'Status')}
                  sortField={sortField}
                  sortDirection={sortDirection}
                  onSort={handleSort}
                />
                <Table.ColumnHeader>{t('runHistory.duration', 'Duration')}</Table.ColumnHeader>
                <Table.ColumnHeader>{t('runHistory.rows', 'Rows')}</Table.ColumnHeader>
                <Table.ColumnHeader>{t('runHistory.warnings', 'Warnings')}</Table.ColumnHeader>
                <Table.ColumnHeader>{t('runHistory.error', 'Error')}</Table.ColumnHeader>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {runs.map((run) => (
                <Table.Row
                  key={run.id}
                  cursor="pointer"
                  _hover={{ bg: 'gray.50' }}
                  bg={selectedRun?.id === run.id ? 'blue.50' : undefined}
                  onClick={() => handleSelectRun(run)}
                  data-testid={`run-row-${run.id}`}
                >
                  <Table.Cell fontSize="xs">{formatDate(run.startedAt)}</Table.Cell>
                  <Table.Cell>
                    <Badge colorPalette={statusColor(run.status)} size="sm">
                      {run.status}
                    </Badge>
                  </Table.Cell>
                  <Table.Cell fontSize="xs">
                    {duration(run.startedAt, run.finishedAt)}
                  </Table.Cell>
                  <Table.Cell fontSize="xs">{run.rowsProcessed ?? '—'}</Table.Cell>
                  <Table.Cell fontSize="xs">{run.warnings.length}</Table.Cell>
                  <Table.Cell fontSize="xs" color={run.errorMessage ? 'red.600' : undefined} maxW="200px" truncate>
                    {run.errorActionId
                      ? `${businessLabel(run.errorActionId)}: ${run.errorMessage ?? ''}`
                      : run.errorMessage ?? '—'}
                  </Table.Cell>
                </Table.Row>
              ))}
            </Table.Body>
          </Table.Root>

          {/* Pagination */}
          {lastPage > 1 && (
            <Flex justify="center" py="3" gap="2" align="center">
              <Button
                size="xs"
                variant="ghost"
                disabled={currentPage <= 1}
                onClick={() => handlePageChange(currentPage - 1)}
                data-testid="prev-page"
              >
                <ArrowLeftIcon size="0.85em" /> {t('runHistory.prev', 'Prev')}
              </Button>
              <Text fontSize="xs" color="gray.500">
                {t('runHistory.page', 'Page {{current}} of {{last}}', {
                  current: currentPage,
                  last: lastPage,
                })}
              </Text>
              <Button
                size="xs"
                variant="ghost"
                disabled={currentPage >= lastPage}
                onClick={() => handlePageChange(currentPage + 1)}
                data-testid="next-page"
              >
                {t('runHistory.next', 'Next')} <ArrowLeftIcon size="0.85em" style={{ transform: 'rotate(180deg)' }} />
              </Button>
            </Flex>
          )}
        </>
      )}
    </Box>
  );
}
