declare module 'configApp/App' {
  import type { ComponentType } from 'react';
  const App: ComponentType;
  export default App;
  export const ConfigRoutes: ComponentType;
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
