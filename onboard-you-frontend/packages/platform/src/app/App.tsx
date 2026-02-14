import { RouterProvider } from 'react-router-dom';
import { AuthProvider } from '@/features/auth';
import { GlobalProvider } from '@/shared/state';
import { router } from './routes';
import '@/styles/global.scss';

export default function App() {
  return (
    <GlobalProvider>
      <AuthProvider>
        <RouterProvider router={router} />
      </AuthProvider>
    </GlobalProvider>
  );
}
