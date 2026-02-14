import { useEffect, useRef } from 'react';
import { useChat } from './useChat';
import { ChatMessageComponent } from './ChatMessage';
import { ChatInput } from './ChatInput';
import type { PipelineConfig } from '@/types';
import styles from './ChatWindow.module.scss';

interface ChatWindowProps {
  config: PipelineConfig;
  onClose: () => void;
}

export function ChatWindow({ config, onClose }: ChatWindowProps) {
  const { messages, isTyping, sendMessage, clearChat } = useChat(config);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isTyping]);

  return (
    <div className={styles.chatWindow}>
      <div className={styles.chatHeader}>
        <div className={styles.chatHeaderLeft}>
          <span className={styles.chatHeaderIcon}>🤖</span>
          <div>
            <h2 className={styles.chatTitle}>Pipeline Assistant</h2>
            <span className={styles.chatSubtitle}>AI-powered help</span>
          </div>
        </div>
        <div className={styles.chatHeaderActions}>
          {messages.length > 0 && (
            <button className={styles.clearChatBtn} onClick={clearChat} aria-label="Clear chat">
              🗑️
            </button>
          )}
          <button className={styles.closeChatBtn} onClick={onClose} aria-label="Close chat">
            ×
          </button>
        </div>
      </div>

      <div className={styles.chatMessages}>
        {messages.length === 0 && (
          <div className={styles.chatWelcome}>
            <span className={styles.welcomeIcon}>💬</span>
            <h3 className={styles.welcomeTitle}>Hi! I'm your Pipeline Assistant</h3>
            <p className={styles.welcomeText}>
              Ask me anything about the <strong>{config.name}</strong> pipeline. I can explain
              stages, suggest improvements, or help troubleshoot issues.
            </p>
            <div className={styles.suggestions}>
              <button
                className={styles.suggestionChip}
                onClick={() => sendMessage('Explain this pipeline')}
              >
                Explain this pipeline
              </button>
              <button
                className={styles.suggestionChip}
                onClick={() => sendMessage('What transformations are applied?')}
              >
                List transformations
              </button>
              <button
                className={styles.suggestionChip}
                onClick={() => sendMessage('What is the status of this pipeline?')}
              >
                Check status
              </button>
            </div>
          </div>
        )}

        {messages.map((msg) => (
          <ChatMessageComponent key={msg.id} message={msg} />
        ))}

        {isTyping && (
          <div className={`${styles.chatMessage} ${styles.assistant}`}>
            <div className={styles.messageAvatar}>🤖</div>
            <div className={styles.messageBubble}>
              <div className={styles.typingIndicator}>
                <span />
                <span />
                <span />
              </div>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      <ChatInput onSend={sendMessage} disabled={isTyping} />
    </div>
  );
}
