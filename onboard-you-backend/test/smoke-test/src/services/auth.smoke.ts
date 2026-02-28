import { describe, it, expect, beforeAll } from 'vitest';
import { client } from '../env.js';

beforeAll(async () => {
  await client.login();
});

describe('POST /auth/login', () => {
  it('returns a valid token set', async () => {
    const tokens = await client.login();

    expect(tokens.id_token).toBeTruthy();
    expect(tokens.access_token).toBeTruthy();
    expect(tokens.token_type).toBe('Bearer');
    expect(tokens.expires_in).toBeGreaterThan(0);
  });

  it('rejects invalid credentials', async () => {
    const res = await fetch(`${(client as any).baseUrl}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email: 'nobody@example.com', password: 'wrong' }),
    });

    expect(res.status).toBe(401);
  });
});
