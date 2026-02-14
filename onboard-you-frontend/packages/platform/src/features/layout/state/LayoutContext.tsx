import { createContext } from 'react';
import type { LayoutState } from '@/features/layout/domain/types';
import type { LayoutAction } from './layoutReducer';

export interface LayoutContextValue {
  state: LayoutState;
  dispatch: React.Dispatch<LayoutAction>;
}

export const LayoutContext = createContext<LayoutContextValue | null>(null);
