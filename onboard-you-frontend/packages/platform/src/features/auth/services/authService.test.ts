import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  login,
  decodeJwtPayload,
  userFromIdToken,
} from './authService';

describe('authService', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  describe('login', () => {
    it('POSTs email + password and returns tokens on success', async () => {
      const mockTokens = {
        id_token: 'mock-id',
        access_token: 'mock-access',
        refresh_token: 'mock-refresh',
        token_type: 'Bearer',
        expires_in: 3600,
      };

      vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(
        new Response(JSON.stringify(mockTokens), { status: 200 }),
      );

      const result = await login('https://api.example.com', 'a@b.com', 'pass');
      expect(result).toEqual(mockTokens);

      const [url, init] = vi.mocked(fetch).mock.calls[0];
      expect(url).toBe('https://api.example.com/auth/login');
      expect(init?.method).toBe('POST');
      expect(JSON.parse(init?.body as string)).toEqual({ email: 'a@b.com', password: 'pass' });
    });

    it('throws with error message from backend on failure', async () => {
      vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(
        new Response(JSON.stringify({ error: 'Invalid credentials' }), { status: 401 }),
      );

      await expect(login('https://api.example.com', 'a@b.com', 'wrong')).rejects.toThrow(
        'Invalid credentials',
      );
    });

    it('throws generic message when backend returns non-JSON error', async () => {
      vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(
        new Response('Internal Server Error', { status: 500 }),
      );

      await expect(login('https://api.example.com', 'a@b.com', 'pass')).rejects.toThrow(
        'Login failed (500)',
      );
    });
  });

  describe('decodeJwtPayload', () => {
    it('decodes a JWT payload', () => {
      const payload = { sub: 'user-123', email: 'test@example.com' };
      const encoded = btoa(JSON.stringify(payload));
      const fakeJwt = `header.${encoded}.signature`;

      const decoded = decodeJwtPayload(fakeJwt);
      expect(decoded).toEqual(payload);
    });

    it('throws on invalid JWT format', () => {
      expect(() => decodeJwtPayload('not-a-jwt')).toThrow('Invalid JWT format');
    });
  });

  describe('userFromIdToken', () => {
    it('extracts a User from a Cognito ID token', () => {
      const claims = {
        sub: 'user-abc',
        email: 'alice@company.com',
        name: 'Alice',
        'custom:organization_id': 'org-99',
        'custom:role': 'editor',
      };
      const encoded = btoa(JSON.stringify(claims));
      const fakeJwt = `header.${encoded}.signature`;

      const user = userFromIdToken(fakeJwt);
      expect(user.id).toBe('user-abc');
      expect(user.email).toBe('alice@company.com');
      expect(user.name).toBe('Alice');
      expect(user.organizationId).toBe('org-99');
      expect(user.role).toBe('editor');
    });
  });
});
