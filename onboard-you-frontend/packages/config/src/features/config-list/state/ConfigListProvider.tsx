import { useReducer, useCallback, useMemo, useEffect, type ReactNode } from 'react';
import { useGlobal } from '@/shared/hooks';
import { configListReducer, configListInitialState } from './configListReducer';
import { ConfigListContext } from './ConfigListContext';
import { fetchConfigs as fetchConfigsService } from '../services';

interface ConfigListProviderProps {
  children: ReactNode;
}

export function ConfigListProvider({ children }: ConfigListProviderProps) {
  const { apiClient, showNotification } = useGlobal();
  const [state, dispatch] = useReducer(configListReducer, configListInitialState);

  const fetchConfigs = useCallback(async () => {
    dispatch({ type: 'FETCH_START' });
    try {
      const configs = await fetchConfigsService(apiClient);
      dispatch({ type: 'FETCH_SUCCESS', payload: configs });
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to fetch configurations';
      dispatch({ type: 'FETCH_ERROR', payload: message });
      showNotification(message, 'error');
    }
  }, [apiClient, showNotification]);

  const setSearchQuery = useCallback((query: string) => {
    dispatch({ type: 'SET_SEARCH_QUERY', payload: query });
  }, []);

  const filteredConfigs = useMemo(() => {
    let result = state.configs;

    if (state.searchQuery) {
      const q = state.searchQuery.toLowerCase();
      result = result.filter(
        (c) =>
          c.name.toLowerCase().includes(q) ||
          c.customerCompanyId.toLowerCase().includes(q),
      );
    }

    return result;
  }, [state.configs, state.searchQuery]);

  useEffect(() => {
    fetchConfigs();
  }, [fetchConfigs]);

  const value = useMemo(
    () => ({
      state,
      dispatch,
      filteredConfigs,
      fetchConfigs,
      setSearchQuery,
    }),
    [state, filteredConfigs, fetchConfigs, setSearchQuery],
  );

  return <ConfigListContext.Provider value={value}>{children}</ConfigListContext.Provider>;
}
