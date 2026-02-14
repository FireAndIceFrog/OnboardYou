import { createBrowserRouter, type RouteObject } from 'react-router-dom';
import { ConfigListScreen } from '@/features/config-list';
import { ConfigDetailsPage } from '@/features/config-details';

const routes: RouteObject[] = [
  {
    path: '/',
    element: <ConfigListScreen />,
  },
  {
    path: '/:configId',
    element: <ConfigDetailsPage />,
  },
];

export const router = createBrowserRouter(routes);
