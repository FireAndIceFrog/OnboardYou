import { Routes, Route } from 'react-router-dom';
import { ConfigListProvider } from '@/features/config-list/state';
import { ChatProvider } from '@/features/chat/state';
import { ConfigListScreen } from '@/features/config-list/ui';
import { ConfigDetailsPage, ConnectionDetailsPage } from '@/features/config-details/ui';

// Global styles — imported here so they load when consumed via Module Federation
import '@/styles/config.scss';
import '@xyflow/react/dist/style.css';

export function ConfigRoutes() {
  return (
    <Routes>
      <Route
        index
        element={
          <ConfigListProvider>
            <ConfigListScreen />
          </ConfigListProvider>
        }
      />
      {/* Step 1: Connection Details (new setup) */}
      <Route
        path="new"
        element={<ConnectionDetailsPage />}
      />
      <Route
        path=":customerCompanyId/connect"
        element={<ConnectionDetailsPage />}
      />
      {/* Step 2: Flow Customization (existing detail page) */}
      <Route
        path=":customerCompanyId/flow"
        element={
          <ChatProvider pipelineConfig={null}>
            <ConfigDetailsPage />
          </ChatProvider>
        }
      />
      {/* Legacy / direct access — redirects to flow */}
      <Route
        path=":customerCompanyId"
        element={
          <ChatProvider pipelineConfig={null}>
            <ConfigDetailsPage />
          </ChatProvider>
        }
      />
    </Routes>
  );
}
