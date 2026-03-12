import { configureStore } from '@reduxjs/toolkit';
import { type TypedUseSelectorHook, useDispatch, useSelector } from 'react-redux';
import configDetailsReducer from '../features/config-details/state/configDetailsSlice';
import configListReducer from '../features/config-list/state/configListSlice';
import type { NotificationType } from '../shared/domain/types';
import { getGlobalValue } from '../shared/hooks';

/* ── Thunk extra ───────────────────────────────────────────── */

export interface ThunkExtra {
  showNotification: (message: string, type: NotificationType) => void;
}

/**
 * Lazy proxy so that thunks always get the latest injected values
 * even if the store is created before setGlobalValue() is called.
 */
const thunkExtra: ThunkExtra = {
  get showNotification() {
    const g = getGlobalValue();
    if (!g) throw new Error('Global value not injected yet — cannot access showNotification');
    return g.showNotification;
  },
};

export const store = configureStore({
  reducer: {
    configDetails: configDetailsReducer,
    configList: configListReducer,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      thunk: { extraArgument: thunkExtra },
    }),
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
export const useAppDispatch: () => AppDispatch = useDispatch;
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;
