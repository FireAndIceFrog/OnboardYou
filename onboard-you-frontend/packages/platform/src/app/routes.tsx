import { createBrowserRouter } from 'react-router-dom';
import { LoginPage, ProtectedRoute } from '@/features/auth';
import { AppLayout } from '@/features/layout';
import { HomeScreen } from '@/features/home';
import { SettingsPage } from '@/features/settings';
import { buildRemoteRoutes } from '@/features/remotePackages';

export const router = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },
  {
    element: <ProtectedRoute />,
    children: [
      {
        element: <AppLayout />,
        children: [
          {
            index: true,
            element: <HomeScreen />,
          },
          ...buildRemoteRoutes(),
          {
            path: 'settings',
            element: <SettingsPage />,
          },
        ],
      },
    ],
  },
]);
