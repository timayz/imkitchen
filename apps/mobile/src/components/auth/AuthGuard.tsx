import React, { useEffect } from 'react';
import { useAuthStore } from '../../store/auth_store';

interface AuthGuardProps {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  requireAuth?: boolean;
}

const AuthGuard: React.FC<AuthGuardProps> = ({ 
  children, 
  fallback = null, 
  requireAuth = true 
}) => {
  const { isAuthenticated, isLoading, restoreSession } = useAuthStore();

  useEffect(() => {
    // Attempt to restore session on mount
    restoreSession();
  }, [restoreSession]);

  if (isLoading) {
    return fallback;
  }

  if (requireAuth && !isAuthenticated) {
    return fallback;
  }

  if (!requireAuth && isAuthenticated) {
    return fallback;
  }

  return <>{children}</>;
};

export default AuthGuard;