import { createSlice, createAsyncThunk, type PayloadAction } from '@reduxjs/toolkit';
import type { RootState } from '@/store';
import type { ActionConfig, PipelineConfig } from '@/generated/api';
import type { ChatMessage } from '@/shared/domain/types';
import type { ChatState } from '../domain/types';
import { generateResponse, deriveFlowAction } from '../services/chatService';

/* ── Extended slice state ──────────────────────────────────── */

export interface ChatSliceState extends ChatState {
  lastFlowAction: ActionConfig | null;
}

const initialState: ChatSliceState = {
  messages: [],
  isTyping: false,
  error: null,
  lastFlowAction: null,
};

/* ── Helpers ───────────────────────────────────────────────── */

function createMessage(role: 'user' | 'assistant', content: string): ChatMessage {
  return {
    id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    role,
    content,
    timestamp: new Date().toISOString(),
  };
}

/* ── Async thunk ───────────────────────────────────────────── */

export const sendMessage = createAsyncThunk<
  void,
  { content: string; pipelineConfig: PipelineConfig }
>('chat/sendMessage', async ({ content, pipelineConfig }, { dispatch, getState }) => {
  const trimmed = content.trim();
  if (!trimmed) return;

  const userMsg = createMessage('user', trimmed);
  dispatch(addUserMessage(userMsg));
  dispatch(setTyping(true));

  // Check if this message implies a flow change
  const flowAction = deriveFlowAction(trimmed);
  if (flowAction) {
    dispatch(setLastFlowAction(flowAction));
  }

  // Simulate AI response with delay
  const delay = 400 + Math.random() * 800;
  await new Promise((resolve) => setTimeout(resolve, delay));

  // If chat was cleared during the delay, bail out
  const { chat } = getState() as RootState;
  if (!chat.messages.some((m) => m.id === userMsg.id)) return;

  const response = generateResponse(pipelineConfig, trimmed);
  const assistantMsg = createMessage('assistant', response);
  dispatch(addAssistantMessage(assistantMsg));
});

/* ── Slice ─────────────────────────────────────────────────── */

const chatSlice = createSlice({
  name: 'chat',
  initialState,
  reducers: {
    addUserMessage(state, action: PayloadAction<ChatMessage>) {
      state.messages.push(action.payload);
    },
    addAssistantMessage(state, action: PayloadAction<ChatMessage>) {
      state.messages.push(action.payload);
      state.isTyping = false;
    },
    setTyping(state, action: PayloadAction<boolean>) {
      state.isTyping = action.payload;
    },
    setError(state, action: PayloadAction<string | null>) {
      state.error = action.payload;
      state.isTyping = false;
    },
    setLastFlowAction(state, action: PayloadAction<ActionConfig | null>) {
      state.lastFlowAction = action.payload;
    },
    clearChat(state) {
      state.messages = [];
      state.isTyping = false;
      state.error = null;
      state.lastFlowAction = null;
    },
  },
});

export const {
  addUserMessage,
  addAssistantMessage,
  setTyping,
  setError,
  setLastFlowAction,
  clearChat,
} = chatSlice.actions;

/* ── Selectors ─────────────────────────────────────────────── */

export const selectChatMessages = (state: RootState) => state.chat.messages;
export const selectIsTyping = (state: RootState) => state.chat.isTyping;
export const selectLastFlowAction = (state: RootState) => state.chat.lastFlowAction;
export const selectChatError = (state: RootState) => state.chat.error;

export default chatSlice.reducer;
