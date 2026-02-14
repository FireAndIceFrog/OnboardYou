import type { PipelineConfig } from '@/shared/domain/types';

export function generateResponse(config: PipelineConfig, userMessage: string): string {
  const lower = userMessage.toLowerCase();
  const actions = config.pipeline.actions;

  if (lower.includes('explain') || lower.includes('what does') || lower.includes('describe')) {
    return (
      `This pipeline, **${config.name}**, is scheduled with \`${config.cron}\` and has ` +
      `**${actions.length}** action(s):\n\n` +
      actions
        .map((a, i) => `${i + 1}. **${a.id}** — \`${a.actionType}\``)
        .join('\n') +
      `\n\nWould you like me to dive deeper into any specific action?`
    );
  }

  if (lower.includes('schedule') || lower.includes('cron')) {
    return (
      `The pipeline **${config.name}** runs on schedule: \`${config.cron}\`.\n\n` +
      `Last edited: ${config.lastEdited || 'unknown'}.`
    );
  }

  if (lower.includes('action') || lower.includes('step') || lower.includes('stage')) {
    return (
      `This pipeline has **${actions.length}** action(s):\n\n` +
      actions
        .map((a, i) => `${i + 1}. **${a.id}** (\`${a.actionType}\`) — config keys: ${Object.keys(a.config).join(', ') || 'none'}`)
        .join('\n') +
      `\n\nActions execute sequentially in the order listed.`
    );
  }

  if (lower.includes('help') || lower.includes('what can you')) {
    return (
      `I can help you with:\n\n` +
      `- **Explaining** the pipeline actions and data flow\n` +
      `- **Checking schedule** and cron configuration\n` +
      `- **Describing actions** and their configurations\n` +
      `- **Suggesting improvements** to your pipeline configuration\n` +
      `- **Answering questions** about action types and settings\n\n` +
      `Just ask me anything about this pipeline!`
    );
  }

  return (
    `I can see your pipeline **${config.name}** has ${actions.length} action(s) ` +
    `running on \`${config.cron}\`. ` +
    `Would you like me to explain any of the actions in detail, or help you modify the configuration?`
  );
}
