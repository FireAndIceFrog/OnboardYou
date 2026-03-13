import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/shared/test/testWrapper';
import type { RunHistoryState } from '../../state/runHistorySlice';

const mockRun = {
  id: 'run-1',
  organizationId: 'org-1',
  customerCompanyId: 'test-company',
  status: 'completed',
  startedAt: '2026-03-10T10:00:00Z',
  finishedAt: '2026-03-10T10:05:00Z',
  rowsProcessed: 42,
  warnings: [
    { action_id: 'cellphone_sanitizer', message: 'Bad phone', count: 3 },
  ],
  currentAction: null,
  errorMessage: null,
  errorActionId: null,
  errorRow: null,
};

const failedRun = {
  ...mockRun,
  id: 'run-2',
  status: 'failed',
  errorMessage: 'Column not found',
  errorActionId: 'api_dispatcher',
  errorRow: 5,
  warnings: [],
};

function makeState(overrides: Partial<RunHistoryState> = {}) {
  return {
    runHistory: {
      runs: [mockRun, failedRun],
      selectedRun: null,
      currentPage: 1,
      lastPage: 1,
      countPerPage: 20,
      isLoadingList: false,
      isLoadingDetail: false,
      error: null,
      searchQuery: '',
      sortField: 'startedAt' as const,
      sortDirection: 'desc' as const,
      ...overrides,
    },
  };
}

// Mock the thunk to prevent network calls
vi.mock('../../state/runHistorySlice', async () => {
  const actual = await vi.importActual<Record<string, unknown>>('../../state/runHistorySlice');
  return {
    ...actual,
    fetchRunHistory: () => ({ type: 'runHistory/fetchRunHistory/noop' }),
    fetchRunDetail: () => ({ type: 'runHistory/fetchRunDetail/noop' }),
  };
});

describe('RunHistoryTab', () => {
  const onSelectRun = vi.fn();

  beforeEach(() => vi.clearAllMocks());

  it('renders run rows', async () => {
    const { RunHistoryTab } = await import('./RunHistoryTab');
    renderWithProviders(
      <RunHistoryTab customerCompanyId="test-company" onSelectRun={onSelectRun} />,
      { preloadedState: makeState() },
    );
    expect(screen.getByTestId('run-row-run-1')).toBeInTheDocument();
    expect(screen.getByTestId('run-row-run-2')).toBeInTheDocument();
  });

  it('shows status badges', async () => {
    const { RunHistoryTab } = await import('./RunHistoryTab');
    renderWithProviders(
      <RunHistoryTab customerCompanyId="test-company" onSelectRun={onSelectRun} />,
      { preloadedState: makeState() },
    );
    expect(screen.getByText('completed')).toBeInTheDocument();
    expect(screen.getByText('failed')).toBeInTheDocument();
  });

  it('shows loading state', async () => {
    const { RunHistoryTab } = await import('./RunHistoryTab');
    renderWithProviders(
      <RunHistoryTab customerCompanyId="test-company" onSelectRun={onSelectRun} />,
      { preloadedState: makeState({ isLoadingList: true, runs: [] }) },
    );
    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it('shows error state', async () => {
    const { RunHistoryTab } = await import('./RunHistoryTab');
    renderWithProviders(
      <RunHistoryTab customerCompanyId="test-company" onSelectRun={onSelectRun} />,
      { preloadedState: makeState({ error: 'Network error' }) },
    );
    expect(screen.getByText('Network error')).toBeInTheDocument();
  });

  it('shows empty state', async () => {
    const { RunHistoryTab } = await import('./RunHistoryTab');
    renderWithProviders(
      <RunHistoryTab customerCompanyId="test-company" onSelectRun={onSelectRun} />,
      { preloadedState: makeState({ runs: [] }) },
    );
    expect(screen.getByText(/no runs/i)).toBeInTheDocument();
  });

  it('shows error info for failed runs', async () => {
    const { RunHistoryTab } = await import('./RunHistoryTab');
    renderWithProviders(
      <RunHistoryTab customerCompanyId="test-company" onSelectRun={onSelectRun} />,
      { preloadedState: makeState() },
    );
    // Should show business label for the error action
    expect(screen.getByText(/Send to API/)).toBeInTheDocument();
  });

  it('has search input', async () => {
    const { RunHistoryTab } = await import('./RunHistoryTab');
    renderWithProviders(
      <RunHistoryTab customerCompanyId="test-company" onSelectRun={onSelectRun} />,
      { preloadedState: makeState() },
    );
    expect(screen.getByTestId('run-history-search')).toBeInTheDocument();
  });
});
