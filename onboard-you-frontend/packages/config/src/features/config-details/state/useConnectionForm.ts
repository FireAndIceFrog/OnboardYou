import { useState, useCallback, useMemo } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import type { ConnectionForm, WorkdayFields, CsvFields, SystemId } from '../domain/types';
import { INITIAL_CONNECTION_FORM } from '../domain/types';

export type ValidationErrors = Record<string, string | undefined>;

const URL_RE = /^https?:\/\/.+/i;

export function useConnectionForm() {
  const { customerCompanyId } = useParams<{ customerCompanyId: string }>();
  const navigate = useNavigate();
  const { t } = useTranslation();
  const [form, setForm] = useState<ConnectionForm>(INITIAL_CONNECTION_FORM);
  const [errors, setErrors] = useState<ValidationErrors>({});
  const [submitted, setSubmitted] = useState(false);

  /* ── Validation helpers ─────────────────────────────────── */

  const validateAllFields = useCallback(
    (f: ConnectionForm): ValidationErrors => {
      const errs: ValidationErrors = {};

      if (!f.system) {
        errs.system = t('validation.selectSystem');
      }

      if (f.system === 'workday') {
        const w = f.workday;
        if (!w.tenantUrl.trim()) {
          errs['workday.tenantUrl'] = t('validation.required');
        } else if (!URL_RE.test(w.tenantUrl.trim())) {
          errs['workday.tenantUrl'] = t('validation.invalidUrl');
        }
        if (!w.username.trim()) {
          errs['workday.username'] = t('validation.required');
        }
        if (!w.password) {
          errs['workday.password'] = t('validation.required');
        } else if (w.password.length < 8) {
          errs['workday.password'] = t('validation.minLength', { min: 8 });
        }
        // API version is tenantId in the form
        if (!w.tenantId.trim()) {
          errs['workday.tenantId'] = t('validation.required');
        }
        // At least one response group
        const groups = w.responseGroup.split(',').filter(Boolean);
        if (groups.length === 0) {
          errs['workday.responseGroup'] = t('validation.selectResponseGroup');
        }
      }

      if (f.system === 'csv') {
        if (!f.csv.csvPath.trim()) {
          errs['csv.csvPath'] = t('validation.required');
        }
      }

      return errs;
    },
    [t],
  );

  const validateField = useCallback(
    (name: string) => {
      // Re-validate all then only update the single field (keeps other errors intact)
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
    // Clear system error on selection
    setErrors((prev) => ({ ...prev, system: undefined }));
  }, []);

  const handleChange = useCallback(
    (field: keyof ConnectionForm) => (e: React.ChangeEvent<HTMLInputElement>) => {
      setForm((prev) => ({ ...prev, [field]: e.target.value }));
    },
    [],
  );

  const handleWorkdayChange = useCallback(
    (field: keyof WorkdayFields) => (e: React.ChangeEvent<HTMLInputElement>) => {
      setForm((prev) => ({
        ...prev,
        workday: { ...prev.workday, [field]: e.target.value },
      }));
      if (submitted) {
        // Re-validate the changed field after a tick so form state is updated
        setTimeout(() => {
          setErrors((prev) => {
            const all = validateAllFields({
              ...form,
              workday: { ...form.workday, [field]: e.target.value },
            } as ConnectionForm);
            return { ...prev, [`workday.${field}`]: all[`workday.${field}`] };
          });
        }, 0);
      }
    },
    [form, submitted, validateAllFields],
  );

  const handleCsvChange = useCallback(
    (field: keyof CsvFields) => (e: React.ChangeEvent<HTMLInputElement>) => {
      setForm((prev) => ({
        ...prev,
        csv: { ...prev.csv, [field]: e.target.value },
      }));
      if (submitted) {
        setTimeout(() => {
          setErrors((prev) => {
            const all = validateAllFields({
              ...form,
              csv: { ...form.csv, [field]: e.target.value },
            } as ConnectionForm);
            return { ...prev, [`csv.${field}`]: all[`csv.${field}`] };
          });
        }, 0);
      }
    },
    [form, submitted, validateAllFields],
  );

  const handleResponseGroupToggle = useCallback((value: string) => {
    setForm((prev) => {
      const current = prev.workday.responseGroup.split(',').filter(Boolean);
      const next = current.includes(value)
        ? current.filter((v) => v !== value)
        : [...current, value];
      const newGroup = next.join(',');

      if (submitted) {
        setTimeout(() => {
          setErrors((prevErrors) => {
            const groups = newGroup.split(',').filter(Boolean);
            return {
              ...prevErrors,
              'workday.responseGroup': groups.length === 0 ? t('validation.selectResponseGroup') : undefined,
            };
          });
        }, 0);
      }

      return {
        ...prev,
        workday: { ...prev.workday, responseGroup: newGroup },
      };
    });
  }, [submitted, t]);

  /* ── Computed validity ──────────────────────────────────── */
  const isValid = useMemo(() => {
    if (!form.system || !form.displayName) return false;
    if (form.system === 'workday') {
      const w = form.workday;
      const groups = w.responseGroup.split(',').filter(Boolean);
      return !!(
        w.tenantUrl &&
        URL_RE.test(w.tenantUrl.trim()) &&
        w.tenantId &&
        w.username &&
        w.password &&
        w.password.length >= 8 &&
        groups.length > 0
      );
    }
    if (form.system === 'csv') {
      return !!form.csv.csvPath;
    }
    return false;
  }, [form]);

  const handleNext = useCallback(() => {
    if (!validate()) return;
    const target = customerCompanyId ?? 'new';
    navigate(`/config/${target}/flow`, { state: { connection: form } });
  }, [validate, customerCompanyId, form, navigate]);

  const handleBack = useCallback(() => {
    navigate(-1);
  }, [navigate]);

  /** Active response groups as a Set for quick lookup. */
  const activeGroups = useMemo(
    () => new Set(form.workday.responseGroup.split(',').filter(Boolean)),
    [form.workday.responseGroup],
  );

  return {
    form,
    errors,
    isValid,
    activeGroups,
    handleSystemSelect,
    handleChange,
    handleWorkdayChange,
    handleCsvChange,
    handleResponseGroupToggle,
    handleNext,
    handleBack,
    validate,
    validateField,
  } as const;
}
