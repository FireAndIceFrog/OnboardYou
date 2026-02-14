import { createContext } from 'react';
import type { AuthState } from '@/features/auth/domain/types';

export interface AuthContextValue {
  state: AuthState;
  login: () => void;
  logout: () => void;
  getToken: () => string | null;
  exchangeCode: (code: string) => Promise<void>;
}

export const AuthContext = createContext<AuthContextValue | null>(null);
