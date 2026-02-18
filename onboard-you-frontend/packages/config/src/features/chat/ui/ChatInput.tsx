import { useState, useRef, useCallback, type KeyboardEvent, type ChangeEvent } from 'react';
import { useTranslation } from 'react-i18next';
import { Box, Flex, chakra } from '@chakra-ui/react';

const StyledTextarea = chakra('textarea');
const StyledButton = chakra('button');

interface ChatInputProps {
  onSend: (content: string) => void;
  disabled?: boolean;
}

export function ChatInput({ onSend, disabled = false }: ChatInputProps) {
  const { t } = useTranslation();
  const [value, setValue] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const adjustHeight = useCallback(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = `${Math.min(el.scrollHeight, 96)}px`;
  }, []);

  const handleChange = useCallback(
    (e: ChangeEvent<HTMLTextAreaElement>) => {
      setValue(e.target.value);
      adjustHeight();
    },
    [adjustHeight],
  );

  const handleSend = useCallback(() => {
    const trimmed = value.trim();
    if (!trimmed || disabled) return;
    onSend(trimmed);
    setValue('');
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
    }
  }, [value, disabled, onSend]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend],
  );

  return (
    <Flex align="flex-end" gap="2" px="4" py="3" borderTop="1px solid" borderColor="gray.200">
      <label htmlFor="chat-message-input" style={{ position: 'absolute', width: 1, height: 1, overflow: 'hidden', clip: 'rect(0,0,0,0)', whiteSpace: 'nowrap' as const }}>{t('chat.input.placeholder')}</label>
      <StyledTextarea
        ref={textareaRef}
        id="chat-message-input"
        value={value}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        placeholder={t('chat.input.placeholder')}
        disabled={disabled}
        rows={1}
        flex="1"
        border="1px solid"
        borderColor="gray.200"
        borderRadius="lg"
        px="3"
        py="2"
        fontSize="sm"
        resize="none"
        _focus={{ borderColor: 'blue.500', outline: 'none', boxShadow: '0 0 0 1px var(--chakra-colors-blue-500)' }}
        _disabled={{ opacity: 0.5, cursor: 'not-allowed' }}
      />
      <StyledButton
        type="button"
        onClick={handleSend}
        disabled={disabled || !value.trim()}
        w="9"
        h="9"
        borderRadius="lg"
        bg="blue.500"
        color="white"
        display="flex"
        alignItems="center"
        justifyContent="center"
        fontSize="md"
        fontWeight="600"
        cursor="pointer"
        flexShrink={0}
        transition="background 0.15s"
        _hover={{ bg: 'blue.600' }}
        _disabled={{ opacity: 0.4, cursor: 'not-allowed' }}
        aria-label={t('chat.input.send')}
      >
        →
      </StyledButton>
    </Flex>
  );
}
