import { describe, it, expect } from 'vitest';
import reducer, {
  setSearchQuery,
  setSort,
  clearSelectedRun,
  resetRunHistory,
  type RunHistoryState,
} from './runHistorySlice';

const initial: RunHistoryState = {
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

describe('runHistorySlice', () => {
  it('returns initial state', () => {
    expect(reducer(undefined, { type: '@@INIT' })).toEqual(initial);
  });

  it('setSearchQuery updates searchQuery', () => {
    const next = reducer(initial, setSearchQuery('failed'));
    expect(next.searchQuery).toBe('failed');
  });

  it('setSort updates sort field and direction', () => {
    const next = reducer(initial, setSort({ field: 'status', direction: 'asc' }));
    expect(next.sortField).toBe('status');
    expect(next.sortDirection).toBe('asc');
  });

  it('clearSelectedRun nulls selectedRun', () => {
    const state = { ...initial, selectedRun: { id: 'run-1' } as RunHistoryState['selectedRun'] };
    const next = reducer(state, clearSelectedRun());
    expect(next.selectedRun).toBeNull();
  });

  it('resetRunHistory returns to initial state', () => {
    const state = { ...initial, searchQuery: 'test', currentPage: 3 };
    const next = reducer(state, resetRunHistory());
    expect(next).toEqual(initial);
  });
});
