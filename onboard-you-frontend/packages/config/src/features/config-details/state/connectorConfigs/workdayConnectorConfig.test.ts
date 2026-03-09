import { describe, it, expect } from 'vitest';
import type { ConnectionForm } from '../../domain/types';
import type { ApplyChangeContext, ConnectorChangeEvent, ConnectorChangeResult } from './IConnectorConfig';
import { WorkdayConnectorConfig } from './workdayConnectorConfig';

const t = (key: string) => key;
const ctx = (validate = false): ApplyChangeContext => ({ validate, t: t as any });

async function collect(gen: AsyncGenerator<ConnectorChangeResult>): Promise<ConnectorChangeResult[]> {
  const results: ConnectorChangeResult[] = [];
  for await (const r of gen) results.push(r);
  return results;
}

function baseForm(): ConnectionForm {
  return {
    system: 'workday' as any,
    displayName: 'Test',
    workday: { tenantUrl: '', tenantId: '', username: '', password: '', workerCountLimit: '200', responseGroup: 'include_personal_information,include_employment_information' },
    sageHr: { subdomain: '', apiToken: '', includeTeamHistory: false, includeEmploymentStatusHistory: false, includePositionHistory: false },
    csv: { filename: '', columns: [], uploadStatus: 'idle' as const, uploadError: null },
  };
}

describe('WorkdayConnectorConfig', () => {
  const config = new WorkdayConnectorConfig();

  describe('applyChange', () => {
    interface Case { name: string; event: ConnectorChangeEvent; validate: boolean; check: (r: ConnectorChangeResult) => void }
    const cases: Case[] = [
      {
        name: 'field event updates workday field',
        event: { type: 'field', key: 'tenantUrl', value: 'https://wd.example.com' },
        validate: false,
        check: (r) => expect(r.form.workday.tenantUrl).toBe('https://wd.example.com'),
      },
      {
        name: 'toggle adds a response group',
        event: { type: 'toggle', key: 'include_compensation' },
        validate: false,
        check: (r) => expect(r.form.workday.responseGroup).toContain('include_compensation'),
      },
      {
        name: 'toggle removes existing response group',
        event: { type: 'toggle', key: 'include_personal_information' },
        validate: false,
        check: (r) => expect(r.form.workday.responseGroup).not.toContain('include_personal_information'),
      },
      {
        name: 'no validation errors when validate=false',
        event: { type: 'field', key: 'tenantUrl', value: '' },
        validate: false,
        check: (r) => expect(r.errors).toEqual({}),
      },
      {
        name: 'returns validation errors when validate=true',
        event: { type: 'field', key: 'tenantUrl', value: '' },
        validate: true,
        check: (r) => expect(Object.keys(r.errors).length).toBeGreaterThan(0),
      },
    ];

    it.each(cases)('$name', async ({ event, validate, check }) => {
      const [result] = await collect(config.applyChange(event, baseForm(), ctx(validate)));
      check(result);
    });
  });

  describe('validate', () => {
    it('returns errors for all empty required fields', () => {
      const errs = config.validate(baseForm(), t as any);
      expect(errs['workday.tenantUrl']).toBeDefined();
      expect(errs['workday.tenantId']).toBeDefined();
      expect(errs['workday.username']).toBeDefined();
      expect(errs['workday.password']).toBeDefined();
    });

    it('returns no errors when all fields are valid', () => {
      const form = baseForm();
      form.workday = { ...form.workday, tenantUrl: 'https://t.example.com', tenantId: 'tid', username: 'u', password: 'longpass1' };
      const errs = config.validate(form, t as any);
      expect(Object.keys(errs)).toHaveLength(0);
    });
  });

  describe('isFormValid', () => {
    it('returns false for empty form', () => {
      expect(config.isFormValid(baseForm())).toBe(false);
    });

    it('returns true for complete form', () => {
      const form = baseForm();
      form.workday = { ...form.workday, tenantUrl: 'https://t.example.com', tenantId: 'tid', username: 'u', password: 'longpass1' };
      expect(config.isFormValid(form)).toBe(true);
    });
  });
});
