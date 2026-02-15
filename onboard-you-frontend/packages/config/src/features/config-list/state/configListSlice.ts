import {
  createSlice,
  createAsyncThunk,
  createSelector,
  type PayloadAction,
} from '@reduxjs/toolkit';
import type { RootState, ThunkExtra } from '@/store';
import type { PipelineConfig } from '@/shared/domain/types';
import type { ConfigListState } from '../domain/types';
import { fetchConfigs as fetchConfigsService } from '../services';

/* ── Initial state ─────────────────────────────────────────── */

const initialState: ConfigListState = {
  configs: [],
  isLoading: false,
  error: null,
  searchQuery: '',
};

/* ── Async thunks ──────────────────────────────────────────── */

export const fetchConfigs = createAsyncThunk<
  PipelineConfig[],
  void,
  { extra: ThunkExtra }
>(
  'configList/fetchConfigs',
  async (_arg, { rejectWithValue, extra }) => {
    try {
      return await fetchConfigsService(extra.apiClient);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to fetch configurations';
      return rejectWithValue(message);
    }
  },
);

/* ── Slice ─────────────────────────────────────────────────── */

const configListSlice = createSlice({
  name: 'configList',
  initialState,
  reducers: {
    setSearchQuery(state, action: PayloadAction<string>) {
      state.searchQuery = action.payload;
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(fetchConfigs.pending, (state) => {
        state.isLoading = true;
        state.error = null;
      })
      .addCase(fetchConfigs.fulfilled, (state, action) => {
        state.isLoading = false;
        state.configs = action.payload;
        state.error = null;
      })
      .addCase(fetchConfigs.rejected, (state, action) => {
        state.isLoading = false;
        state.error = (action.payload as string) ?? 'Failed to fetch configurations';
      });
  },
});

export const { setSearchQuery } = configListSlice.actions;

/* ── Selectors ─────────────────────────────────────────────── */

export const selectConfigList = (state: RootState) => state.configList;
export const selectIsLoading = (state: RootState) => state.configList.isLoading;
export const selectSearchQuery = (state: RootState) => state.configList.searchQuery;
export const selectConfigListError = (state: RootState) => state.configList.error;

export const selectFilteredConfigs = createSelector(
  [(state: RootState) => state.configList.configs, (state: RootState) => state.configList.searchQuery],
  (configs, searchQuery) => {
    if (!searchQuery) return configs;
    const q = searchQuery.toLowerCase();
    return configs.filter(
      (c) =>
        c.name.toLowerCase().includes(q) ||
        c.customerCompanyId.toLowerCase().includes(q),
    );
  },
);

export default configListSlice.reducer;
