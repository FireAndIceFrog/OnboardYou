import { useContext, useEffect, useRef } from 'react';
import { ChatContext } from '../state/ChatContext';
import { ChatMessageComponent } from './ChatMessage';
import { ChatInput } from './ChatInput';
import styles from './ChatWindow.module.scss';

interface ChatWindowProps {
  onClose: () => void;
}

const SUGGESTION_CHIPS = [
  'Clean up my address data',
  'Format all phone numbers to international',
  'Remove duplicate employee records',
  'Mask sensitive personal information',
  'Standardise country codes',
  'What can you help me with?',
];

export function ChatWindow({ onClose }: ChatWindowProps) {
  const ctx = useContext(ChatContext);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  if (!ctx) {
    throw new Error('ChatWindow must be used within a ChatProvider');
  }

  const { state, sendMessage, clearChat, pipelineConfig } = ctx;
  const { messages, isTyping } = state;

  // Auto-scroll to bottom on new messages or typing state change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isTyping]);

  const hasMessages = messages.length > 0;

  return (
    <div className={styles.chatWindow}>
      {/* Header */}
      <div className={styles.chatHeader}>
        <div className={styles.chatHeaderLeft}>
          <span className={styles.chatHeaderIcon}>🤖</span>
          <div>
            <h3 className={styles.chatTitle}>Onboarding Assistant</h3>
            <span className={styles.chatSubtitle}>
              {pipelineConfig ? pipelineConfig.name : 'No system selected'}
            </span>
          </div>
        </div>
        <div className={styles.chatHeaderActions}>
          {hasMessages && (
            <button
              type="button"
              className={styles.iconBtn}
              onClick={clearChat}
              aria-label="Clear chat"
              title="Clear chat"
            >
              🗑
            </button>
          )}
          <button
            type="button"
            className={styles.iconBtn}
            onClick={onClose}
            aria-label="Close chat"
            title="Close chat"
          >
            ✕
          </button>
        </div>
      </div>

      {/* Messages */}
      <div className={styles.chatMessages}>
        {!hasMessages ? (
          <div className={styles.chatWelcome}>
            <span className={styles.welcomeIcon}>💬</span>
            <h4 className={styles.welcomeTitle}>How can I help?</h4>
            <p className={styles.welcomeText}>
              Tell me what you'd like to do with your data — I'll update the flow for you
              automatically.
            </p>
            <div className={styles.suggestions}>
              {SUGGESTION_CHIPS.map((chip) => (
                <button
                  key={chip}
                  type="button"
                  className={styles.suggestionChip}
                  onClick={() => sendMessage(chip)}
                >
                  {chip}
                </button>
              ))}
            </div>
          </div>
        ) : (
          messages.map((msg) => <ChatMessageComponent key={msg.id} message={msg} />)
        )}

        {/* Typing indicator */}
        {isTyping && (
          <div className={styles.chatMessage}>
            <div className={styles.messageAvatar}>🤖</div>
            <div className={`${styles.messageBubble} ${styles.messageBubbleAssistant}`}>
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

      {/* Input */}
      <ChatInput onSend={sendMessage} disabled={isTyping || !pipelineConfig} />
    </div>
  );
}
