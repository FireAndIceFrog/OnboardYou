import '@/i18n';
import { Provider } from 'react-redux';
import { Routes, Route } from 'react-router-dom';
import { ChakraProvider } from '@chakra-ui/react';
import { store } from '@/store';
import { system } from '@/theme';
import { ErrorBoundary } from '@/shared/ui';
import { ConfigListScreen } from '@/features/config-list/ui';
import { ConfigDetailsPage, ConnectionDetailsPage } from '@/features/config-details/ui';

// React Flow stylesheet — needed when consumed via Module Federation
import '@xyflow/react/dist/style.css';

export function ConfigRoutes() {
  return (
    <ChakraProvider value={system}>
      <ErrorBoundary>
        <Provider store={store}>
          <Routes>
          <Route index element={<ConfigListScreen />} />
          {/* Step 1: Connection Details (new setup) */}
          <Route path="new" element={<ConnectionDetailsPage />} />
          <Route path=":customerCompanyId/connect" element={<ConnectionDetailsPage />} />
          {/* Step 2: Flow Customization (existing detail page) */}
          <Route path=":customerCompanyId/flow" element={<ConfigDetailsPage />} />
          {/* Legacy / direct access — redirects to flow */}
          <Route path=":customerCompanyId" element={<ConfigDetailsPage />} />
        </Routes>
      </Provider>
    </ErrorBoundary>
  </ChakraProvider>
  );
}
