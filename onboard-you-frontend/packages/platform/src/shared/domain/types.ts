export type UserRole = 'admin' | 'editor' | 'viewer';
export type Theme = 'light' | 'dark';
export type NotificationType = 'success' | 'error' | 'warning' | 'info';

export interface User {
  id: string;
  email: string;
  name: string;
  organizationId: string;
  role: UserRole;
}

export interface Organization {
  id: string;
  name: string;
  plan: 'starter' | 'professional' | 'enterprise';
}

export interface Notification {
  id: string;
  message: string;
  type: NotificationType;
  timestamp: number;
}

export interface ApiErrorResponse {
  statusCode: number;
  message: string;
  details?: string;
}
