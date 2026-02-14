import type { ReactElement } from 'react';
import type { ChatMessage } from '@/shared/domain/types';
import styles from './ChatWindow.module.scss';

interface ChatMessageProps {
  message: ChatMessage;
}

function formatTimestamp(iso: string): string {
  const date = new Date(iso);
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

/**
 * Renders message content with simple markdown-like formatting:
 * **bold**, `inline code`, and newlines as <br />.
 */
function renderContent(content: string): (string | ReactElement)[] {
  const parts: (string | ReactElement)[] = [];
  // Split by bold and code patterns
  const regex = /(\*\*(.+?)\*\*|`(.+?)`)/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;
  let key = 0;

  while ((match = regex.exec(content)) !== null) {
    // Push text before the match
    if (match.index > lastIndex) {
      parts.push(...splitNewlines(content.slice(lastIndex, match.index), key));
      key += 10;
    }

    if (match[2]) {
      // Bold text
      parts.push(
        <strong key={`b-${key++}`} className={styles.boldText}>
          {match[2]}
        </strong>,
      );
    } else if (match[3]) {
      // Inline code
      parts.push(
        <code key={`c-${key++}`} className={styles.inlineCode}>
          {match[3]}
        </code>,
      );
    }

    lastIndex = match.index + match[0].length;
  }

  // Remaining text
  if (lastIndex < content.length) {
    parts.push(...splitNewlines(content.slice(lastIndex), key));
  }

  return parts;
}

function splitNewlines(text: string, startKey: number): (string | ReactElement)[] {
  const segments = text.split('\n');
  const result: (string | ReactElement)[] = [];

  segments.forEach((segment, i) => {
    if (i > 0) {
      result.push(<br key={`nl-${startKey}-${i}`} />);
    }
    if (segment) {
      result.push(segment);
    }
  });

  return result;
}

export function ChatMessageComponent({ message }: ChatMessageProps) {
  const isUser = message.role === 'user';

  return (
    <div
      className={`${styles.chatMessage} ${isUser ? styles.chatMessageUser : ''}`}
    >
      <div className={styles.messageAvatar}>
        {isUser ? '👤' : '🤖'}
      </div>
      <div>
        <div
          className={`${styles.messageBubble} ${
            isUser ? styles.messageBubbleUser : styles.messageBubbleAssistant
          }`}
        >
          {renderContent(message.content)}
        </div>
        <div
          className={`${styles.messageTime} ${isUser ? styles.messageTimeUser : ''}`}
        >
          {formatTimestamp(message.timestamp)}
        </div>
      </div>
    </div>
  );
}
