import i18n from '@/i18n';
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
      return i18n.t('chat.responses.ssnNotFound');
    }
  }

  // ── Action-driven responses (flow was updated) ─────────────
  const flowAction = deriveFlowAction(userMessage);
  if (flowAction) {
    const label = businessLabel(flowAction.actionType);
    return i18n.t('chat.responses.actionAdded', { label });
  }

  // ── Informational responses ────────────────────────────────
  if (lower.includes('explain') || lower.includes('what does') || lower.includes('describe')) {
    const steps = actions
      .map((a, i) => `${i + 1}. **${(a.config.name as string) || businessLabel(a.actionType)}**`)
      .join('\n');
    return i18n.t('chat.responses.explain', { name: config.name, count: actions.length, steps });
  }

  if (lower.includes('schedule') || lower.includes('frequency') || lower.includes('cron')) {
    return i18n.t('chat.responses.schedule', {
      name: config.name,
      cron: config.cron,
      lastEdited: config.lastEdited || i18n.t('chat.responses.lastEditedUnknown'),
    });
  }

  if (lower.includes('help') || lower.includes('what can you')) {
    return i18n.t('chat.responses.help');
  }

  if (lower.includes('error') || lower.includes('fail') || lower.includes('wrong')) {
    return i18n.t('chat.responses.error', { count: actions.length });
  }

  return i18n.t('chat.responses.default', { name: config.name, count: actions.length });
}
