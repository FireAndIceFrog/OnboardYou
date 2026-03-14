import { describe, it, expect, beforeAll } from 'vitest';
import { client } from '../env.js';
import type { ValidationResult } from '../models/validation.js';

beforeAll(async () => {
  await client.login();
});

const companyId = `smoke-validate-${Date.now()}`; // dry-run endpoint; no configs are persisted

describe('POST /config/{id}/validate', () => {
  it('validates an empty pipeline', async () => {
    const { status, body } = await client.post<ValidationResult>(
      `/config/${companyId}/validate`,
      {
        name: 'Empty Pipeline',
        cron: 'rate(1 day)',
        pipeline: { version: '1.0', actions: [] },
      },
    );

    expect(status).toBe(200);
    expect(body.steps).toEqual([]);
    expect(body.final_columns).toEqual([]);
  });

  it('propagates columns through a CSV → drop pipeline', async () => {
    const { status, body } = await client.post<ValidationResult>(
      `/config/${companyId}/validate`,
      {
        name: 'CSV + Drop',
        cron: 'rate(1 day)',
        pipeline: {
          version: '1.0',
          actions: [
            {
              id: 'ingest',
              action_type: 'csv_hris_connector',
              config: {
                filename: 'employees.csv',
                columns: ['employee_id', 'first_name', 'last_name', 'email', 'ssn'],
              },
            },
            {
              id: 'drop_ssn',
              action_type: 'drop_column',
              config: { columns: ['ssn'] },
            },
          ],
        },
      },
    );

    expect(status).toBe(200);
    expect(body.steps).toHaveLength(2);

    // After CSV ingestion: all 5 columns present
    expect(body.steps[0].action_id).toBe('ingest');
    expect(body.steps[0].columns_after).toEqual(
      expect.arrayContaining(['employee_id', 'first_name', 'last_name', 'email', 'ssn']),
    );

    // After drop_column: ssn removed
    expect(body.steps[1].action_id).toBe('drop_ssn');
    expect(body.steps[1].columns_after).not.toContain('ssn');
    expect(body.steps[1].columns_after).toContain('employee_id');

    // final_columns matches last step
    expect(body.final_columns).toEqual(body.steps[1].columns_after);
  });

  it('returns 400 for a misconfigured action', async () => {
    const { status } = await client.post<unknown>(
      `/config/${companyId}/validate`,
      {
        name: 'Bad Config',
        cron: 'rate(1 day)',
        pipeline: {
          version: '1.0',
          actions: [
            {
              id: 'bad',
              action_type: 'csv_hris_connector',
              config: { filename: 'data.csv', columns: [] },
            },
          ],
        },
      },
    );

    expect(status).toBe(400);
  });

  it('propagates columns through a multi-step pipeline', async () => {
    const { status, body } = await client.post<ValidationResult>(
      `/config/${companyId}/validate`,
      {
        name: 'Multi-step',
        cron: 'rate(1 day)',
        pipeline: {
          version: '1.0',
          actions: [
            {
              id: 'ingest',
              action_type: 'csv_hris_connector',
              config: {
                filename: 'employees.csv',
                columns: ['employee_id', 'first_name', 'last_name', 'email'],
              },
            },
            {
              id: 'rename',
              action_type: 'rename_column',
              config: { mapping: { first_name: 'given_name' } },
            },
            {
              id: 'drop',
              action_type: 'drop_column',
              config: { columns: ['last_name'] },
            },
          ],
        },
      },
    );

    expect(status).toBe(200);
    expect(body.steps).toHaveLength(3);

    // After rename: first_name → given_name
    expect(body.steps[1].columns_after).toContain('given_name');
    expect(body.steps[1].columns_after).not.toContain('first_name');

    // After drop: last_name gone
    expect(body.final_columns).toContain('given_name');
    expect(body.final_columns).toContain('employee_id');
    expect(body.final_columns).not.toContain('last_name');
  });
});
