import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { client } from '../env.js';
import type { OrgSettings } from '../models/org-settings.js';
import type { PipelineConfig } from '../models/pipeline-config.js';
import type { ValidationResult } from '../models/validation.js';

/**
 * Smoke tests for the "default auth" flow:
 *
 * 1. Save org settings with a bearer auth config.
 * 2. Create a pipeline config whose egress uses `{ auth_type: "default" }`.
 * 3. Validate that pipeline — the API should propagate columns through
 *    the api_dispatcher step without error (it's a pass-through for columns).
 * 4. Clean up by deleting the config.
 */

beforeAll(async () => {
  await client.login();
});

const prefix = `smoke-default-auth-${Date.now()}`;
const companyId = prefix;

afterAll(async () => {
  await client.deleteConfigsWithPrefix(prefix);
  await client.deleteConfigsWithPrefix("Default");
});

describe('Default auth end-to-end', () => {
  it('saves org settings with a bearer auth config', async () => {
    const { status, body } = await client.put<OrgSettings>('/settings', {
      defaultAuth: {
        auth_type: 'bearer',
        destination_url: 'https://httpbin.org/post',
        token: 'smoke-test-token',
        schema: {
          employee_id: 'string',
          cellphone: 'string',
          first_name: 'string',
          country: 'string',
          country_code: 'string',
          international_phone: 'string',
        },
      },
    });

    expect(status).toBe(200);
    expect((body.defaultAuth as Record<string, unknown>).auth_type).toBe('bearer');
  });

  it('creates a config with api_dispatcher using auth_type default', async () => {
    const payload: Partial<PipelineConfig> = {
      name: 'Default Auth Smoke',
      cron: 'rate(1 day)',
      pipeline: {
        version: '1.0',
        actions: [
          {
            id: 'ingest',
            action_type: 'generic_ingestion_connector',
            config: {
              filename: 'employees.csv',
              columns: ['employee_id', 'cellphone', 'first_name', 'country'],
            },
          },
          {
            id: 'egress',
            action_type: 'api_dispatcher',
            config: { auth_type: 'default' },
          },
        ],
      },
    };

    const { status, body } = await client.post<PipelineConfig>(
      `/config/${companyId}`,
      payload,
    );

    expect(status).toBe(200);
    expect(body.customerCompanyId).toBe(companyId);
  });

  it('validates the pipeline with default auth successfully', async () => {
    const { status, body } = await client.post<ValidationResult>(
      `/config/${companyId}/validate`,
      {
        name: 'Default Auth Smoke',
        cron: 'rate(1 day)',
        pipeline: {
          version: '1.0',
          actions: [
            {
              id: 'ingest',
              action_type: 'generic_ingestion_connector',
              config: {
                filename: 'employees.csv',
                columns: ['employee_id', 'cellphone', 'first_name', 'country'],
              },
            },
            {
              id: 'egress',
              action_type: 'api_dispatcher',
              config: { auth_type: 'default' },
            },
          ],
        },
      },
    );

    expect(status).toBe(200);
    expect(body.steps).toHaveLength(2);

    // api_dispatcher is a pass-through — columns unchanged
    expect(body.steps[1].action_id).toBe('egress');
    expect(body.steps[1].columns_after).toEqual(
      expect.arrayContaining(['employee_id', 'cellphone', 'first_name', 'country']),
    );
    expect(body.final_columns).toEqual(body.steps[1].columns_after);
  });

});
