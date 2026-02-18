import '@/i18n';
import { Provider } from 'react-redux';
import { RouterProvider } from 'react-router-dom';
import { ChakraProvider } from '@chakra-ui/react';
import { store } from '@/store';
import { system } from '@/theme';
import { router } from './routes';
import { ErrorBoundary } from '@/shared/ui';
import '@xyflow/react/dist/style.css';

export default function App() {
  return (
    <ErrorBoundary>
      <ChakraProvider value={system}>
        <Provider store={store}>
          <RouterProvider router={router} />
        </Provider>
      </ChakraProvider>
    </ErrorBoundary>
  );
}

// Export for Module Federation — renders inside host's router context
export { ConfigRoutes } from './ConfigRoutes';
export { ConfigRoutes as Routes } from './ConfigRoutes';
export { setGlobalValue } from '@/shared/hooks';
