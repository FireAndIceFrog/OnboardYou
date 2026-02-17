import i18n from '@/i18n';
import type { PipelineConfig, ActionConfig, ActionConfigPayload } from '@/generated/api';
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
      action_type: 'cellphone_sanitizer',
      config: {
        phone_column: 'phone',
        country_columns: ['country'],
        output_column: 'phone_intl',
      } satisfies ActionConfigPayload,
    };
  }

  if (lower.includes('address') || lower.includes('clean up')) {
    return {
      id: `step-${Date.now()}`,
      action_type: 'regex_replace',
      config: {
        column: 'address',
        pattern: '\\s+',
        replacement: ' ',
      } satisfies ActionConfigPayload,
    };
  }

  if (lower.includes('duplicate') || lower.includes('dedup')) {
    return {
      id: `step-${Date.now()}`,
      action_type: 'identity_deduplicator',
      config: {
        columns: ['national_id', 'email'],
        employee_id_column: 'employee_id',
      } satisfies ActionConfigPayload,
    };
  }

  if (lower.includes('mask') || lower.includes('sensitive') || lower.includes('pii')) {
    return {
      id: `step-${Date.now()}`,
      action_type: 'pii_masking',
      config: {
        columns: [
          { name: 'ssn', strategy: { Redact: { keep_last: 4, mask_prefix: '***-**-' } } },
          { name: 'salary', strategy: 'Zero' },
        ],
      } satisfies ActionConfigPayload,
    };
  }

  if (lower.includes('country') || lower.includes('standardise') || lower.includes('standardize')) {
    return {
      id: `step-${Date.now()}`,
      action_type: 'iso_country_sanitizer',
      config: {
        source_column: 'country',
        output_column: 'country_iso',
        output_format: 'alpha2',
      } satisfies ActionConfigPayload,
    };
  }

  if (lower.includes('diacritics') || lower.includes('special character') || lower.includes('accent')) {
    return {
      id: `step-${Date.now()}`,
      action_type: 'handle_diacritics',
      config: {
        columns: ['firstName', 'lastName'],
      } satisfies ActionConfigPayload,
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
    const label = businessLabel(flowAction.action_type);
    return i18n.t('chat.responses.actionAdded', { label });
  }

  // ── Informational responses ────────────────────────────────
  if (lower.includes('explain') || lower.includes('what does') || lower.includes('describe')) {
    const steps = actions
      .map((a, i) => `${i + 1}. **${businessLabel(a.action_type)}**`)
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
