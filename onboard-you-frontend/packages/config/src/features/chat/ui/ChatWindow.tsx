import { useCallback, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useAppDispatch, useAppSelector } from '@/store';
import { sendMessage, clearChat, selectChatMessages, selectIsTyping } from '../state/chatSlice';
import { selectConfig } from '@/features/config-details/state/configDetailsSlice';
import { ChatMessageComponent } from './ChatMessage';
import { ChatInput } from './ChatInput';
import styles from './ChatWindow.module.scss';

interface ChatWindowProps {
  onClose: () => void;
}

const SUGGESTION_KEYS = [
  'chat.suggestions.cleanAddress',
  'chat.suggestions.formatPhone',
  'chat.suggestions.removeDuplicates',
  'chat.suggestions.maskPii',
  'chat.suggestions.standardiseCountry',
  'chat.suggestions.whatCanYouDo',
] as const;

export function ChatWindow({ onClose }: ChatWindowProps) {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const messages = useAppSelector(selectChatMessages);
  const isTyping = useAppSelector(selectIsTyping);
  const pipelineConfig = useAppSelector(selectConfig);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const handleSend = useCallback(
    (content: string) => {
      if (!pipelineConfig) return;
      dispatch(sendMessage({ content, pipelineConfig }));
    },
    [dispatch, pipelineConfig],
  );

  const handleClear = useCallback(() => {
    dispatch(clearChat());
  }, [dispatch]);

  // Auto-scroll to bottom on new messages or typing state change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isTyping]);

  const hasMessages = messages.length > 0;

  return (
    <aside className={styles.chatWindow} aria-label={t('chat.title')}>
      {/* Header */}
      <div className={styles.chatHeader}>
        <div className={styles.chatHeaderLeft}>
          <span className={styles.chatHeaderIcon}>🤖</span>
          <div>
            <h3 className={styles.chatTitle}>{t('chat.title')}</h3>
            <span className={styles.chatSubtitle}>
              {pipelineConfig ? pipelineConfig.name : t('chat.noSystemSelected')}
            </span>
          </div>
        </div>
        <div className={styles.chatHeaderActions}>
          {hasMessages && (
            <button
              type="button"
              className={styles.iconBtn}
              onClick={handleClear}
              aria-label={t('chat.clearChat')}
              title={t('chat.clearChat')}
            >
              🗑
            </button>
          )}
          <button
            type="button"
            className={styles.iconBtn}
            onClick={onClose}
            aria-label={t('chat.closeChat')}
            title={t('chat.closeChat')}
          >
            ✕
          </button>
        </div>
      </div>

      {/* Messages */}
      <div className={styles.chatMessages} role="log" aria-live="polite">
        {!hasMessages ? (
          <div className={styles.chatWelcome}>
            <span className={styles.welcomeIcon}>💬</span>
            <h4 className={styles.welcomeTitle}>{t('chat.welcome.title')}</h4>
            <p className={styles.welcomeText}>
              {t('chat.welcome.text')}
            </p>
            <div className={styles.suggestions} role="group" aria-label="Suggested prompts">
              {SUGGESTION_KEYS.map((key) => (
                <button
                  key={key}
                  type="button"
                  className={styles.suggestionChip}
                  onClick={() => handleSend(t(key))}
                >
                  {t(key)}
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
      <ChatInput onSend={handleSend} disabled={isTyping || !pipelineConfig} />
    </aside>
  );
}
