import { useCallback, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, Heading, Text, chakra } from '@chakra-ui/react';
import { useAppDispatch, useAppSelector } from '@/store';
import { sendMessage, clearChat, selectChatMessages, selectIsTyping } from '../../state/chatSlice';
import { selectConfig } from '@/features/config-details/state/configDetailsSlice';
import { ChatMessageComponent } from './ChatMessage';
import { ChatInput } from './ChatInput';

const StyledButton = chakra('button');

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

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isTyping]);

  const hasMessages = messages.length > 0;

  return (
    <Flex as="aside" direction="column" h="100%" aria-label={t('chat.title')}>
      {/* Header */}
      <Flex align="center" justify="space-between" px="4" py="3" borderBottom="1px solid" borderColor="gray.200">
        <Flex align="center" gap="3">
          <Text fontSize="xl">🤖</Text>
          <Box>
            <Heading size="sm">{t('chat.title')}</Heading>
            <Text fontSize="xs" color="gray.500">
              {pipelineConfig ? pipelineConfig.name : t('chat.noSystemSelected')}
            </Text>
          </Box>
        </Flex>
        <Flex gap="1">
          {hasMessages && (
            <StyledButton
              type="button"
              p="1.5"
              borderRadius="md"
              bg="transparent"
              cursor="pointer"
              _hover={{ bg: 'gray.100' }}
              onClick={handleClear}
              aria-label={t('chat.clearChat')}
              title={t('chat.clearChat')}
            >
              🗑
            </StyledButton>
          )}
          <StyledButton
            type="button"
            p="1.5"
            borderRadius="md"
            bg="transparent"
            cursor="pointer"
            _hover={{ bg: 'gray.100' }}
            onClick={onClose}
            aria-label={t('chat.closeChat')}
            title={t('chat.closeChat')}
          >
            ✕
          </StyledButton>
        </Flex>
      </Flex>

      {/* Messages */}
      <Box flex="1" overflowY="auto" px="4" py="4" role="log" aria-live="polite">
        {!hasMessages ? (
          <Flex direction="column" align="center" justify="center" h="100%" textAlign="center" gap="3" px="4">
            <Text fontSize="3xl">💬</Text>
            <Heading size="sm">{t('chat.welcome.title')}</Heading>
            <Text fontSize="sm" color="gray.500">{t('chat.welcome.text')}</Text>
            <Flex wrap="wrap" gap="2" justify="center" mt="2" role="group" aria-label="Suggested prompts">
              {SUGGESTION_KEYS.map((key) => (
                <StyledButton
                  key={key}
                  type="button"
                  px="3"
                  py="1.5"
                  borderRadius="full"
                  border="1px solid"
                  borderColor="gray.200"
                  bg="white"
                  fontSize="xs"
                  color="gray.600"
                  cursor="pointer"
                  transition="all 0.15s"
                  _hover={{ borderColor: 'blue.300', bg: 'blue.50', color: 'blue.600' }}
                  onClick={() => handleSend(t(key))}
                >
                  {t(key)}
                </StyledButton>
              ))}
            </Flex>
          </Flex>
        ) : (
          messages.map((msg) => <ChatMessageComponent key={msg.id} message={msg} />)
        )}

        {/* Typing indicator */}
        {isTyping && (
          <Flex gap="3" mb="3">
            <Text fontSize="lg">🤖</Text>
            <Box bg="gray.100" borderRadius="lg" borderTopLeftRadius="sm" px="4" py="3">
              <Flex gap="1" align="center">
                <Box w="2" h="2" borderRadius="full" bg="gray.400" css={{ animation: 'pulse 1.4s ease-in-out infinite', animationDelay: '0s' }} />
                <Box w="2" h="2" borderRadius="full" bg="gray.400" css={{ animation: 'pulse 1.4s ease-in-out infinite', animationDelay: '0.2s' }} />
                <Box w="2" h="2" borderRadius="full" bg="gray.400" css={{ animation: 'pulse 1.4s ease-in-out infinite', animationDelay: '0.4s' }} />
              </Flex>
            </Box>
          </Flex>
        )}

        <div ref={messagesEndRef} />
      </Box>

      {/* Input */}
      <ChatInput onSend={handleSend} disabled={isTyping || !pipelineConfig} />
    </Flex>
  );
}
