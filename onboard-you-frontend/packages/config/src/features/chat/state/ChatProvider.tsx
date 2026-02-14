import { useCallback, useRef, useState, type ReactNode } from 'react';
import { useReducer } from 'react';
import { ChatContext } from './ChatContext';
import { chatReducer, initialChatState } from './chatReducer';
import { generateResponse, deriveFlowAction } from '../services/chatService';
import type { PipelineConfig, ChatMessage, ActionConfig } from '@/shared/domain/types';

interface ChatProviderProps {
  children: ReactNode;
  pipelineConfig: PipelineConfig | null;
}

function createMessage(role: 'user' | 'assistant', content: string): ChatMessage {
  return {
    id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    role,
    content,
    timestamp: new Date().toISOString(),
  };
}

export function ChatProvider({ children, pipelineConfig }: ChatProviderProps) {
  const [state, dispatch] = useReducer(chatReducer, initialChatState);
  const typingRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [lastFlowAction, setLastFlowAction] = useState<ActionConfig | null>(null);

  const sendMessage = useCallback(
    (content: string) => {
      if (!content.trim() || !pipelineConfig) return;

      const userMsg = createMessage('user', content.trim());
      dispatch({ type: 'ADD_USER_MESSAGE', payload: userMsg });
      dispatch({ type: 'SET_TYPING', payload: true });

      // Check if this message implies a flow change
      const flowAction = deriveFlowAction(content);
      if (flowAction) {
        setLastFlowAction(flowAction);
      }

      // Simulate AI response with delay
      const delay = 400 + Math.random() * 800;
      typingRef.current = setTimeout(() => {
        const response = generateResponse(pipelineConfig, content);
        const assistantMsg = createMessage('assistant', response);
        dispatch({ type: 'ADD_ASSISTANT_MESSAGE', payload: assistantMsg });
      }, delay);
    },
    [pipelineConfig],
  );

  const clearChat = useCallback(() => {
    if (typingRef.current) {
      clearTimeout(typingRef.current);
      typingRef.current = null;
    }
    dispatch({ type: 'CLEAR_CHAT' });
    setLastFlowAction(null);
  }, []);

  return (
    <ChatContext.Provider value={{ state, sendMessage, clearChat, pipelineConfig, lastFlowAction }}>
      {children}
    </ChatContext.Provider>
  );
}
