import { Suspense, lazy } from 'react';
import { createBrowserRouter } from 'react-router-dom';
import { LoginPage, CallbackPage, ProtectedRoute } from '@/features/auth';
import { AppLayout } from '@/features/layout';
import { HomeScreen } from '@/features/home';
import { SettingsPage } from '@/features/settings';
import { Spinner } from '@/shared/ui/Spinner';

const ConfigApp = lazy(async () => {
  const m = await import('configApp/App');
  return { default: m.ConfigRoutes };
});

function ConfigRemote() {
  return (
    <Suspense
      fallback={
        <div style={{ display: 'flex', justifyContent: 'center', padding: '4rem' }}>
          <Spinner size="lg" />
        </div>
      }
    >
      <ConfigApp />
    </Suspense>
  );
}

export const router = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },
  {
    path: '/callback',
    element: <CallbackPage />,
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
          {
            path: 'config/*',
            element: <ConfigRemote />,
          },
          {
            path: 'settings',
            element: <SettingsPage />,
          },
        ],
      },
    ],
  },
]);
