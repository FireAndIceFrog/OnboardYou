import { describe, it, expect, beforeAll } from 'vitest';
import { client } from '../env.js';
import type { PipelineConfig } from '../models/pipeline-config.js';

beforeAll(async () => {
  await client.login();
});

const testId = `smoke-test-${Date.now()}`;

describe('GET /config', () => {
  it('lists pipeline configs', async () => {
    const { status, body } = await client.get<PipelineConfig[]>('/config');

    expect(status).toBe(200);
    expect(Array.isArray(body)).toBe(true);
  });
});

describe('POST /config/{id}', () => {
  it('creates a pipeline config', async () => {
    const payload: Partial<PipelineConfig> = {
      name: 'Smoke Test Pipeline',
      cron: 'rate(1 day)',
      pipeline: { version: '1.0', actions: [] },
    };

    const { status, body } = await client.post<PipelineConfig>(`/config/${testId}`, payload);

    expect(status).toBe(200);
    expect(body.name).toBe('Smoke Test Pipeline');
    expect(body.customerCompanyId).toBe(testId);
    expect(body.lastEdited).toBeTruthy();
  });
});

describe('GET /config/{id}', () => {
  it('reads the created config back', async () => {
    const { status, body } = await client.get<PipelineConfig>(`/config/${testId}`);

    expect(status).toBe(200);
    expect(body.name).toBe('Smoke Test Pipeline');
  });

  it('returns 404 for a non-existent config', async () => {
    const { status } = await client.get('/config/does-not-exist-999');

    expect(status).toBe(404);
  });
});

describe('PUT /config/{id}', () => {
  it('updates an existing config', async () => {
    const payload: Partial<PipelineConfig> = {
      name: 'Smoke Test Pipeline (updated)',
      cron: 'rate(1 day)',
      pipeline: { version: '1.0', actions: [] },
    };

    const { status, body } = await client.put<PipelineConfig>(`/config/${testId}`, payload);

    expect(status).toBe(200);
    expect(body.name).toBe('Smoke Test Pipeline (updated)');
  });
});
