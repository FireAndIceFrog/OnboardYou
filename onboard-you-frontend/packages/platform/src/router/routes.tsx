// ============================================================================
// OnboardYou — Router Configuration
//
// React Router v7 — createBrowserRouter + RouterProvider.
// ============================================================================

import { createBrowserRouter, Navigate } from 'react-router-dom';
import { lazy, Suspense } from 'react';
import { LoginPage } from '@/auth/LoginPage';
import { CallbackPage } from '@/auth/CallbackPage';
import { ProtectedRoute } from '@/auth/ProtectedRoute';
import { AppLayout } from '@/layout/AppLayout';
import { HomeScreen } from '@/home/HomeScreen';

// ---------------------------------------------------------------------------
// Lazy-loaded routes
// ---------------------------------------------------------------------------

const ConfigsPlaceholder = lazy(() => import('./ConfigsPlaceholder'));

// ---------------------------------------------------------------------------
// Inline Settings placeholder
// ---------------------------------------------------------------------------

function SettingsPage() {
  return (
    <div style={{ padding: '2rem', maxWidth: 800 }}>
      <h1
        style={{
          fontSize: '1.5rem',
          fontWeight: 700,
          color: '#0F172A',
          marginBottom: '0.5rem',
        }}
      >
        Settings
      </h1>
      <p style={{ color: '#475569', fontSize: '0.875rem' }}>
        Organization settings, integrations, and preferences will be configured here.
      </p>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Spinner for Suspense fallback
// ---------------------------------------------------------------------------

function RouteFallback() {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '4rem',
        color: '#475569',
        gap: '0.75rem',
      }}
    >
      <div
        style={{
          width: 24,
          height: 24,
          border: '2.5px solid #E2E8F0',
          borderTopColor: '#2563EB',
          borderRadius: '50%',
          animation: 'spin 0.8s linear infinite',
        }}
      />
      <span style={{ fontSize: '0.875rem' }}>Loading…</span>
      <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Router definition
// ---------------------------------------------------------------------------

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
    path: '/',
    element: (
      <ProtectedRoute>
        <AppLayout />
      </ProtectedRoute>
    ),
    children: [
      {
        index: true,
        element: <HomeScreen />,
      },
      {
        path: 'configs/*',
        element: (
          <Suspense fallback={<RouteFallback />}>
            <ConfigsPlaceholder />
          </Suspense>
        ),
      },
      {
        path: 'settings',
        element: <SettingsPage />,
      },
      {
        // Catch unknown nested paths → redirect home
        path: '*',
        element: <Navigate to="/" replace />,
      },
    ],
  },
]);
