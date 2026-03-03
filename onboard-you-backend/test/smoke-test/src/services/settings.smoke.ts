import { describe, it, expect, beforeAll } from 'vitest';
import { client } from '../env.js';
import type { OrgSettings } from '../models/org-settings.js';

beforeAll(async () => {
  await client.login();
});

describe('PUT /settings', () => {
  it('upserts organization settings', async () => {
    const payload: Partial<OrgSettings> = {
      defaultAuth: {
        auth_type: 'bearer',
        destination_url: 'https://httpbin.org/post',
        token: 'smoke-test-token',
        schema: { foo: 'string' },
        body_path: 'foo',
      },
    };

    const { status, body } = await client.put<OrgSettings>('/settings', payload);

    expect(status).toBe(200);
    expect(body.organizationId).toBeTruthy();
    const respAuth = body.defaultAuth as Record<string, unknown>;
    expect(respAuth.auth_type).toBe('bearer');
    expect(respAuth.schema).toEqual({ foo: 'string' });
    expect(respAuth.body_path).toBe('foo');
  });
});

describe('GET /settings', () => {
  it('reads settings back', async () => {
    const { status, body } = await client.get<OrgSettings>('/settings');

    expect(status).toBe(200);
    expect(body.defaultAuth).toBeTruthy();
  });
});
