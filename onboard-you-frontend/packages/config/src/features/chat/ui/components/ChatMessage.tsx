import type { ReactElement } from 'react';
import { Box, Flex, Text, Code } from '@chakra-ui/react';
import type { ChatMessage } from '@/shared/domain/types';

interface ChatMessageProps {
  message: ChatMessage;
}

export function formatTimestamp(iso: string): string {
  const date = new Date(iso);
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

/**
 * Renders message content with simple markdown-like formatting:
 * **bold**, `inline code`, and newlines as <br />.
 */
export function renderContent(content: string): (string | ReactElement)[] {
  const parts: (string | ReactElement)[] = [];
  const regex = /(\*\*(.+?)\*\*|`(.+?)`)/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;
  let key = 0;

  while ((match = regex.exec(content)) !== null) {
    if (match.index > lastIndex) {
      parts.push(...splitNewlines(content.slice(lastIndex, match.index), key));
      key += 10;
    }

    if (match[2]) {
      parts.push(
        <Text as="strong" key={`b-${key++}`} fontWeight="600">
          {match[2]}
        </Text>,
      );
    } else if (match[3]) {
      parts.push(
        <Code key={`c-${key++}`} fontSize="xs" px="1" py="0.5" borderRadius="sm">
          {match[3]}
        </Code>,
      );
    }

    lastIndex = match.index + match[0].length;
  }

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
    <Flex gap="3" mb="3" direction={isUser ? 'row-reverse' : 'row'}>
      <Text fontSize="lg" flexShrink={0}>{isUser ? '👤' : '🤖'}</Text>
      <Box>
        <Box
          px="3"
          py="2"
          borderRadius="lg"
          borderTopRightRadius={isUser ? 'sm' : undefined}
          borderTopLeftRadius={!isUser ? 'sm' : undefined}
          bg={isUser ? 'blue.500' : 'gray.100'}
          color={isUser ? 'white' : 'gray.800'}
          fontSize="sm"
          lineHeight="tall"
        >
          {renderContent(message.content)}
        </Box>
        <Text
          fontSize="xs"
          color="gray.400"
          mt="1"
          textAlign={isUser ? 'right' : 'left'}
        >
          {formatTimestamp(message.timestamp)}
        </Text>
      </Box>
    </Flex>
  );
}
