import { useEffect } from 'react';
import { useNavigation } from '@react-navigation/native';
import { useAuthStore } from '../store/auth_store';

/**
 * Hook to handle navigation based on authentication state
 */
export const useAuthNavigation = () => {
  const navigation = useNavigation();
  const { isAuthenticated, user, isLoading } = useAuthStore();

  useEffect(() => {
    if (isLoading) return; // Don't navigate while loading

    if (isAuthenticated && user) {
      // User is authenticated, navigate to main app
      navigation.reset({
        index: 0,
        routes: [{ name: 'Main' as never }],
      });
    } else {
      // User is not authenticated, navigate to auth flow
      navigation.reset({
        index: 0,
        routes: [{ name: 'Auth' as never }],
      });
    }
  }, [isAuthenticated, user, isLoading, navigation]);

  return {
    isAuthenticated,
    user,
    isLoading,
  };
};

/**
 * Hook to protect routes that require authentication
 */
export const useAuthGuard = (redirectTo: string = 'Login') => {
  const navigation = useNavigation();
  const { isAuthenticated, isLoading, restoreSession } = useAuthStore();

  useEffect(() => {
    // Attempt to restore session
    restoreSession();
  }, [restoreSession]);

  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      navigation.navigate(redirectTo as never);
    }
  }, [isAuthenticated, isLoading, navigation, redirectTo]);

  return {
    isAuthenticated,
    isLoading,
    canAccess: !isLoading && isAuthenticated,
  };
};

/**
 * Hook to redirect authenticated users away from auth screens
 */
export const useGuestGuard = (redirectTo: string = 'Main') => {
  const navigation = useNavigation();
  const { isAuthenticated, isLoading } = useAuthStore();

  useEffect(() => {
    if (!isLoading && isAuthenticated) {
      navigation.navigate(redirectTo as never);
    }
  }, [isAuthenticated, isLoading, navigation, redirectTo]);

  return {
    isAuthenticated,
    isLoading,
    canAccess: !isLoading && !isAuthenticated,
  };
};