import { useState, useCallback, useMemo } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import type { ConnectionForm, WorkdayFields, CsvFields, SystemId } from '../domain/types';
import { INITIAL_CONNECTION_FORM } from '../domain/types';

export function useConnectionForm() {
  const { customerCompanyId } = useParams<{ customerCompanyId: string }>();
  const navigate = useNavigate();
  const [form, setForm] = useState<ConnectionForm>(INITIAL_CONNECTION_FORM);

  const handleSystemSelect = useCallback((systemId: SystemId) => {
    setForm((prev) => ({ ...prev, system: systemId }));
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
    },
    [],
  );

  const handleCsvChange = useCallback(
    (field: keyof CsvFields) => (e: React.ChangeEvent<HTMLInputElement>) => {
      setForm((prev) => ({
        ...prev,
        csv: { ...prev.csv, [field]: e.target.value },
      }));
    },
    [],
  );

  const handleResponseGroupToggle = useCallback((value: string) => {
    setForm((prev) => {
      const current = prev.workday.responseGroup.split(',').filter(Boolean);
      const next = current.includes(value)
        ? current.filter((v) => v !== value)
        : [...current, value];
      return {
        ...prev,
        workday: { ...prev.workday, responseGroup: next.join(',') },
      };
    });
  }, []);

  /* ── Validation ─────────────────────────────────────────── */
  const isValid = useMemo(() => {
    if (!form.system || !form.displayName) return false;
    if (form.system === 'workday') {
      const w = form.workday;
      return !!(w.tenantUrl && w.tenantId && w.username && w.password);
    }
    if (form.system === 'csv') {
      return !!form.csv.csvPath;
    }
    return false;
  }, [form]);

  const handleNext = useCallback(() => {
    if (!isValid) return;
    const target = customerCompanyId ?? 'new';
    navigate(`/config/${target}/flow`, { state: { connection: form } });
  }, [isValid, customerCompanyId, form, navigate]);

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
    isValid,
    activeGroups,
    handleSystemSelect,
    handleChange,
    handleWorkdayChange,
    handleCsvChange,
    handleResponseGroupToggle,
    handleNext,
    handleBack,
  } as const;
}
