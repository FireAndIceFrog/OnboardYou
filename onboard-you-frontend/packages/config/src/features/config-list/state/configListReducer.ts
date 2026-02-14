import type { PipelineConfig } from '@/shared/domain/types';
import type { ConfigListState } from '../domain/types';

export type ConfigListAction =
  | { type: 'FETCH_START' }
  | { type: 'FETCH_SUCCESS'; payload: PipelineConfig[] }
  | { type: 'FETCH_ERROR'; payload: string }
  | { type: 'SET_SEARCH_QUERY'; payload: string }
  | { type: 'SET_STATUS_FILTER'; payload: string | null }
  | { type: 'DELETE_CONFIG'; payload: string };

export const configListInitialState: ConfigListState = {
  configs: [],
  isLoading: false,
  error: null,
  searchQuery: '',
  statusFilter: null,
};

export function configListReducer(
  state: ConfigListState,
  action: ConfigListAction,
): ConfigListState {
  switch (action.type) {
    case 'FETCH_START':
      return { ...state, isLoading: true, error: null };

    case 'FETCH_SUCCESS':
      return { ...state, isLoading: false, configs: action.payload, error: null };

    case 'FETCH_ERROR':
      return { ...state, isLoading: false, error: action.payload };

    case 'SET_SEARCH_QUERY':
      return { ...state, searchQuery: action.payload };

    case 'SET_STATUS_FILTER':
      return { ...state, statusFilter: action.payload };

    case 'DELETE_CONFIG':
      return {
        ...state,
        configs: state.configs.filter((c) => c.id !== action.payload),
      };

    default:
      return state;
  }
}
