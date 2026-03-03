import { getSettings as getSettingsApi, upsertSettings as upsertSettingsApi } from '@/generated/api';
import type {
  EgressSettings,
  AuthType,
  BearerConfig,
  OAuth2Config,
} from '../domain/types';
import {
  DEFAULT_BEARER,
  DEFAULT_OAUTH2,
  DEFAULT_RETRY,
} from '../domain/types';

/* ── API payload shape ─────────────────────────────────────── */

/** Wire format returned by GET /settings and sent to PUT /settings. */
export interface OrgSettingsPayload {
  organizationId?: string;
  defaultAuth: Record<string, unknown>;
}

/* ── Mapping: frontend → API (camelCase → snake_case) ──────── */

export function toApi(settings: EgressSettings): OrgSettingsPayload {
  const base: Record<string, unknown> = {
    auth_type: settings.authType,
  };

  if (settings.authType === 'bearer') {
    base.destination_url = settings.bearer.destinationUrl;
    base.token = settings.bearer.token;
    base.placement = settings.bearer.placement;
    base.placement_key = settings.bearer.placementKey;
    base.extra_headers = settings.bearer.extraHeaders;
    if (Object.keys(settings.bearer.schema).length > 0) {
      base.schema = settings.bearer.schema;
    }
    if (settings.bearer.bodyPath) {
      base.body_path = settings.bearer.bodyPath;
    }
  } else {
    base.destination_url = settings.oauth2.destinationUrl;
    base.client_id = settings.oauth2.clientId;
    base.client_secret = settings.oauth2.clientSecret;
    base.token_url = settings.oauth2.tokenUrl;
    base.scopes = settings.oauth2.scopes
      ? settings.oauth2.scopes
          .split(',')
          .map((s) => s.trim())
          .filter(Boolean)
      : [];
    base.grant_type = settings.oauth2.grantType;
    if (settings.oauth2.refreshToken) {
      base.refresh_token = settings.oauth2.refreshToken;
    }
    if (Object.keys(settings.oauth2.schema).length > 0) {
      base.schema = settings.oauth2.schema;
    }
    if (settings.oauth2.bodyPath) {
      base.body_path = settings.oauth2.bodyPath;
    }
  }

  base.retry_policy = {
    max_attempts: settings.retryPolicy.maxAttempts,
    initial_backoff_ms: settings.retryPolicy.initialBackoffMs,
    retryable_status_codes: settings.retryPolicy.retryableStatusCodes,
  };

  return { defaultAuth: base };
}

/* ── Mapping: API → frontend (snake_case → camelCase) ──────── */

export function fromApi(raw: OrgSettingsPayload): EgressSettings {
  const auth = raw.defaultAuth;
  const authType: AuthType = auth.auth_type === 'oauth2' ? 'oauth2' : 'bearer';
  const retryRaw = auth.retry_policy as Record<string, unknown> | undefined;

  return {
    authType,
    bearer:
      authType === 'bearer'
        ? {
            destinationUrl: (auth.destination_url as string) ?? '',
            token: (auth.token as string) ?? '',
            placement:
              (auth.placement as BearerConfig['placement']) ?? 'authorization_header',
            placementKey: (auth.placement_key as string) ?? 'Authorization',
            extraHeaders: (auth.extra_headers as Record<string, string>) ?? {},
            schema: (auth.schema as Record<string, string>) ?? {},
            bodyPath: (auth.body_path as string) ?? '',
          }
        : DEFAULT_BEARER,
    oauth2:
      authType === 'oauth2'
        ? {
            destinationUrl: (auth.destination_url as string) ?? '',
            clientId: (auth.client_id as string) ?? '',
            clientSecret: (auth.client_secret as string) ?? '',
            tokenUrl: (auth.token_url as string) ?? '',
            scopes: Array.isArray(auth.scopes)
              ? (auth.scopes as string[]).join(', ')
              : '',
            grantType:
              (auth.grant_type as OAuth2Config['grantType']) ?? 'client_credentials',
            refreshToken: (auth.refresh_token as string) ?? '',
            schema: (auth.schema as Record<string, string>) ?? {},
            bodyPath: (auth.body_path as string) ?? '',
          }
        : DEFAULT_OAUTH2,
    retryPolicy: {
      maxAttempts:
        (retryRaw?.max_attempts as number) ?? DEFAULT_RETRY.maxAttempts,
      initialBackoffMs:
        (retryRaw?.initial_backoff_ms as number) ?? DEFAULT_RETRY.initialBackoffMs,
      retryableStatusCodes:
        (retryRaw?.retryable_status_codes as number[]) ??
        DEFAULT_RETRY.retryableStatusCodes,
    },
  };
}

/* ── API calls ─────────────────────────────────────────────── */

export async function fetchSettings(): Promise<EgressSettings> {
  const { data } = await getSettingsApi({ throwOnError: true });
  return fromApi({ defaultAuth: data.defaultAuth as Record<string, unknown> });
}

export async function saveSettings(
  settings: EgressSettings,
): Promise<EgressSettings> {
  const body = toApi(settings);
  const { data } = await upsertSettingsApi({
    body: { defaultAuth: body.defaultAuth },
    throwOnError: true,
  });
  return fromApi({ defaultAuth: data.defaultAuth as Record<string, unknown> });
}
