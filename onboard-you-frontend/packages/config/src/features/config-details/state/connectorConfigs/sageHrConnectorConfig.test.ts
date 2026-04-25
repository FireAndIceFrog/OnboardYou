import { describe, it, expect } from 'vitest';
import type { ConnectionForm } from '../../domain/types';
import type { ApplyChangeContext, ConnectorChangeEvent, ConnectorChangeResult } from './IConnectorConfig';
import { SageHrConnectorConfig } from './sageHrConnectorConfig';

const t = (key: string) => key;
const ctx = (validate = false): ApplyChangeContext => ({ validate, t: t as any });

async function collect(gen: AsyncGenerator<ConnectorChangeResult>): Promise<ConnectorChangeResult[]> {
  const results: ConnectorChangeResult[] = [];
  for await (const r of gen) results.push(r);
  return results;
}

function baseForm(overrides: Partial<ConnectionForm> = {}): ConnectionForm {
  return {
    system: 'sage_hr' as any,
    displayName: 'Test',
    workday: { tenantUrl: '', tenantId: '', username: '', password: '', workerCountLimit: '200', responseGroup: 'include_personal_information,include_employment_information' },
    sageHr: { subdomain: '', apiToken: '', includeTeamHistory: false, includeEmploymentStatusHistory: false, includePositionHistory: false },
    genericIngestion: { filename: '', columns: [], uploadStatus: 'idle' as const, uploadError: null, conversionStatus: null },
    emailIngestion: { allowedSenders: '', subjectFilter: '' },
    ...overrides,
  };
}

describe('SageHrConnectorConfig', () => {
  const config = new SageHrConnectorConfig();

  describe('applyChange', () => {
    interface Case { name: string; event: ConnectorChangeEvent; validate: boolean; form?: ConnectionForm; check: (r: ConnectorChangeResult) => void }
    const cases: Case[] = [
      {
        name: 'field event updates sageHr field',
        event: { type: 'field', key: 'subdomain', value: 'acme' },
        validate: false,
        check: (r) => expect(r.form.sageHr.subdomain).toBe('acme'),
      },
      {
        name: 'toggle flips boolean flag on (false → true)',
        event: { type: 'toggle', key: 'includeTeamHistory' },
        validate: false,
        check: (r) => expect(r.form.sageHr.includeTeamHistory).toBe(true),
      },
      {
        name: 'toggle flips boolean flag off (true → false)',
        event: { type: 'toggle', key: 'includeTeamHistory' },
        validate: false,
        form: baseForm({ sageHr: { subdomain: '', apiToken: '', includeTeamHistory: true, includeEmploymentStatusHistory: false, includePositionHistory: false } }),
        check: (r) => expect(r.form.sageHr.includeTeamHistory).toBe(false),
      },
      {
        name: 'returns errors when validate=true and fields empty',
        event: { type: 'field', key: 'subdomain', value: '' },
        validate: true,
        check: (r) => expect(r.errors['sageHr.subdomain']).toBeDefined(),
      },
      {
        name: 'no validation errors when validate=false',
        event: { type: 'field', key: 'subdomain', value: '' },
        validate: false,
        check: (r) => expect(r.errors).toEqual({}),
      },
    ];

    it.each(cases)('$name', async ({ event, validate, form, check }) => {
      const [result] = await collect(config.applyChange(event, form ?? baseForm(), ctx(validate)));
      check(result);
    });
  });

  describe('validate', () => {
    it('returns errors for empty required fields', () => {
      const errs = config.validate(baseForm(), t as any);
      expect(errs['sageHr.subdomain']).toBeDefined();
      expect(errs['sageHr.apiToken']).toBeDefined();
    });

    it('returns error when apiToken is too short', () => {
      const form = baseForm({ sageHr: { subdomain: 'acme', apiToken: 'short', includeTeamHistory: false, includeEmploymentStatusHistory: false, includePositionHistory: false } });
      const errs = config.validate(form, t as any);
      expect(errs['sageHr.apiToken']).toBeDefined();
    });

    it('returns no errors when fields are valid', () => {
      const form = baseForm({ sageHr: { subdomain: 'acme', apiToken: 'longtoken1', includeTeamHistory: false, includeEmploymentStatusHistory: false, includePositionHistory: false } });
      const errs = config.validate(form, t as any);
      expect(Object.keys(errs)).toHaveLength(0);
    });
  });

  describe('isFormValid', () => {
    it('returns false for empty form', () => {
      expect(config.isFormValid(baseForm())).toBe(false);
    });

    it('returns true when subdomain and long token provided', () => {
      const form = baseForm({ sageHr: { subdomain: 'acme', apiToken: 'longtoken1', includeTeamHistory: false, includeEmploymentStatusHistory: false, includePositionHistory: false } });
      expect(config.isFormValid(form)).toBe(true);
    });
  });
});
