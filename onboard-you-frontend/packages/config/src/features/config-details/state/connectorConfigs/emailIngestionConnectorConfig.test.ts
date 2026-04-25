import { describe, it, expect } from 'vitest';
import type { ConnectionForm } from '../../domain/types';
import type { ApplyChangeContext, ConnectorChangeEvent, ConnectorChangeResult } from './IConnectorConfig';
import { EmailIngestionConnectorConfig } from './emailIngestionConnectorConfig';

const t = (key: string) => key;
const ctx = (validate = false): ApplyChangeContext => ({ validate, t: t as any });

async function collect(gen: AsyncGenerator<ConnectorChangeResult>): Promise<ConnectorChangeResult[]> {
  const results: ConnectorChangeResult[] = [];
  for await (const r of gen) results.push(r);
  return results;
}

function baseForm(overrides: Partial<ConnectionForm> = {}): ConnectionForm {
  return {
    system: 'email_ingestion' as any,
    displayName: 'Test',
    workday: { tenantUrl: '', tenantId: '', username: '', password: '', workerCountLimit: '200', responseGroup: '' },
    sageHr: { subdomain: '', apiToken: '', includeTeamHistory: false, includeEmploymentStatusHistory: false, includePositionHistory: false },
    genericIngestion: { filename: '', columns: [], uploadStatus: 'idle' as const, uploadError: null, conversionStatus: null },
    emailIngestion: { allowedSenders: '', subjectFilter: '' },
    ...overrides,
  };
}

describe('EmailIngestionConnectorConfig', () => {
  const config = new EmailIngestionConnectorConfig();

  // ── getDefaultState ─────────────────────────────────────────

  it('getDefaultState returns empty emailIngestion fields', () => {
    const state = config.getDefaultState();
    expect(state.emailIngestion?.allowedSenders).toBe('');
    expect(state.emailIngestion?.subjectFilter).toBe('');
  });

  // ── applyChange ──────────────────────────────────────────────

  describe('applyChange', () => {
    interface Case {
      name: string;
      event: ConnectorChangeEvent;
      validate: boolean;
      form?: ConnectionForm;
      check: (r: ConnectorChangeResult) => void;
    }

    const cases: Case[] = [
      {
        name: 'field event updates allowedSenders',
        event: { type: 'field', key: 'allowedSenders', value: 'hr@acme.com' },
        validate: false,
        check: (r) => expect(r.form.emailIngestion.allowedSenders).toBe('hr@acme.com'),
      },
      {
        name: 'field event updates subjectFilter',
        event: { type: 'field', key: 'subjectFilter', value: 'Monthly Roster' },
        validate: false,
        form: baseForm({ emailIngestion: { allowedSenders: 'hr@acme.com', subjectFilter: '' } }),
        check: (r) => expect(r.form.emailIngestion.subjectFilter).toBe('Monthly Roster'),
      },
      {
        name: 'returns validation errors when validate=true and senders empty',
        event: { type: 'field', key: 'allowedSenders', value: '' },
        validate: true,
        check: (r) => expect(r.errors['emailIngestion.allowedSenders']).toBeDefined(),
      },
      {
        name: 'no errors when validate=false',
        event: { type: 'field', key: 'allowedSenders', value: '' },
        validate: false,
        check: (r) => expect(r.errors).toEqual({}),
      },
      {
        name: 'non-field event yields nothing',
        event: { type: 'file', file: new File([''], 'test.csv') },
        validate: false,
        check: (_r) => { /* unreachable — generator yields nothing for file events */ },
      },
    ];

    it.each(cases.slice(0, 4))('$name', async ({ event, validate, form, check }) => {
      const [result] = await collect(config.applyChange(event, form ?? baseForm(), ctx(validate)));
      check(result);
    });

    it('non-field event yields nothing', async () => {
      const results = await collect(
        config.applyChange({ type: 'file', file: new File([''], 'test.csv') }, baseForm(), ctx()),
      );
      expect(results).toHaveLength(0);
    });
  });

  // ── validate ─────────────────────────────────────────────────

  describe('validate', () => {
    interface Case { name: string; allowedSenders: string; expectError: boolean }
    const cases: Case[] = [
      { name: 'empty senders',              allowedSenders: '',                           expectError: true  },
      { name: 'whitespace only',            allowedSenders: '   ',                        expectError: true  },
      { name: 'valid email',                allowedSenders: 'hr@acme.com',                expectError: false },
      { name: 'valid domain glob',          allowedSenders: '@partner.com',               expectError: false },
      { name: 'multiple valid senders',     allowedSenders: 'hr@acme.com, @partner.com',  expectError: false },
      { name: 'invalid entry (no @)',       allowedSenders: 'not-an-email',               expectError: true  },
      { name: 'mixed valid + invalid',      allowedSenders: 'hr@acme.com, bad',           expectError: true  },
    ];

    it.each(cases)('$name → expectError=$expectError', ({ allowedSenders, expectError }) => {
      const form = baseForm({ emailIngestion: { allowedSenders, subjectFilter: '' } });
      const errs = config.validate(form, t as any);
      if (expectError) {
        expect(errs['emailIngestion.allowedSenders']).toBeDefined();
      } else {
        expect(errs['emailIngestion.allowedSenders']).toBeUndefined();
      }
    });
  });

  // ── isFormValid ───────────────────────────────────────────────

  describe('isFormValid', () => {
    interface Case { name: string; allowedSenders: string; expected: boolean }
    const cases: Case[] = [
      { name: 'empty',                      allowedSenders: '',                           expected: false },
      { name: 'invalid entry',              allowedSenders: 'bad',                        expected: false },
      { name: 'valid single email',         allowedSenders: 'hr@acme.com',                expected: true  },
      { name: 'valid domain glob',          allowedSenders: '@acme.com',                  expected: true  },
      { name: 'multiple valid',             allowedSenders: 'hr@a.com, @b.com',           expected: true  },
      { name: 'one valid, one invalid',     allowedSenders: 'hr@a.com, bad',              expected: false },
    ];

    it.each(cases)('$name → $expected', ({ allowedSenders, expected }) => {
      const form = baseForm({ emailIngestion: { allowedSenders, subjectFilter: '' } });
      expect(config.isFormValid(form)).toBe(expected);
    });
  });

  // ── getActionConfig ───────────────────────────────────────────

  describe('getActionConfig', () => {
    it('splits allowedSenders on comma and trims', () => {
      const form = baseForm({ emailIngestion: { allowedSenders: 'hr@acme.com, @partner.com', subjectFilter: '' } });
      const ac = config.getActionConfig(form);
      expect(ac.action_type).toBe('email_ingestion_connector');
      const cfg = ac.config as { allowed_senders: string[] };
      expect(cfg.allowed_senders).toEqual(['hr@acme.com', '@partner.com']);
    });

    it('omits subject_filter when empty', () => {
      const form = baseForm({ emailIngestion: { allowedSenders: 'hr@acme.com', subjectFilter: '' } });
      const ac = config.getActionConfig(form);
      expect((ac.config as any).subject_filter).toBeUndefined();
    });

    it('includes subject_filter when non-empty', () => {
      const form = baseForm({ emailIngestion: { allowedSenders: 'hr@acme.com', subjectFilter: 'Monthly Roster' } });
      const ac = config.getActionConfig(form);
      expect((ac.config as any).subject_filter).toBe('Monthly Roster');
    });
  });
});
