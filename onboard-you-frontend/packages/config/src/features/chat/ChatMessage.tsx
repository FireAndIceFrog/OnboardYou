import type { ChatMessage as ChatMessageType } from '@/types';
import styles from './ChatWindow.module.scss';

interface ChatMessageProps {
  message: ChatMessageType;
}

/**
 * Simple markdown-like formatting:
 * - **bold** -> <strong>
 * - `inline code` -> <code>
 * - ```code blocks``` -> <pre><code>
 */
function formatContent(content: string): React.ReactNode[] {
  const parts: React.ReactNode[] = [];

  // Split on code blocks first
  const codeBlockRegex = /```([\s\S]*?)```/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  const segments: { type: 'text' | 'codeblock'; value: string }[] = [];

  while ((match = codeBlockRegex.exec(content)) !== null) {
    if (match.index > lastIndex) {
      segments.push({ type: 'text', value: content.slice(lastIndex, match.index) });
    }
    segments.push({ type: 'codeblock', value: match[1].trim() });
    lastIndex = match.index + match[0].length;
  }
  if (lastIndex < content.length) {
    segments.push({ type: 'text', value: content.slice(lastIndex) });
  }

  segments.forEach((segment, segIdx) => {
    if (segment.type === 'codeblock') {
      parts.push(
        <pre key={`cb-${segIdx}`} className={styles.codeBlock}>
          <code>{segment.value}</code>
        </pre>,
      );
    } else {
      // Process inline formatting
      const inlineRegex = /(\*\*(.+?)\*\*)|(`(.+?)`)/g;
      let inlineLastIndex = 0;
      let inlineMatch: RegExpExecArray | null;
      const inlineParts: React.ReactNode[] = [];

      while ((inlineMatch = inlineRegex.exec(segment.value)) !== null) {
        if (inlineMatch.index > inlineLastIndex) {
          inlineParts.push(segment.value.slice(inlineLastIndex, inlineMatch.index));
        }
        if (inlineMatch[2]) {
          inlineParts.push(
            <strong key={`b-${segIdx}-${inlineMatch.index}`}>{inlineMatch[2]}</strong>,
          );
        } else if (inlineMatch[4]) {
          inlineParts.push(
            <code key={`c-${segIdx}-${inlineMatch.index}`} className={styles.inlineCode}>
              {inlineMatch[4]}
            </code>,
          );
        }
        inlineLastIndex = inlineMatch.index + inlineMatch[0].length;
      }
      if (inlineLastIndex < segment.value.length) {
        inlineParts.push(segment.value.slice(inlineLastIndex));
      }

      // Convert newlines to <br/>
      const withBreaks: React.ReactNode[] = [];
      inlineParts.forEach((part, pi) => {
        if (typeof part === 'string') {
          const lines = part.split('\n');
          lines.forEach((line, li) => {
            if (li > 0) withBreaks.push(<br key={`br-${segIdx}-${pi}-${li}`} />);
            if (line) withBreaks.push(line);
          });
        } else {
          withBreaks.push(part);
        }
      });

      parts.push(<span key={`t-${segIdx}`}>{withBreaks}</span>);
    }
  });

  return parts;
}

function formatTime(iso: string): string {
  return new Date(iso).toLocaleTimeString('en-US', {
    hour: 'numeric',
    minute: '2-digit',
  });
}

export function ChatMessageComponent({ message }: ChatMessageProps) {
  const isUser = message.role === 'user';

  return (
    <div className={`${styles.chatMessage} ${isUser ? styles.user : styles.assistant}`}>
      <div className={styles.messageAvatar}>{isUser ? '👤' : '🤖'}</div>
      <div className={styles.messageBubble}>
        <div className={styles.messageContent}>{formatContent(message.content)}</div>
        <span className={styles.messageTime}>{formatTime(message.timestamp)}</span>
      </div>
    </div>
  );
}
