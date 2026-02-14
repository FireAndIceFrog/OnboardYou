import { createContext } from 'react';
import type { GlobalState, GlobalAction } from './globalReducer';

export interface GlobalContextValue {
  state: GlobalState;
  dispatch: React.Dispatch<GlobalAction>;
}

export const GlobalContext = createContext<GlobalContextValue | null>(null);
