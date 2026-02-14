import { createContext } from 'react';
import type { ChatState } from '../domain/types';
import type { PipelineConfig, ActionConfig } from '@/shared/domain/types';

export interface ChatContextValue {
  state: ChatState;
  sendMessage: (content: string) => void;
  clearChat: () => void;
  pipelineConfig: PipelineConfig | null;
  /** The last action that was appended to the flow via chat, if any. */
  lastFlowAction: ActionConfig | null;
}

export const ChatContext = createContext<ChatContextValue | null>(null);
