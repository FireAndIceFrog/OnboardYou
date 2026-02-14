import { createContext, useContext } from 'react';
import type { Dispatch } from 'react';
import type { ConfigDetailsState } from '../domain/types';
import type { ConfigDetailsAction } from './configDetailsReducer';

export interface ConfigDetailsContextValue {
  state: ConfigDetailsState;
  dispatch: Dispatch<ConfigDetailsAction>;
}

export const ConfigDetailsContext = createContext<ConfigDetailsContextValue | null>(null);

export function useConfigDetails(): ConfigDetailsContextValue {
  const ctx = useContext(ConfigDetailsContext);
  if (!ctx) {
    throw new Error('useConfigDetails must be used within a ConfigDetailsProvider');
  }
  return ctx;
}
