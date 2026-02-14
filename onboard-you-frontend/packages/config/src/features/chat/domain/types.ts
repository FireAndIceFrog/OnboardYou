import type { ChatMessage } from '@/shared/domain/types';

export interface ChatState {
  messages: ChatMessage[];
  isTyping: boolean;
  error: string | null;
}
