/* ── API Dispatcher egress types ──────────────────────────── */

export type AuthType = 'bearer' | 'oauth2';

export interface BearerConfig {
  destinationUrl: string;
  token: string;
  placement: 'authorization_header' | 'custom_header' | 'query_param';
  placementKey: string;
  extraHeaders: Record<string, string>;
  /**
   * Dynamic schema for the API payload sent to the egress endpoint.  Keys are
   * field names and values are their types (e.g. "id": "string").  Stored
   * alongside the auth object so the backend can generate a body dynamically.
   */
  schema: Record<string, string>;
  /** Optional JSON‑pointer (dot/bracket) to the location within the
   * generated object that should be used as the request body. */
  bodyPath: string;
}

export interface OAuth2Config {
  destinationUrl: string;
  clientId: string;
  clientSecret: string;
  tokenUrl: string;
  scopes: string;
  grantType: 'client_credentials' | 'authorization_code';
  refreshToken: string;
  /** dynamic schema similar to {@link BearerConfig.schema} */
  schema: Record<string, string>;
  /** Optional pointer into the generated object for the request body */
  bodyPath: string;
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
  schema: {},
  bodyPath: '',
};

export const DEFAULT_OAUTH2: OAuth2Config = {
  destinationUrl: '',
  clientId: '',
  clientSecret: '',
  tokenUrl: '',
  scopes: '',
  grantType: 'client_credentials',
  refreshToken: '',
  schema: {},
  bodyPath: '',
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

export const FIELD_TYPE_OPTIONS = [
  { value: 'string', label: 'String' },
  { value: 'number', label: 'Number' },
  { value: 'boolean', label: 'Boolean' },
] as const;

export type SchemaFieldType = (typeof FIELD_TYPE_OPTIONS)[number]['value'];
