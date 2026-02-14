// ============================================================================
// OnboardYou — App Root Component
//
// Wraps the router with the AuthProvider so every route has access to auth
// context. Global styles are imported here.
// ============================================================================

import { RouterProvider } from 'react-router-dom';
import { AuthProvider } from '@/auth/AuthProvider';
import { router } from '@/router';
import './styles/global.scss';

export default function App() {
  return (
    <AuthProvider>
      <RouterProvider router={router} />
    </AuthProvider>
  );
}
