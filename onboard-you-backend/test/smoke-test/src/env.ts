// ── Shared environment loader ───────────────────────────────

import 'dotenv/config';
import { ApiClient } from './repositories/api-client.js';

function requireEnv(name: string): string {
  const value = process.env[name];
  if (!value) throw new Error(`Missing required env var: ${name}`);
  return value;
}

export const client = new ApiClient({
  baseUrl:  requireEnv('API_BASE_URL'),
  email:    requireEnv('SMOKE_EMAIL'),
  password: requireEnv('SMOKE_PASSWORD'),
});
