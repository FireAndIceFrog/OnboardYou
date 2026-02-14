import type { ChatMessage } from '@/shared/domain/types';
import type { ChatState } from '../domain/types';

export type ChatAction =
  | { type: 'ADD_USER_MESSAGE'; payload: ChatMessage }
  | { type: 'ADD_ASSISTANT_MESSAGE'; payload: ChatMessage }
  | { type: 'SET_TYPING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'CLEAR_CHAT' };

export const initialChatState: ChatState = {
  messages: [],
  isTyping: false,
  error: null,
};

export function chatReducer(state: ChatState, action: ChatAction): ChatState {
  switch (action.type) {
    case 'ADD_USER_MESSAGE':
      return { ...state, messages: [...state.messages, action.payload] };
    case 'ADD_ASSISTANT_MESSAGE':
      return { ...state, messages: [...state.messages, action.payload], isTyping: false };
    case 'SET_TYPING':
      return { ...state, isTyping: action.payload };
    case 'SET_ERROR':
      return { ...state, error: action.payload, isTyping: false };
    case 'CLEAR_CHAT':
      return { ...initialChatState };
    default:
      return state;
  }
}
