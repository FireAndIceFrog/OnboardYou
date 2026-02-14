import { createBrowserRouter } from 'react-router-dom';
import { ConfigListProvider } from '@/features/config-list/state';
import { ChatProvider } from '@/features/chat/state';
import { ConfigListScreen } from '@/features/config-list/ui';
import { ConfigDetailsPage } from '@/features/config-details/ui';

export const router = createBrowserRouter([
  {
    path: '/',
    element: (
      <ConfigListProvider>
        <ConfigListScreen />
      </ConfigListProvider>
    ),
  },
  {
    path: '/:configId',
    element: (
      <ChatProvider pipelineConfig={null}>
        <ConfigDetailsPage />
      </ChatProvider>
    ),
  },
]);
