import { useContext } from 'react';
import { Navigate, Outlet } from 'react-router-dom';
import { AuthContext } from '@/features/auth/state/AuthContext';
import { Spinner } from '@/shared/ui/Spinner';

export function ProtectedRoute() {
  const authCtx = useContext(AuthContext);

  if (!authCtx) {
    throw new Error('ProtectedRoute must be used within an AuthProvider');
  }

  const { state } = authCtx;

  if (state.isLoading) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', minHeight: '100vh' }}>
        <Spinner size="lg" />
      </div>
    );
  }

  if (!state.isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <Outlet />;
}
