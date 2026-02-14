/* ── API Dispatcher egress types ──────────────────────────── */

export type AuthType = 'bearer' | 'oauth2';

export interface BearerConfig {
  destinationUrl: string;
  token: string;
  placement: 'authorization_header' | 'custom_header' | 'query_param';
  placementKey: string;
  extraHeaders: Record<string, string>;
}

export interface OAuth2Config {
  destinationUrl: string;
  clientId: string;
  clientSecret: string;
  tokenUrl: string;
  scopes: string;
  grantType: 'client_credentials' | 'authorization_code';
  refreshToken: string;
}

export interface RetryPolicy {
  maxAttempts: number;
  initialBackoffMs: number;
  retryableStatusCodes: number[];
}

export interface EgressSettings {
  authType: AuthType;
  bearer: BearerConfig;
  oauth2: OAuth2Config;
  retryPolicy: RetryPolicy;
}

export const DEFAULT_BEARER: BearerConfig = {
  destinationUrl: '',
  token: '',
  placement: 'authorization_header',
  placementKey: 'Authorization',
  extraHeaders: {},
};

export const DEFAULT_OAUTH2: OAuth2Config = {
  destinationUrl: '',
  clientId: '',
  clientSecret: '',
  tokenUrl: '',
  scopes: '',
  grantType: 'client_credentials',
  refreshToken: '',
};

export const DEFAULT_RETRY: RetryPolicy = {
  maxAttempts: 3,
  initialBackoffMs: 1000,
  retryableStatusCodes: [429, 502, 503, 504],
};

export const DEFAULT_EGRESS_SETTINGS: EgressSettings = {
  authType: 'bearer',
  bearer: DEFAULT_BEARER,
  oauth2: DEFAULT_OAUTH2,
  retryPolicy: DEFAULT_RETRY,
};

export const PLACEMENT_OPTIONS = [
  { value: 'authorization_header', label: 'Authorization Header' },
  { value: 'custom_header', label: 'Custom Header' },
  { value: 'query_param', label: 'Query Parameter' },
] as const;

export const GRANT_TYPE_OPTIONS = [
  { value: 'client_credentials', label: 'Client Credentials' },
  { value: 'authorization_code', label: 'Authorization Code' },
] as const;
