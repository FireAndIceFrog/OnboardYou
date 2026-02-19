import { Provider } from 'react-redux';
import { RouterProvider } from 'react-router-dom';
import { ChakraProvider } from '@chakra-ui/react';
import { store } from '@/store';
import { initAuth } from '@/features/auth/state/authSlice';
import { configureApiClient } from '@/shared/services/configureApiClient';
import { ErrorBoundary } from '@/shared/ui/ErrorBoundary';
import { system } from '@/theme';
import { router } from './routes';

configureApiClient();
store.dispatch(initAuth());

export default function App() {
  return (
    <ErrorBoundary>
      <Provider store={store}>
        <ChakraProvider value={system}>
          <RouterProvider router={router} />
        </ChakraProvider>
      </Provider>
    </ErrorBoundary>
  );
}
