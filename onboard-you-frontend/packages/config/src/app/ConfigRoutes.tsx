import '@/i18n';
import { Provider } from 'react-redux';
import { Routes, Route } from 'react-router-dom';
import { store } from '@/store';
import { ErrorBoundary } from '@/shared/ui';
import { ConfigListScreen } from '@/features/config-list/ui';
import { ConfigDetailsPage, ConnectionDetailsPage } from '@/features/config-details/ui';

// Global styles — imported here so they load when consumed via Module Federation
import '@/styles/config.scss';
import '@xyflow/react/dist/style.css';

export function ConfigRoutes() {
  return (
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
  );
}
