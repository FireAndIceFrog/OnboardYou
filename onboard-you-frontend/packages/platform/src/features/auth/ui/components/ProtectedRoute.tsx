import { Navigate, Outlet } from 'react-router-dom';
import { Center, Spinner } from '@chakra-ui/react';
import { useAppSelector } from '@/store';
import {
  selectIsAuthenticated,
  selectIsLoading,
} from '@/features/auth/state/authSlice';

export function ProtectedRoute() {
  const isLoading = useAppSelector(selectIsLoading);
  const isAuthenticated = useAppSelector(selectIsAuthenticated);

  if (isLoading) {
    return (
      <Center minH="100%">
        <Spinner size="lg" />
      </Center>
    );
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <Outlet />;
}
