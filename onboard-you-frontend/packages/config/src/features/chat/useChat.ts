import { useState, useCallback, useRef } from 'react';
import type { ChatMessage, PipelineConfig } from '@/types';

function generateId(): string {
  return `msg-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function timestamp(): string {
  return new Date().toISOString();
}

function buildSystemContext(config: PipelineConfig): string {
  const txNames = config.pipeline.transformations.map((t) => t.name).join(', ');
  return (
    `You are the OnboardYou Pipeline Assistant. You are helping the user understand and configure ` +
    `the "${config.name}" pipeline (status: ${config.status}, v${config.version}). ` +
    `This pipeline ingests from ${config.pipeline.ingestion.source} via ${config.pipeline.ingestion.type}, ` +
    `applies transformations [${txNames}], and outputs to ${config.pipeline.egress.destination} ` +
    `via ${config.pipeline.egress.type}. ` +
    `Answer clearly and concisely. Offer suggestions when appropriate.`
  );
}

function generateMockResponse(config: PipelineConfig, userMessage: string): string {
  const lower = userMessage.toLowerCase();

  if (lower.includes('explain') || lower.includes('what does') || lower.includes('describe')) {
    return (
      `This pipeline, **${config.name}**, pulls data from **${config.pipeline.ingestion.source}** ` +
      `using a \`${config.pipeline.ingestion.type}\` connector. It then passes records through ` +
      `${config.pipeline.transformations.length} transformation stage(s):\n\n` +
      config.pipeline.transformations
        .map((t, i) => `${i + 1}. **${t.name}** — a \`${t.type}\` step`)
        .join('\n') +
      `\n\nFinally, the processed data is written to **${config.pipeline.egress.destination}** ` +
      `via \`${config.pipeline.egress.type}\`.\n\nWould you like me to dive deeper into any specific stage?`
    );
  }

  if (lower.includes('status') || lower.includes('health')) {
    return (
      `The current status of **${config.name}** is \`${config.status}\`. ` +
      (config.status === 'active'
        ? 'Everything looks healthy — the pipeline is running on schedule.'
        : config.status === 'paused'
          ? 'The pipeline is paused. You can resume it from the pipeline settings.'
          : config.status === 'draft'
            ? "This config is still in draft. It hasn't been activated yet."
            : 'There appears to be an error. Check the ingestion logs for details.') +
      ' Let me know if you need help troubleshooting.'
    );
  }

  if (lower.includes('transform') || lower.includes('step') || lower.includes('stage')) {
    return (
      `This pipeline has **${config.pipeline.transformations.length}** transformation stage(s):\n\n` +
      config.pipeline.transformations
        .map(
          (t) =>
            `- **${t.name}** (\`${t.type}\`): depends on [${t.dependsOn.length > 0 ? t.dependsOn.join(', ') : 'ingestion'}]`,
        )
        .join('\n') +
      `\n\nEach stage processes records sequentially based on its dependencies. Would you like to edit any of these?`
    );
  }

  if (lower.includes('help') || lower.includes('what can you')) {
    return (
      `I can help you with:\n\n` +
      `- **Explaining** the pipeline stages and data flow\n` +
      `- **Checking status** and health of the pipeline\n` +
      `- **Describing transformations** and their dependencies\n` +
      `- **Suggesting improvements** to your pipeline configuration\n` +
      `- **Answering questions** about field mappings, filters, and egress settings\n\n` +
      `Just ask me anything about this pipeline!`
    );
  }

  return (
    `I can see your pipeline **${config.name}** has ${config.pipeline.transformations.length} transformation stage(s). ` +
    `It ingests from **${config.pipeline.ingestion.source}** and outputs to **${config.pipeline.egress.destination}**. ` +
    `Would you like me to explain any of the stages in detail, or help you modify the configuration?`
  );
}

interface UseChatReturn {
  messages: ChatMessage[];
  isTyping: boolean;
  sendMessage: (content: string) => void;
  clearChat: () => void;
}

export function useChat(config: PipelineConfig | null): UseChatReturn {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isTyping, setIsTyping] = useState(false);
  const typingTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);

  const sendMessage = useCallback(
    (content: string) => {
      if (!content.trim() || !config) return;

      const userMessage: ChatMessage = {
        id: generateId(),
        role: 'user',
        content: content.trim(),
        timestamp: timestamp(),
      };

      setMessages((prev) => [...prev, userMessage]);
      setIsTyping(true);

      // Simulate AI response delay (400-1200ms)
      const delay = 400 + Math.random() * 800;
      typingTimeout.current = setTimeout(() => {
        const response = generateMockResponse(config, content);
        const assistantMessage: ChatMessage = {
          id: generateId(),
          role: 'assistant',
          content: response,
          timestamp: timestamp(),
        };
        setMessages((prev) => [...prev, assistantMessage]);
        setIsTyping(false);
      }, delay);
    },
    [config],
  );

  const clearChat = useCallback(() => {
    if (typingTimeout.current) {
      clearTimeout(typingTimeout.current);
      typingTimeout.current = null;
    }
    setMessages([]);
    setIsTyping(false);
  }, []);

  return { messages, isTyping, sendMessage, clearChat };
}
