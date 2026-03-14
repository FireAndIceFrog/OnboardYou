import {
  createSlice,
  createAsyncThunk,
  createSelector,
  type PayloadAction,
} from '@reduxjs/toolkit';
import type { RootState, ThunkExtra } from '@/store';
import type { PipelineRun, ListResponsePipelineRun } from '@/generated/api';
import { fetchRuns as fetchRunsService, fetchRun as fetchRunService } from '../services/runHistoryService';

/* ── State shape ───────────────────────────────────────────── */

export interface RunHistoryState {
  runs: PipelineRun[];
  selectedRun: PipelineRun | null;
  currentPage: number;
  lastPage: number;
  countPerPage: number;
  isLoadingList: boolean;
  isLoadingDetail: boolean;
  error: string | null;
  searchQuery: string;
  sortField: 'startedAt' | 'status';
  sortDirection: 'asc' | 'desc';
}

const initialState: RunHistoryState = {
  runs: [],
  selectedRun: null,
  currentPage: 1,
  lastPage: 1,
  countPerPage: 20,
  isLoadingList: false,
  isLoadingDetail: false,
  error: null,
  searchQuery: '',
  sortField: 'startedAt',
  sortDirection: 'desc',
};

/* ── Async thunks ──────────────────────────────────────────── */

export const fetchRunHistory = createAsyncThunk<
  ListResponsePipelineRun,
  { customerCompanyId: string; page?: number },
  { extra: ThunkExtra }
>(
  'runHistory/fetchRunHistory',
  async ({ customerCompanyId, page = 1 }, { rejectWithValue, getState }) => {
    try {
      const { runHistory } = getState() as RootState;
      return await fetchRunsService(customerCompanyId, page, runHistory.countPerPage);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to fetch run history';
      return rejectWithValue(message);
    }
  },
);

export const fetchRunDetail = createAsyncThunk<
  PipelineRun,
  { customerCompanyId: string; runId: string },
  { extra: ThunkExtra }
>(
  'runHistory/fetchRunDetail',
  async ({ customerCompanyId, runId }, { rejectWithValue }) => {
    try {
      return await fetchRunService(customerCompanyId, runId);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to fetch run details';
      return rejectWithValue(message);
    }
  },
);

/* ── Slice ─────────────────────────────────────────────────── */

const runHistorySlice = createSlice({
  name: 'runHistory',
  initialState,
  reducers: {
    setSearchQuery(state, action: PayloadAction<string>) {
      state.searchQuery = action.payload;
    },
    setSort(state, action: PayloadAction<{ field: 'startedAt' | 'status'; direction: 'asc' | 'desc' }>) {
      state.sortField = action.payload.field;
      state.sortDirection = action.payload.direction;
    },
    clearSelectedRun(state) {
      state.selectedRun = null;
    },
    resetRunHistory() {
      return initialState;
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(fetchRunHistory.pending, (state) => {
        state.isLoadingList = true;
        state.error = null;
      })
      .addCase(fetchRunHistory.fulfilled, (state, action) => {
        state.isLoadingList = false;
        state.runs = action.payload.data;
        state.currentPage = action.payload.currentPage;
        state.lastPage = action.payload.lastPage;
        state.countPerPage = action.payload.countPerPage;
      })
      .addCase(fetchRunHistory.rejected, (state, action) => {
        state.isLoadingList = false;
        state.error = (action.payload as string) ?? 'Failed to fetch run history';
      })
      .addCase(fetchRunDetail.pending, (state) => {
        state.isLoadingDetail = true;
        state.error = null;
      })
      .addCase(fetchRunDetail.fulfilled, (state, action) => {
        state.isLoadingDetail = false;
        state.selectedRun = action.payload;
      })
      .addCase(fetchRunDetail.rejected, (state, action) => {
        state.isLoadingDetail = false;
        state.error = (action.payload as string) ?? 'Failed to fetch run details';
      });
  },
});

export const { setSearchQuery, setSort, clearSelectedRun, resetRunHistory } = runHistorySlice.actions;

/* ── Selectors ─────────────────────────────────────────────── */

export const selectRunHistory = (state: RootState) => state.runHistory;
export const selectRuns = (state: RootState) => state.runHistory.runs;
export const selectSelectedRun = (state: RootState) => state.runHistory.selectedRun;
export const selectRunHistoryLoading = (state: RootState) => state.runHistory.isLoadingList;
export const selectRunDetailLoading = (state: RootState) => state.runHistory.isLoadingDetail;
export const selectRunHistoryError = (state: RootState) => state.runHistory.error;
export const selectCurrentPage = (state: RootState) => state.runHistory.currentPage;
export const selectLastPage = (state: RootState) => state.runHistory.lastPage;

export const selectIsRunning = (state: RootState) => {
  const halfDayAgo = Date.now() - 12 * 60 * 60 * 1000;
  return state.runHistory.runs.some(
    (r) => r.status === 'running' && new Date(r.startedAt).getTime() > halfDayAgo,
  );
};

export const selectFilteredRuns = createSelector(
  [
    selectRuns,
    (state: RootState) => state.runHistory.searchQuery,
    (state: RootState) => state.runHistory.sortField,
    (state: RootState) => state.runHistory.sortDirection,
  ],
  (runs, searchQuery, sortField, sortDirection) => {
    let filtered = runs;

    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      filtered = runs.filter(
        (r) =>
          r.status.toLowerCase().includes(q) ||
          r.startedAt.toLowerCase().includes(q) ||
          (r.errorMessage?.toLowerCase().includes(q) ?? false) ||
          (r.errorActionId?.toLowerCase().includes(q) ?? false),
      );
    }

    const sorted = [...filtered].sort((a, b) => {
      if (sortField === 'startedAt') {
        const diff = new Date(a.startedAt).getTime() - new Date(b.startedAt).getTime();
        return sortDirection === 'asc' ? diff : -diff;
      }
      const diff = a.status.localeCompare(b.status);
      return sortDirection === 'asc' ? diff : -diff;
    });

    return sorted;
  },
);

export default runHistorySlice.reducer;
