
/* ── Types ─────────────────────────────────────────────────── */

import { ComponentType, LazyExoticComponent } from "react";

export interface RemotePackageConfig {
  package: string;
  path: string;
  useGlobal: boolean;
  version: number;
  entry: () => Promise<RemoteModule>;
}

export type SetGlobalValueFn = (value: {
  apiClient: unknown;
  showNotification: (message: string, type: 'success' | 'error' | 'warning' | 'info') => void;
  theme: 'light' | 'dark';
}) => void;

export interface RemoteModule {
  Routes: ComponentType;
  setGlobalValue?: SetGlobalValueFn;
}

export interface RemoteHandle {
  Component: LazyExoticComponent<ComponentType>;
  getSetGlobalValue: () => SetGlobalValueFn | null;
  injectGlobals: boolean;
}