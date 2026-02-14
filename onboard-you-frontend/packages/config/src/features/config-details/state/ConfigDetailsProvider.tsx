import { useReducer, useEffect, useMemo, type ReactNode } from 'react';
import { useGlobal } from '@/shared/hooks';
import { configDetailsReducer, configDetailsInitialState } from './configDetailsReducer';
import { ConfigDetailsContext } from './ConfigDetailsContext';
import { fetchConfig } from '../services/configDetailsService';
import { convertToFlow } from '../services/pipelineLayoutService';

interface ConfigDetailsProviderProps {
  children: ReactNode;
  customerCompanyId: string;
}

export function ConfigDetailsProvider({ children, customerCompanyId }: ConfigDetailsProviderProps) {
  const { apiClient, showNotification } = useGlobal();
  const [state, dispatch] = useReducer(configDetailsReducer, configDetailsInitialState);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      dispatch({ type: 'FETCH_START' });
      try {
        const config = await fetchConfig(apiClient, customerCompanyId);
        if (cancelled) return;

        const { nodes, edges } = convertToFlow(config.pipeline);
        dispatch({ type: 'FETCH_SUCCESS', payload: { config, nodes, edges } });
      } catch (err) {
        if (cancelled) return;
        const message =
          err instanceof Error ? err.message : 'Failed to load configuration';
        dispatch({ type: 'FETCH_ERROR', payload: message });
        showNotification(message, 'error');
      }
    }

    load();

    return () => {
      cancelled = true;
    };
  }, [apiClient, customerCompanyId, showNotification]);

  const value = useMemo(() => ({ state, dispatch }), [state]);

  return (
    <ConfigDetailsContext.Provider value={value}>
      {children}
    </ConfigDetailsContext.Provider>
  );
}
