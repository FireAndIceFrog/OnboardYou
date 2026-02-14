import { useReducer, useEffect, type ReactNode } from 'react';
import { GlobalContext } from './GlobalContext';
import { globalReducer, initialGlobalState } from './globalReducer';

const NOTIFICATION_DISMISS_MS = 5_000;

interface GlobalProviderProps {
  children: ReactNode;
}

export function GlobalProvider({ children }: GlobalProviderProps) {
  const [state, dispatch] = useReducer(globalReducer, initialGlobalState);

  // Auto-dismiss notifications after 5 seconds
  useEffect(() => {
    if (state.notifications.length === 0) return;

    const latest = state.notifications[state.notifications.length - 1];
    const timer = setTimeout(() => {
      dispatch({ type: 'DISMISS_NOTIFICATION', payload: latest.id });
    }, NOTIFICATION_DISMISS_MS);

    return () => clearTimeout(timer);
  }, [state.notifications]);

  return (
    <GlobalContext.Provider value={{ state, dispatch }}>
      {children}
    </GlobalContext.Provider>
  );
}
