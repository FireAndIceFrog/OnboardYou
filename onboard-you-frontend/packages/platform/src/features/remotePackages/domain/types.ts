
/* ── Types ─────────────────────────────────────────────────── */

import { ComponentType, LazyExoticComponent } from "react";

export interface RemotePackageConfig {
  package: string;
  path: string;
  useGlobal: boolean;
  version: number;
  translations?: () => Promise<any>;
  entry: () => Promise<RemoteModule>;
}

export type SetGlobalValueFn = (value: {
  apiBaseUrl: string;
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