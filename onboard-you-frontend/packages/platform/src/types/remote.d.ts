declare module 'configApp/App' {
  import type { ComponentType } from 'react';
  const App: ComponentType;
  export default App;
  export const ConfigRoutes: ComponentType;
  /** Standardized export name — all remotes expose Routes */
  export const Routes: ComponentType;
  export function setGlobalValue(value: {
    apiBaseUrl: string;
    showNotification: (message: string, type: 'success' | 'error' | 'warning' | 'info') => void;
    theme: 'light' | 'dark';
  }): void;
}

declare module 'configApp/ConfigListScreen' {
  import type { ComponentType } from 'react';
  const ConfigListScreen: ComponentType;
  export default ConfigListScreen;
}

declare module 'configApp/ConfigDetailsPage' {
  import type { ComponentType } from 'react';
  const ConfigDetailsPage: ComponentType;
  export default ConfigDetailsPage;
}
