import { createContext, useContext } from 'react';
import type { Dispatch } from 'react';
import type { PipelineConfig } from '@/shared/domain/types';
import type { ConfigListState } from '../domain/types';
import type { ConfigListAction } from './configListReducer';

export interface ConfigListContextValue {
  state: ConfigListState;
  dispatch: Dispatch<ConfigListAction>;
  filteredConfigs: PipelineConfig[];
  fetchConfigs: () => Promise<void>;
  deleteConfig: (id: string) => Promise<void>;
  setSearchQuery: (query: string) => void;
  setStatusFilter: (status: string | null) => void;
}

export const ConfigListContext = createContext<ConfigListContextValue | null>(null);

export function useConfigList(): ConfigListContextValue {
  const ctx = useContext(ConfigListContext);
  if (!ctx) {
    throw new Error('useConfigList must be used within a ConfigListProvider');
  }
  return ctx;
}
