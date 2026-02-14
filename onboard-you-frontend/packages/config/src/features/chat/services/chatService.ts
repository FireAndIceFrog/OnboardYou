import type { PipelineConfig, ActionConfig } from '@/shared/domain/types';
import { businessLabel } from '@/shared/domain/types';

/**
 * Determines if the user message implies adding a new step to the flow.
 * Returns the matching ActionConfig to append, or null.
 */
export function deriveFlowAction(userMessage: string): ActionConfig | null {
  const lower = userMessage.toLowerCase();

  if (lower.includes('phone') || lower.includes('cellphone') || lower.includes('international')) {
    return {
      id: `step-${Date.now()}`,
      actionType: 'cellphone_sanitizer',
      config: {
        name: 'Clean Phone Numbers',
        column: 'phone',
        default_country: 'US',
      },
    };
  }

  if (lower.includes('address') || lower.includes('clean up')) {
    return {
      id: `step-${Date.now()}`,
      actionType: 'regex_replace',
      config: {
        name: 'Clean Up Address Data',
        column: 'address',
        pattern: '\\s+',
        replacement: ' ',
      },
    };
  }

  if (lower.includes('duplicate') || lower.includes('dedup')) {
    return {
      id: `step-${Date.now()}`,
      actionType: 'identity_deduplicator',
      config: {
        name: 'Remove Duplicates',
        strategy: 'composite_key',
        keys: ['employeeId', 'email'],
      },
    };
  }

  if (lower.includes('mask') || lower.includes('sensitive') || lower.includes('pii')) {
    return {
      id: `step-${Date.now()}`,
      actionType: 'pii_masking',
      config: {
        name: 'Mask Sensitive Data',
        columns: [
          { column: 'ssn', strategy: 'full' },
          { column: 'email', strategy: 'partial' },
        ],
      },
    };
  }

  if (lower.includes('country') || lower.includes('standardise') || lower.includes('standardize')) {
    return {
      id: `step-${Date.now()}`,
      actionType: 'iso_country_sanitizer',
      config: {
        name: 'Standardise Country Codes',
        column: 'country',
      },
    };
  }

  if (lower.includes('diacritics') || lower.includes('special character') || lower.includes('accent')) {
    return {
      id: `step-${Date.now()}`,
      actionType: 'handle_diacritics',
      config: {
        name: 'Fix Special Characters',
        columns: ['firstName', 'lastName'],
      },
    };
  }

  return null;
}

export function generateResponse(config: PipelineConfig, userMessage: string): string {
  const lower = userMessage.toLowerCase();
  const actions = config.pipeline.actions;

  // ── Natural-language validation errors ──────────────────────
  if (lower.includes('social security') || lower.includes('ssn column')) {
    const hasSsn = actions.some(
      (a) =>
        JSON.stringify(a.config).toLowerCase().includes('ssn') ||
        JSON.stringify(a.config).toLowerCase().includes('social_security'),
    );
    if (!hasSsn) {
      return (
        `I couldn't find a "Social Security" column in your data. ` +
        `Should I skip this step, or would you like to map a different column to it?`
      );
    }
  }

  // ── Action-driven responses (flow was updated) ─────────────
  const flowAction = deriveFlowAction(userMessage);
  if (flowAction) {
    const label = businessLabel(flowAction.actionType);
    return (
      `Done! I've added a new **"${label}"** step to your flow. ` +
      `You should see it appear on the flowchart now.\n\n` +
      `Would you like to adjust its settings, or add another step?`
    );
  }

  // ── Informational responses ────────────────────────────────
  if (lower.includes('explain') || lower.includes('what does') || lower.includes('describe')) {
    return (
      `Your connection **${config.name}** has **${actions.length}** step(s):\n\n` +
      actions
        .map((a, i) => `${i + 1}. **${(a.config.name as string) || businessLabel(a.actionType)}**`)
        .join('\n') +
      `\n\nWould you like me to explain any specific step in more detail?`
    );
  }

  if (lower.includes('schedule') || lower.includes('frequency') || lower.includes('cron')) {
    return (
      `**${config.name}** syncs on schedule: \`${config.cron}\`.\n\n` +
      `Last edited: ${config.lastEdited || 'unknown'}. Would you like to change the sync frequency?`
    );
  }

  if (lower.includes('help') || lower.includes('what can you')) {
    return (
      `I can help you:\n\n` +
      `- **Clean up data** — addresses, phone numbers, special characters\n` +
      `- **Remove duplicates** from your employee records\n` +
      `- **Mask sensitive data** like SSNs and emails\n` +
      `- **Standardise fields** — country codes, column names\n` +
      `- **Explain** each step in your flow\n\n` +
      `Just tell me what you need in plain English!`
    );
  }

  if (lower.includes('error') || lower.includes('fail') || lower.includes('wrong')) {
    return (
      `I noticed something might not be right. Let me check…\n\n` +
      `Your flow currently has **${actions.length}** step(s). ` +
      `Could you describe what went wrong? For example:\n` +
      `- "I'm seeing the wrong column names"\n` +
      `- "Some records are missing"\n\n` +
      `I'll do my best to help!`
    );
  }

  return (
    `I see your connection **${config.name}** has ${actions.length} step(s). ` +
    `Tell me what you'd like to change — for example:\n\n` +
    `• "Clean up my address data"\n` +
    `• "Remove duplicate records"\n` +
    `• "Format phone numbers to international"\n\n` +
    `I'll update the flow for you automatically!`
  );
}
