import { useMemo, useState, useCallback, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import type { EgressSettings } from '../domain/types';

export type ValidationErrors = Record<string, string | undefined>;

const URL_RE = /^https?:\/\/.+/i;

function isPositiveInteger(v: number): boolean {
  return Number.isInteger(v) && v > 0;
}

/**
 * Pure validation — returns all current field errors for the given settings.
 */
function validateSettings(
  settings: EgressSettings,
  t: (key: string, opts?: Record<string, unknown>) => string,
): ValidationErrors {
  const errors: ValidationErrors = {};

  /* ── Bearer ──────────────────────────────────────────── */
  if (settings.authType === 'bearer') {
    if (!settings.bearer.token.trim()) {
      errors['bearer.token'] = t('validation.required');
    }
  }

  /* ── OAuth2 ──────────────────────────────────────────── */
  if (settings.authType === 'oauth2') {
    if (!settings.oauth2.clientId.trim()) {
      errors['oauth2.clientId'] = t('validation.required');
    }
    if (!settings.oauth2.clientSecret.trim()) {
      errors['oauth2.clientSecret'] = t('validation.required');
    }
    if (!settings.oauth2.tokenUrl.trim()) {
      errors['oauth2.tokenUrl'] = t('validation.required');
    } else if (!URL_RE.test(settings.oauth2.tokenUrl.trim())) {
      errors['oauth2.tokenUrl'] = t('validation.invalidUrl');
    }
  }

  /* ── Retry policy ────────────────────────────────────── */
  const { maxAttempts, initialBackoffMs } = settings.retryPolicy;

  if (!isPositiveInteger(maxAttempts) || maxAttempts < 1 || maxAttempts > 10) {
    errors['retry.maxAttempts'] = Number.isInteger(maxAttempts)
      ? t('validation.retryRange', { min: 1, max: 10 })
      : t('validation.positiveInteger');
  }

  if (
    !isPositiveInteger(initialBackoffMs) ||
    initialBackoffMs < 100 ||
    initialBackoffMs > 60000
  ) {
    errors['retry.initialBackoffMs'] = Number.isInteger(initialBackoffMs)
      ? t('validation.retryRange', { min: 100, max: 60000 })
      : t('validation.positiveInteger');
  }

  return errors;
}

/**
 * Hook that provides debounced field-level validation for the settings form.
 *
 * - `errors`   — current field-level error map
 * - `isValid`  — true when zero errors
 * - `validateAll` — eagerly validate every field (call on submit)
 */
export function useSettingsValidation(settings: EgressSettings) {
  const { t } = useTranslation();
  const [errors, setErrors] = useState<ValidationErrors>({});
  const [submitted, setSubmitted] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  /* ── Debounced on-change validation ──────────────────── */
  useEffect(() => {
    if (!submitted) return; // don't show errors until first submit attempt

    clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      setErrors(validateSettings(settings, t));
    }, 300);

    return () => clearTimeout(timerRef.current);
  }, [settings, submitted, t]);

  /* ── Immediate full validation (on submit) ───────────── */
  const validateAll = useCallback((): boolean => {
    setSubmitted(true);
    const next = validateSettings(settings, t);
    setErrors(next);
    return Object.keys(next).length === 0;
  }, [settings, t]);

  const isValid = useMemo(
    () => Object.keys(errors).length === 0,
    [errors],
  );

  return { errors, isValid, validateAll } as const;
}
