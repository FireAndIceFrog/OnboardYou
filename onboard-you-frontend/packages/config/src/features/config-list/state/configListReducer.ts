import type { PipelineConfig } from '@/shared/domain/types';
import type { ConfigListState } from '../domain/types';

export type ConfigListAction =
  | { type: 'FETCH_START' }
  | { type: 'FETCH_SUCCESS'; payload: PipelineConfig[] }
  | { type: 'FETCH_ERROR'; payload: string }
  | { type: 'SET_SEARCH_QUERY'; payload: string };

export const configListInitialState: ConfigListState = {
  configs: [],
  isLoading: false,
  error: null,
  searchQuery: '',
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

    default:
      return state;
  }
}
