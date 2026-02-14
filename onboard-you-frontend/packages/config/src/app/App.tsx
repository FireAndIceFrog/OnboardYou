import { RouterProvider } from 'react-router-dom';
import { router } from './routes';
import '@/styles/config.scss';
import '@xyflow/react/dist/style.css';

export default function App() {
  return <RouterProvider router={router} />;
}

// Export for Module Federation — renders inside host's router context
export { ConfigRoutes } from './ConfigRoutes';
