import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { client } from '../env.js';
import type { PipelineConfig } from '../models/pipeline-config.js';

beforeAll(async () => {
  await client.login();
});

const companyId = `plan-gen-smoke-${Date.now()}`;

/**
 * Smoke test for the plan generation pipeline.
 *
 * Creates a Workday ingress + api_dispatcher egress pipeline with a
 * destination schema expecting phone_number, country, first_name, last_name.
 * Triggers plan generation and polls until the LLM response is written back.
 */
describe('POST /config/{id}/generate-plan', () => {
  // ── Setup: create a pipeline config with Workday → ApiDispatcher ────

  beforeAll(async () => {
    const payload: Partial<PipelineConfig> = {
      name: 'Plan Generation Smoke Test',
      cron: 'rate(1 day)',
      pipeline: {
        version: '1.0',
        actions: [
          {
            id: 'ingest',
            action_type: 'workday_hris_connector',
            config: {
              tenant_url: 'https://wd3-impl-services1.workday.com',
              tenant_id: 'smoke_test_tenant',
              username: 'ISU_Smoke',
              password: 'env:WORKDAY_PASSWORD',
            },
          },
          {
            id: 'egress',
            action_type: 'api_dispatcher',
            config: {
              auth_type: 'default',
              schema: {
                first_name: 'firstName',
                last_name: 'lastName',
                phone_number: 'phoneNumber',
                country: 'countryCode',
              },
            },
          },
        ],
      },
    };

    const { status } = await client.post<PipelineConfig>(
      `/config/${companyId}`,
      payload,
    );
    expect(status).toBe(200);
  });

  // ── Teardown ────────────────────────────────────────────────────────

  afterAll(async () => {
    await client.delete(`/config/${companyId}`);
  });

  // ── Tests ───────────────────────────────────────────────────────────

  it('triggers plan generation and returns 202', async () => {
    const { status, body } = await client.post<{ status: string }>(
      `/config/${companyId}/generate-plan`,
      { sourceSystem: 'Workday' },
    );

    expect(status).toBe(202);
    expect(body.status).toBe('InProgress');
  });

  it('is idempotent while in progress', async () => {
    const { status, body } = await client.post<{ status: string }>(
      `/config/${companyId}/generate-plan`,
      { sourceSystem: 'Workday' },
    );

    // Should return 202 without queuing a second job
    expect(status).toBe(202);
    expect(body.status).toBe('InProgress');
  });

  it('completes plan generation within 60 seconds', async () => {
    const maxWaitMs = 60_000;
    const pollIntervalMs = 3_000;
    const start = Date.now();

    let config: PipelineConfig | null = null;

    while (Date.now() - start < maxWaitMs) {
      const { status, body } = await client.get<PipelineConfig>(
        `/config/${companyId}`,
      );
      expect(status).toBe(200);

      if (body.planSummary?.generationStatus === 'completed') {
        config = body;
        break;
      }

      await new Promise((r) => setTimeout(r, pollIntervalMs));
    }

    expect(config).not.toBeNull();
    const summary = config!.planSummary!;

    // Plan summary was generated
    expect(summary.generationStatus).toBe('completed');
    expect(summary.headline).toBeTruthy();
    expect(typeof summary.headline).toBe('string');
    expect(summary.description).toBeTruthy();

    // Features array is populated
    expect(Array.isArray(summary.features)).toBe(true);
    expect(summary.features.length).toBeGreaterThan(0);

    // Preview has before/after data
    expect(summary.preview.sourceLabel).toBeTruthy();
    expect(summary.preview.targetLabel).toBeTruthy();
    expect(summary.preview.before).toBeTruthy();
    expect(summary.preview.after).toBeTruthy();
  }, 90_000); // 90s vitest timeout to cover the 60s polling window
});
