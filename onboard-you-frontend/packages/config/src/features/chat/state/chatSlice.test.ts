import { describe, it, expect } from 'vitest';
import reducer, {
  addUserMessage,
  clearChat,
  setTyping,
  setError,
  setLastFlowAction,
} from './chatSlice';
import type { ChatSliceState } from './chatSlice';
import type { ChatMessage } from '@/shared/domain/types';

const initialState: ChatSliceState = {
  messages: [],
  isTyping: false,
  error: null,
  lastFlowAction: null,
};

const mockMessage: ChatMessage = {
  id: 'msg-001',
  role: 'user',
  content: 'Hello',
  timestamp: '2026-02-15T00:00:00Z',
};

describe('chatSlice', () => {
  it('should return the initial state', () => {
    const state = reducer(undefined, { type: '@@INIT' });
    expect(state).toEqual(initialState);
  });

  it('addUserMessage adds a message to the array', () => {
    const state = reducer(initialState, addUserMessage(mockMessage));
    expect(state.messages).toHaveLength(1);
    expect(state.messages[0]).toEqual(mockMessage);
  });

  it('addUserMessage appends to existing messages', () => {
    const withOne = reducer(initialState, addUserMessage(mockMessage));
    const secondMsg: ChatMessage = { ...mockMessage, id: 'msg-002', content: 'World' };
    const state = reducer(withOne, addUserMessage(secondMsg));
    expect(state.messages).toHaveLength(2);
  });

  it('clearChat resets messages, typing, error, and lastFlowAction', () => {
    const dirtyState: ChatSliceState = {
      messages: [mockMessage],
      isTyping: true,
      error: 'some error',
      lastFlowAction: { id: 'step-1', action_type: 'pii_masking', config: { columns: [] } },
    };
    const state = reducer(dirtyState, clearChat());
    expect(state.messages).toEqual([]);
    expect(state.isTyping).toBe(false);
    expect(state.error).toBeNull();
    expect(state.lastFlowAction).toBeNull();
  });

  it('setTyping changes isTyping', () => {
    const state = reducer(initialState, setTyping(true));
    expect(state.isTyping).toBe(true);

    const state2 = reducer(state, setTyping(false));
    expect(state2.isTyping).toBe(false);
  });

  it('setError sets error and clears isTyping', () => {
    const typingState = { ...initialState, isTyping: true };
    const state = reducer(typingState, setError('Connection lost'));
    expect(state.error).toBe('Connection lost');
    expect(state.isTyping).toBe(false);
  });

  it('setLastFlowAction stores the action', () => {
    const action = { id: 'step-1', action_type: 'regex_replace' as const, config: { column: 'address', pattern: '\\s+', replacement: ' ' } };
    const state = reducer(initialState, setLastFlowAction(action));
    expect(state.lastFlowAction).toEqual(action);
  });
});
