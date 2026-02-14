import { Suspense, lazy } from 'react';
import { createBrowserRouter } from 'react-router-dom';
import { LoginPage, CallbackPage, ProtectedRoute } from '@/features/auth';
import { AppLayout } from '@/features/layout';
import { HomeScreen } from '@/features/home';
import { Spinner } from '@/shared/ui/Spinner';

const ConfigApp = lazy(() => import('configApp/App'));

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

function SettingsPlaceholder() {
  return (
    <div style={{ padding: '2rem' }}>
      <h2>Settings</h2>
      <p style={{ color: '#64748B', marginTop: '0.5rem' }}>
        Settings page coming soon.
      </p>
    </div>
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
            path: 'config',
            element: <ConfigRemote />,
          },
          {
            path: 'config/:customerCompanyId',
            element: <ConfigRemote />,
          },
          {
            path: 'settings',
            element: <SettingsPlaceholder />,
          },
        ],
      },
    ],
  },
]);
