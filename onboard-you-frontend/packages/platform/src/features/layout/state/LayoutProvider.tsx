import { useReducer, type ReactNode } from 'react';
import { LayoutContext } from './LayoutContext';
import { layoutReducer, initialLayoutState } from './layoutReducer';

interface LayoutProviderProps {
  children: ReactNode;
}

export function LayoutProvider({ children }: LayoutProviderProps) {
  const [state, dispatch] = useReducer(layoutReducer, initialLayoutState);

  return (
    <LayoutContext.Provider value={{ state, dispatch }}>
      {children}
    </LayoutContext.Provider>
  );
}
