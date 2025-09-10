import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import AsyncStorage from '@react-native-async-storage/async-storage';
import authService, { AuthUser, AuthData, LoginRequest, RegisterRequest } from '../services/auth_service';

// Helper function to clear all local data
const clearLocalData = async () => {
  try {
    // Clear specific app data keys
    const keysToRemove = [
      'imkitchen-recipes-cache',
      'imkitchen-meal-plans-cache',
      'imkitchen-preferences-cache',
      'imkitchen-user-data-cache',
      'imkitchen-app-state',
    ];
    
    await Promise.all(keysToRemove.map(key => AsyncStorage.removeItem(key)));
  } catch (error) {
    console.error('Failed to clear local data:', error);
  }
};

interface AuthState {
  // State
  user: AuthUser | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  token: string | null;
  refreshToken: string | null;
  sessionExpiry: Date | null;
  error: string | null;
  
  // Actions
  login: (request: LoginRequest) => Promise<void>;
  register: (request: RegisterRequest) => Promise<void>;
  logout: () => Promise<void>;
  refreshAuth: () => Promise<void>;
  forgotPassword: (email: string) => Promise<void>;
  resetPassword: (newPassword: string) => Promise<void>;
  restoreSession: () => Promise<void>;
  clearError: () => void;
  setLoading: (loading: boolean) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      // Initial state
      user: null,
      isAuthenticated: false,
      isLoading: false,
      token: null,
      refreshToken: null,
      sessionExpiry: null,
      error: null,

      // Actions
      login: async (request: LoginRequest) => {
        set({ isLoading: true, error: null });
        
        try {
          const authData = await authService.login(request);
          
          set({
            user: authData.user,
            isAuthenticated: true,
            token: authData.accessToken,
            refreshToken: authData.refreshToken,
            sessionExpiry: new Date(Date.now() + authData.expiresIn * 1000),
            isLoading: false,
            error: null,
          });
        } catch (error) {
          set({
            user: null,
            isAuthenticated: false,
            token: null,
            refreshToken: null,
            sessionExpiry: null,
            isLoading: false,
            error: error instanceof Error ? error.message : 'Login failed',
          });
          throw error;
        }
      },

      register: async (request: RegisterRequest) => {
        set({ isLoading: true, error: null });
        
        try {
          const authData = await authService.register(request);
          
          set({
            user: authData.user,
            isAuthenticated: true,
            token: authData.accessToken,
            refreshToken: authData.refreshToken,
            sessionExpiry: new Date(Date.now() + authData.expiresIn * 1000),
            isLoading: false,
            error: null,
          });
        } catch (error) {
          set({
            user: null,
            isAuthenticated: false,
            token: null,
            refreshToken: null,
            sessionExpiry: null,
            isLoading: false,
            error: error instanceof Error ? error.message : 'Registration failed',
          });
          throw error;
        }
      },

      logout: async () => {
        set({ isLoading: true, error: null });
        
        try {
          await authService.logout();
        } catch (error) {
          console.warn('Logout service call failed:', error);
          // Continue with local cleanup even if server logout fails
        }

        // Always clear local state and storage
        try {
          // Clear any cached data (recipes, meal plans, etc.)
          await clearLocalData();
          
          set({
            user: null,
            isAuthenticated: false,
            token: null,
            refreshToken: null,
            sessionExpiry: null,
            isLoading: false,
            error: null,
          });
        } catch (error) {
          set({
            user: null,
            isAuthenticated: false,
            token: null,
            refreshToken: null,
            sessionExpiry: null,
            isLoading: false,
            error: error instanceof Error ? error.message : 'Logout cleanup failed',
          });
        }
      },

      refreshAuth: async () => {
        const state = get();
        
        if (!state.refreshToken) {
          return;
        }

        try {
          const authData = await authService.refreshToken();
          
          if (authData) {
            set({
              user: authData.user,
              isAuthenticated: true,
              token: authData.accessToken,
              refreshToken: authData.refreshToken,
              sessionExpiry: new Date(Date.now() + authData.expiresIn * 1000),
              error: null,
            });
          } else {
            // Clear state if refresh failed
            set({
              user: null,
              isAuthenticated: false,
              token: null,
              refreshToken: null,
              sessionExpiry: null,
            });
          }
        } catch (error) {
          console.error('Token refresh failed:', error);
          
          // Clear state on refresh failure
          set({
            user: null,
            isAuthenticated: false,
            token: null,
            refreshToken: null,
            sessionExpiry: null,
            error: error instanceof Error ? error.message : 'Session expired',
          });
        }
      },

      forgotPassword: async (email: string) => {
        set({ isLoading: true, error: null });
        
        try {
          await authService.forgotPassword(email);
          set({ isLoading: false, error: null });
        } catch (error) {
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Password reset failed',
          });
          throw error;
        }
      },

      resetPassword: async (newPassword: string) => {
        set({ isLoading: true, error: null });
        
        try {
          await authService.resetPassword(newPassword);
          set({ isLoading: false, error: null });
        } catch (error) {
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Password update failed',
          });
          throw error;
        }
      },

      restoreSession: async () => {
        set({ isLoading: true, error: null });
        
        try {
          const authData = await authService.restoreSession();
          
          if (authData) {
            set({
              user: authData.user,
              isAuthenticated: true,
              token: authData.accessToken,
              refreshToken: authData.refreshToken,
              sessionExpiry: new Date(Date.now() + authData.expiresIn * 1000),
              isLoading: false,
              error: null,
            });
          } else {
            set({
              user: null,
              isAuthenticated: false,
              token: null,
              refreshToken: null,
              sessionExpiry: null,
              isLoading: false,
              error: null,
            });
          }
        } catch (error) {
          set({
            user: null,
            isAuthenticated: false,
            token: null,
            refreshToken: null,
            sessionExpiry: null,
            isLoading: false,
            error: error instanceof Error ? error.message : 'Session restore failed',
          });
        }
      },

      clearError: () => set({ error: null }),
      setLoading: (loading: boolean) => set({ isLoading: loading }),
    }),
    {
      name: 'imkitchen-auth-store',
      storage: {
        getItem: async (name: string) => {
          const value = await AsyncStorage.getItem(name);
          return value ? JSON.parse(value) : null;
        },
        setItem: async (name: string, value: any) => {
          await AsyncStorage.setItem(name, JSON.stringify(value));
        },
        removeItem: async (name: string) => {
          await AsyncStorage.removeItem(name);
        },
      },
      partialize: (state) => ({
        // Only persist essential state, not sensitive tokens
        isAuthenticated: state.isAuthenticated,
        user: state.user,
      }),
    }
  )
);

// Auto-refresh token when it's close to expiring
let refreshTimer: NodeJS.Timeout | null = null;

useAuthStore.subscribe((state) => {
  if (refreshTimer) {
    clearTimeout(refreshTimer);
    refreshTimer = null;
  }

  if (state.isAuthenticated && state.sessionExpiry) {
    const timeUntilExpiry = state.sessionExpiry.getTime() - Date.now();
    const refreshTime = Math.max(0, timeUntilExpiry - 5 * 60 * 1000); // Refresh 5 minutes before expiry

    if (refreshTime > 0) {
      refreshTimer = setTimeout(() => {
        state.refreshAuth();
      }, refreshTime);
    } else {
      // Token is already expired or close to expiry, refresh immediately
      state.refreshAuth();
    }
  }
});

export default useAuthStore;