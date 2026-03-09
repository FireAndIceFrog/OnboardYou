import { useState, useCallback, useMemo } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import type { ConnectionForm, SystemId, ValidationErrors } from '../domain/types';
import { INITIAL_CONNECTION_FORM } from '../domain/types';
import { ConnectorConfigFactory } from './connectorConfigs/connectorConfigFactory';
import type { ConnectorChangeEvent } from './connectorConfigs/IConnectorConfig';

const connectorFactory = new ConnectorConfigFactory();

export function useConnectionForm() {
  const { customerCompanyId } = useParams<{ customerCompanyId: string }>();
  const navigate = useNavigate();
  const { t } = useTranslation();
  const [form, setForm] = useState<ConnectionForm>(INITIAL_CONNECTION_FORM);
  const [errors, setErrors] = useState<ValidationErrors>({});
  const [submitted, setSubmitted] = useState(false);

  /* ── Active connector config ────────────────────────────── */

  const config = useMemo(
    () => connectorFactory.getConfig(form.system),
    [form.system],
  );

  /* ── Validation helpers ─────────────────────────────────── */

  const validateAllFields = useCallback(
    (f: ConnectionForm): ValidationErrors => {
      if (!f.system) return { system: t('validation.selectSystem') };
      return connectorFactory.getConfig(f.system).validate(f, t);
    },
    [t],
  );

  const validateField = useCallback(
    (name: string) => {
      setErrors((prev) => {
        const all = validateAllFields(form);
        return { ...prev, [name]: all[name] };
      });
    },
    [form, validateAllFields],
  );

  const validate = useCallback((): boolean => {
    setSubmitted(true);
    const all = validateAllFields(form);
    setErrors(all);
    return Object.keys(all).length === 0;
  }, [form, validateAllFields]);

  /* ── Handlers ───────────────────────────────────────────── */

  const handleSystemSelect = useCallback((systemId: SystemId) => {
    setForm((prev) => ({ ...prev, system: systemId }));
    setErrors((prev) => ({ ...prev, system: undefined }));
  }, []);

  const handleChange = useCallback(
    (field: keyof ConnectionForm) => (e: React.ChangeEvent<HTMLInputElement>) => {
      setForm((prev) => ({ ...prev, [field]: e.target.value }));
    },
    [],
  );

  /**
   * Single handler for all connector-specific field changes.
   * Delegates to the active connector's async generator `applyChange`.
   * File events always validate (triggers upload). Field/toggle events
   * only validate after the form has been submitted once.
   */
  const handleConnectorChange = useCallback(
    (event: ConnectorChangeEvent) => {
      const shouldValidate = event.type === 'file' || submitted;
      void (async () => {
        for await (const { form: next, errors: delta } of config.applyChange(event, form, {
          validate: shouldValidate,
          t,
          companyId: customerCompanyId ?? 'new',
        })) {
          setForm(next);
          setErrors((prev) => ({ ...prev, ...delta }));
        }
      })();
    },
    [config, customerCompanyId, form, submitted, t],
  );

  /* ── Computed validity ──────────────────────────────────── */

  const isValid = useMemo(() => {
    if (!form.system || !form.displayName) return false;
    return config.isFormValid(form);
  }, [form, config]);

  const handleNext = useCallback(() => {
    if (!validate()) return;
    const target = customerCompanyId ?? 'new';
    navigate(`/config/${target}/flow`, { state: { connection: form } });
  }, [validate, customerCompanyId, form, navigate]);

  const handleBack = useCallback(() => {
    navigate(-1);
  }, [navigate]);

  return {
    form,
    errors,
    isValid,
    config,
    handleSystemSelect,
    handleChange,
    handleConnectorChange,
    handleNext,
    handleBack,
    validate,
    validateField,
  } as const;
}

