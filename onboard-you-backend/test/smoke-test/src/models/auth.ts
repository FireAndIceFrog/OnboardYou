// ── Auth DTOs ───────────────────────────────────────────────

/** POST /auth/login request body. */
export interface LoginRequest {
  /** User email address (Cognito username). */
  email: string;
  /** User password. */
  password: string;
}

/** Successful login response — Cognito token set. */
export interface LoginResponse {
  /** JWT ID token (contains custom claims such as organizationId). */
  id_token: string;
  /** JWT access token. */
  access_token: string;
  /** Refresh token (can be exchanged for new tokens). */
  refresh_token?: string;
  /** Token type — always "Bearer". */
  token_type: string;
  /** Token lifetime in seconds. */
  expires_in: number;
}
