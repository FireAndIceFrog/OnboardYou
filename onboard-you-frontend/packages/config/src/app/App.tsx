import '@/i18n';
import { Provider } from 'react-redux';
import { RouterProvider } from 'react-router-dom';
import { store } from '@/store';
import { router } from './routes';
import { ErrorBoundary } from '@/shared/ui';
import '@/styles/config.scss';
import '@xyflow/react/dist/style.css';

export default function App() {
  return (
    <ErrorBoundary>
      <Provider store={store}>
        <RouterProvider router={router} />
      </Provider>
    </ErrorBoundary>
  );
}

// Export for Module Federation — renders inside host's router context
export { ConfigRoutes } from './ConfigRoutes';
export { setGlobalValue } from '@/shared/hooks';
