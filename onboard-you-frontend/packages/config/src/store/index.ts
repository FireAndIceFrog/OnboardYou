import { configureStore } from '@reduxjs/toolkit';
import { type TypedUseSelectorHook, useDispatch, useSelector } from 'react-redux';
import chatReducer from '../features/chat/state/chatSlice';
import configDetailsReducer from '../features/config-details/state/configDetailsSlice';
import configListReducer from '../features/config-list/state/configListSlice';

export const store = configureStore({
  reducer: {
    chat: chatReducer,
    configDetails: configDetailsReducer,
    configList: configListReducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
export const useAppDispatch: () => AppDispatch = useDispatch;
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;
