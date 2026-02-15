import { describe, it, expect } from 'vitest';
import { toApi, fromApi } from './settingsService';
import {
  DEFAULT_EGRESS_SETTINGS,
  DEFAULT_BEARER,
  DEFAULT_OAUTH2,
  DEFAULT_RETRY,
} from '../domain/types';
import type { EgressSettings } from '../domain/types';

describe('settingsService mapping', () => {
  /* ── toApi ──────────────────────────────────────────────── */

  describe('toApi', () => {
    it('maps bearer settings to snake_case API format', () => {
      const settings: EgressSettings = {
        ...DEFAULT_EGRESS_SETTINGS,
        authType: 'bearer',
        bearer: {
          destinationUrl: 'https://api.example.com/employees',
          token: 'sk-test',
          placement: 'custom_header',
          placementKey: 'X-API-Key',
          extraHeaders: { 'Content-Type': 'application/json' },
        },
      };

      const result = toApi(settings);
      expect(result.defaultAuth.auth_type).toBe('bearer');
      expect(result.defaultAuth.destination_url).toBe('https://api.example.com/employees');
      expect(result.defaultAuth.token).toBe('sk-test');
      expect(result.defaultAuth.placement).toBe('custom_header');
      expect(result.defaultAuth.placement_key).toBe('X-API-Key');
      expect(result.defaultAuth.extra_headers).toEqual({ 'Content-Type': 'application/json' });
    });

    it('maps oauth2 settings with scopes as array', () => {
      const settings: EgressSettings = {
        ...DEFAULT_EGRESS_SETTINGS,
        authType: 'oauth2',
        oauth2: {
          destinationUrl: 'https://api.example.com/v2/employees',
          clientId: 'app-123',
          clientSecret: 'secret',
          tokenUrl: 'https://auth.example.com/token',
          scopes: 'read, write',
          grantType: 'client_credentials',
          refreshToken: '',
        },
      };

      const result = toApi(settings);
      expect(result.defaultAuth.auth_type).toBe('oauth2');
      expect(result.defaultAuth.client_id).toBe('app-123');
      expect(result.defaultAuth.scopes).toEqual(['read', 'write']);
    });

    it('includes retry policy in snake_case', () => {
      const result = toApi(DEFAULT_EGRESS_SETTINGS);
      const retry = result.defaultAuth.retry_policy as Record<string, unknown>;
      expect(retry.max_attempts).toBe(3);
      expect(retry.initial_backoff_ms).toBe(1000);
      expect(retry.retryable_status_codes).toEqual([429, 502, 503, 504]);
    });
  });

  /* ── fromApi ────────────────────────────────────────────── */

  describe('fromApi', () => {
    it('maps bearer API response to camelCase frontend type', () => {
      const raw = {
        organizationId: 'org-001',
        defaultAuth: {
          auth_type: 'bearer',
          destination_url: 'https://api.example.com/employees',
          token: 'sk-test',
          placement: 'authorization_header',
          placement_key: 'Authorization',
          extra_headers: {},
          retry_policy: {
            max_attempts: 5,
            initial_backoff_ms: 2000,
            retryable_status_codes: [429, 503],
          },
        },
      };

      const result = fromApi(raw);
      expect(result.authType).toBe('bearer');
      expect(result.bearer.destinationUrl).toBe('https://api.example.com/employees');
      expect(result.bearer.token).toBe('sk-test');
      expect(result.oauth2).toEqual(DEFAULT_OAUTH2);
      expect(result.retryPolicy.maxAttempts).toBe(5);
      expect(result.retryPolicy.retryableStatusCodes).toEqual([429, 503]);
    });

    it('maps oauth2 API response and joins scopes', () => {
      const raw = {
        organizationId: 'org-001',
        defaultAuth: {
          auth_type: 'oauth2',
          destination_url: 'https://api.example.com/v2/employees',
          client_id: 'app-123',
          client_secret: 'secret',
          token_url: 'https://auth.example.com/token',
          scopes: ['read', 'write'],
          grant_type: 'client_credentials',
        },
      };

      const result = fromApi(raw);
      expect(result.authType).toBe('oauth2');
      expect(result.oauth2.clientId).toBe('app-123');
      expect(result.oauth2.scopes).toBe('read, write');
      expect(result.bearer).toEqual(DEFAULT_BEARER);
      expect(result.retryPolicy).toEqual(DEFAULT_RETRY);
    });

    it('defaults to bearer when auth_type is unknown', () => {
      const raw = {
        defaultAuth: { destination_url: 'https://example.com' },
      };

      const result = fromApi(raw);
      expect(result.authType).toBe('bearer');
    });

    it('round-trips bearer settings through toApi → fromApi', () => {
      const original: EgressSettings = {
        authType: 'bearer',
        bearer: {
          destinationUrl: 'https://api.example.com/employees',
          token: 'sk-live-abc',
          placement: 'authorization_header',
          placementKey: 'Authorization',
          extraHeaders: {},
        },
        oauth2: DEFAULT_OAUTH2,
        retryPolicy: { maxAttempts: 3, initialBackoffMs: 1000, retryableStatusCodes: [429, 502] },
      };

      const apiPayload = toApi(original);
      const roundTripped = fromApi(apiPayload);
      expect(roundTripped).toEqual(original);
    });
  });
});
