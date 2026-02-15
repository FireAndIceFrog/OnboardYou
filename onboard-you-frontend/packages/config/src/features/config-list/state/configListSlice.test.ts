import { describe, it, expect } from 'vitest';
import reducer, {
  setSearchQuery,
  fetchConfigs,
  selectFilteredConfigs,
} from './configListSlice';
import type { PipelineConfig } from '@/shared/domain/types';

const initialState = {
  configs: [],
  isLoading: false,
  error: null,
  searchQuery: '',
};

const mockConfigs: PipelineConfig[] = [
  {
    name: 'Acme Corp Pipeline',
    cron: 'rate(1 day)',
    organizationId: 'org-1',
    customerCompanyId: 'acme-corp',
    lastEdited: '2026-01-01T00:00:00Z',
    pipeline: { version: '1', actions: [] },
  },
  {
    name: 'Beta Inc Pipeline',
    cron: 'rate(1 hour)',
    organizationId: 'org-1',
    customerCompanyId: 'beta-inc',
    lastEdited: '2026-02-01T00:00:00Z',
    pipeline: { version: '1', actions: [] },
  },
];

describe('configListSlice', () => {
  it('should return the initial state', () => {
    const state = reducer(undefined, { type: '@@INIT' });
    expect(state).toEqual(initialState);
  });

  it('setSearchQuery updates the search query', () => {
    const state = reducer(initialState, setSearchQuery('acme'));
    expect(state.searchQuery).toBe('acme');
  });

  it('fetchConfigs.pending sets loading true and clears error', () => {
    const state = reducer(
      { ...initialState, error: 'old error' },
      fetchConfigs.pending('req-id', undefined),
    );
    expect(state.isLoading).toBe(true);
    expect(state.error).toBeNull();
  });

  it('fetchConfigs.fulfilled sets configs and clears loading', () => {
    const state = reducer(
      { ...initialState, isLoading: true },
      fetchConfigs.fulfilled(mockConfigs, 'req-id', undefined),
    );
    expect(state.isLoading).toBe(false);
    expect(state.configs).toEqual(mockConfigs);
    expect(state.error).toBeNull();
  });

  it('fetchConfigs.rejected sets error and clears loading', () => {
    const state = reducer(
      { ...initialState, isLoading: true },
      fetchConfigs.rejected(null, 'req-id', undefined, 'Network error'),
    );
    expect(state.isLoading).toBe(false);
    expect(state.error).toBe('Network error');
  });

  describe('selectFilteredConfigs', () => {
    const stateWith = (searchQuery: string) => ({
      configList: { ...initialState, configs: mockConfigs, searchQuery },
    });

    it('returns all configs when search query is empty', () => {
      const result = selectFilteredConfigs(stateWith('') as never);
      expect(result).toHaveLength(2);
    });

    it('filters by config name', () => {
      const result = selectFilteredConfigs(stateWith('acme') as never);
      expect(result).toHaveLength(1);
      expect(result[0].name).toBe('Acme Corp Pipeline');
    });

    it('filters by customerCompanyId', () => {
      const result = selectFilteredConfigs(stateWith('beta-inc') as never);
      expect(result).toHaveLength(1);
      expect(result[0].customerCompanyId).toBe('beta-inc');
    });

    it('returns empty when no match', () => {
      const result = selectFilteredConfigs(stateWith('nonexistent') as never);
      expect(result).toHaveLength(0);
    });
  });
});
