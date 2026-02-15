import { Navigate, Outlet } from 'react-router-dom';
import { useAppSelector } from '@/store';
import { selectIsAuthenticated, selectIsLoading } from '@/features/auth/state/authSlice';
import { Spinner } from '@/shared/ui/Spinner';

export function ProtectedRoute() {
  const isLoading = useAppSelector(selectIsLoading);
  const isAuthenticated = useAppSelector(selectIsAuthenticated);

  if (isLoading) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', minHeight: '100vh' }}>
        <Spinner size="lg" />
      </div>
    );
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <Outlet />;
}
