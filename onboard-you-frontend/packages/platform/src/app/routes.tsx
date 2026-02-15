import { Suspense, lazy } from 'react';
import { createBrowserRouter } from 'react-router-dom';
import { LoginPage, CallbackPage, ProtectedRoute } from '@/features/auth';
import { AppLayout } from '@/features/layout';
import { HomeScreen } from '@/features/home';
import { SettingsPage } from '@/features/settings';
import { Spinner } from '@/shared/ui/Spinner';
import { ErrorBoundary } from '@/shared/ui/ErrorBoundary';

const ConfigApp = lazy(async () => {
  const m = await import('configApp/App');
  return { default: m.ConfigRoutes };
});

function RemoteLoadFallback({ reset }: { reset: () => void }) {
  return (
    <div
      role="alert"
      aria-live="assertive"
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: '1rem',
        padding: '4rem',
        textAlign: 'center',
      }}
    >
      <span style={{ fontSize: '2rem' }} aria-hidden="true">⚠️</span>
      <h2 style={{ margin: 0 }}>Failed to load module</h2>
      <p style={{ margin: 0, color: '#64748B' }}>
        The configuration module could not be loaded. Please check your connection and try again.
      </p>
      <button
        onClick={reset}
        type="button"
        style={{
          marginTop: '0.5rem',
          padding: '0.5rem 1.5rem',
          background: '#2563EB',
          color: '#fff',
          border: 'none',
          borderRadius: '0.375rem',
          cursor: 'pointer',
        }}
      >
        Try Again
      </button>
    </div>
  );
}

function ConfigRemote() {
  return (
    <ErrorBoundary
      fallback={(error, reset) => <RemoteLoadFallback reset={reset} />}
    >
      <Suspense
        fallback={
          <div style={{ display: 'flex', justifyContent: 'center', padding: '4rem' }}>
            <Spinner size="lg" />
          </div>
        }
      >
        <ConfigApp />
      </Suspense>
    </ErrorBoundary>
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
