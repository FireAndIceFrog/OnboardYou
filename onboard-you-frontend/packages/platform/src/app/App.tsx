import { Provider } from 'react-redux';
import { RouterProvider } from 'react-router-dom';
import { store } from '@/store';
import { initAuth } from '@/features/auth/state/authSlice';
import { configureApiClient } from '@/shared/services/configureApiClient';
import { ErrorBoundary } from '@/shared/ui/ErrorBoundary';
import { useThemeEffect } from '@/shared/hooks/useThemeEffect';
import { router } from './routes';
import '@/styles/global.scss';

// Configure the generated API client with baseUrl + auth interceptor
configureApiClient();

// Initialise auth on startup (mock auto-login / session check)
store.dispatch(initAuth());

// Set initial theme on DOM before first paint to prevent flash
document.documentElement.setAttribute(
  'data-theme',
  store.getState().global.theme,
);

function AppShell() {
  useThemeEffect();
  return <RouterProvider router={router} />;
}

export default function App() {
  return (
    <ErrorBoundary>
      <Provider store={store}>
        <AppShell />
      </Provider>
    </ErrorBoundary>
  );
}
