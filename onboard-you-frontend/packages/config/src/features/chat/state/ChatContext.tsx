import { createContext } from 'react';
import type { ChatState } from '../domain/types';
import type { PipelineConfig } from '@/shared/domain/types';

export interface ChatContextValue {
  state: ChatState;
  sendMessage: (content: string) => void;
  clearChat: () => void;
  pipelineConfig: PipelineConfig | null;
}

export const ChatContext = createContext<ChatContextValue | null>(null);
