import { describe, it, expect } from 'vitest';
import {
  buildLoginUrl,
  buildLogoutUrl,
  decodeJwtPayload,
  userFromIdToken,
} from './authService';

describe('authService', () => {
  describe('buildLoginUrl', () => {
    it('builds correct Cognito authorize URL with all params', () => {
      const url = buildLoginUrl(
        'https://auth.example.com',
        'my-client-id',
        'https://app.example.com/callback',
      );
      expect(url).toContain('https://auth.example.com/oauth2/authorize?');
      expect(url).toContain('response_type=code');
      expect(url).toContain('client_id=my-client-id');
      expect(url).toContain(encodeURIComponent('https://app.example.com/callback'));
      expect(url).toContain('scope=openid+profile+email');
    });
  });

  describe('buildLogoutUrl', () => {
    it('builds correct Cognito logout URL', () => {
      const url = buildLogoutUrl(
        'https://auth.example.com',
        'my-client-id',
        'https://app.example.com',
      );
      expect(url).toContain('https://auth.example.com/logout?');
      expect(url).toContain('client_id=my-client-id');
      expect(url).toContain('logout_uri=');
      expect(url).toContain(encodeURIComponent('https://app.example.com'));
    });
  });

  describe('decodeJwtPayload', () => {
    it('decodes a JWT payload', () => {
      // Build a fake JWT: header.payload.signature
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
