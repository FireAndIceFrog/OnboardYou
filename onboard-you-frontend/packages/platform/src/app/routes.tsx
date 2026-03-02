import { createBrowserRouter } from 'react-router-dom';
import { LoginPage, ProtectedRoute } from '@/features/auth';
import { AppLayout } from '@/features/layout';
import { HomeScreen } from '@/features/home';
import { SettingsPage } from '@/features/settings';
import { buildRemoteRoutes } from '@/features/remotePackages';

// Vite sets import.meta.env.BASE_URL from the `base` config.
// For GitHub Pages (/OnboardYou/) this tells React Router where app is mounted.
const basename = import.meta.env.BASE_URL.replace(/\/$/, '') || '/';

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
], { basename });
