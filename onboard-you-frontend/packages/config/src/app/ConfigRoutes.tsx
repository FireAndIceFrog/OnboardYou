import { Routes, Route } from 'react-router-dom';
import { ConfigListProvider } from '@/features/config-list/state';
import { ChatProvider } from '@/features/chat/state';
import { ConfigListScreen } from '@/features/config-list/ui';
import { ConfigDetailsPage } from '@/features/config-details/ui';

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
      <Route
        path=":configId"
        element={
          <ChatProvider pipelineConfig={null}>
            <ConfigDetailsPage />
          </ChatProvider>
        }
      />
    </Routes>
  );
}
