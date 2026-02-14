// ============================================================================
// OnboardYou — Shared Types
// ============================================================================

/** Supported user roles within an organization. */
export type UserRole = 'owner' | 'admin' | 'member' | 'viewer';

/** Subscription plan tiers. */
export type PlanTier = 'free' | 'starter' | 'professional' | 'enterprise';

/** UI theme options. */
export type Theme = 'light' | 'dark';

// ---------------------------------------------------------------------------
// Domain Models
// ---------------------------------------------------------------------------

/** Authenticated user profile. */
export interface User {
  id: string;
  email: string;
  name: string;
  organizationId: string;
  role: UserRole;
}

/** Organization / tenant. */
export interface Organization {
  id: string;
  name: string;
  plan: PlanTier;
}

// ---------------------------------------------------------------------------
// Auth State
// ---------------------------------------------------------------------------

/** Authentication slice managed by Zustand. */
export interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  token: string | null;
}

// ---------------------------------------------------------------------------
// Global State
// ---------------------------------------------------------------------------

/** Top-level application state shape. */
export interface GlobalState {
  auth: AuthState;
  organization: Organization | null;
  sidebarOpen: boolean;
  theme: Theme;
}

// ---------------------------------------------------------------------------
// API
// ---------------------------------------------------------------------------

/** Standard error response returned by the backend API. */
export interface ApiError {
  statusCode: number;
  message: string;
  details?: string;
}
