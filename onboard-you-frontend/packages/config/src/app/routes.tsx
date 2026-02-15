import { createBrowserRouter } from 'react-router-dom';
import { ErrorBoundary } from '@/shared/ui';
import { ConfigListScreen } from '@/features/config-list/ui';
import { ConfigDetailsPage, ConnectionDetailsPage } from '@/features/config-details/ui';

export const router = createBrowserRouter([
  {
    path: '/',
    element: <ErrorBoundary><ConfigListScreen /></ErrorBoundary>,
  },
  {
    path: '/new',
    element: <ErrorBoundary><ConnectionDetailsPage /></ErrorBoundary>,
  },
  {
    path: '/:customerCompanyId/connect',
    element: <ErrorBoundary><ConnectionDetailsPage /></ErrorBoundary>,
  },
  {
    path: '/:customerCompanyId/flow',
    element: <ErrorBoundary><ConfigDetailsPage /></ErrorBoundary>,
  },
  {
    path: '/:customerCompanyId',
    element: <ErrorBoundary><ConfigDetailsPage /></ErrorBoundary>,
  },
]);
