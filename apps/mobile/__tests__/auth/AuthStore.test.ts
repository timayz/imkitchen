import AsyncStorage from '@react-native-async-storage/async-storage';
import { act, renderHook } from '@testing-library/react-hooks';
import { useAuthStore } from '../../src/store/auth_store';
import authService from '../../src/services/auth_service';

// Mock dependencies
jest.mock('@react-native-async-storage/async-storage');
jest.mock('../../src/services/auth_service');

const mockAuthService = authService as jest.Mocked<typeof authService>;
const mockAsyncStorage = AsyncStorage as jest.Mocked<typeof AsyncStorage>;

describe('Auth Store', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    // Reset store state
    useAuthStore.getState().logout();
  });

  describe('login', () => {
    it('should login successfully and update state', async () => {
      const mockAuthData = {
        accessToken: 'mock-access-token',
        refreshToken: 'mock-refresh-token',
        expiresIn: 3600,
        tokenType: 'bearer',
        user: {
          id: 'user-123',
          email: 'test@example.com',
          emailVerified: true,
          name: 'Test User',
          createdAt: '2023-01-01T00:00:00Z',
          updatedAt: '2023-01-01T00:00:00Z',
        },
      };

      mockAuthService.login.mockResolvedValue(mockAuthData);

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.login({
          email: 'test@example.com',
          password: 'password123',
        });
      });

      expect(result.current.isAuthenticated).toBe(true);
      expect(result.current.user).toEqual(mockAuthData.user);
      expect(result.current.token).toBe(mockAuthData.accessToken);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it('should handle login failure', async () => {
      const mockError = new Error('Invalid credentials');
      mockAuthService.login.mockRejectedValue(mockError);

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        try {
          await result.current.login({
            email: 'test@example.com',
            password: 'wrongpassword',
          });
        } catch (error) {
          // Expected to throw
        }
      });

      expect(result.current.isAuthenticated).toBe(false);
      expect(result.current.user).toBeNull();
      expect(result.current.token).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBe('Invalid credentials');
    });

    it('should set loading state during login', async () => {
      let resolveLogin: (value: any) => void;
      const loginPromise = new Promise((resolve) => {
        resolveLogin = resolve;
      });

      mockAuthService.login.mockReturnValue(loginPromise);

      const { result } = renderHook(() => useAuthStore());

      act(() => {
        result.current.login({
          email: 'test@example.com',
          password: 'password123',
        });
      });

      expect(result.current.isLoading).toBe(true);

      const mockAuthData = {
        accessToken: 'mock-access-token',
        refreshToken: 'mock-refresh-token',
        expiresIn: 3600,
        tokenType: 'bearer',
        user: {
          id: 'user-123',
          email: 'test@example.com',
          emailVerified: true,
          name: 'Test User',
          createdAt: '2023-01-01T00:00:00Z',
          updatedAt: '2023-01-01T00:00:00Z',
        },
      };

      await act(async () => {
        resolveLogin!(mockAuthData);
      });

      expect(result.current.isLoading).toBe(false);
    });
  });

  describe('register', () => {
    it('should register successfully and update state', async () => {
      const mockAuthData = {
        accessToken: 'mock-access-token',
        refreshToken: 'mock-refresh-token',
        expiresIn: 3600,
        tokenType: 'bearer',
        user: {
          id: 'user-123',
          email: 'test@example.com',
          emailVerified: false,
          name: 'Test User',
          createdAt: '2023-01-01T00:00:00Z',
          updatedAt: '2023-01-01T00:00:00Z',
        },
      };

      mockAuthService.register.mockResolvedValue(mockAuthData);

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.register({
          email: 'test@example.com',
          password: 'Password123!',
          name: 'Test User',
        });
      });

      expect(result.current.isAuthenticated).toBe(true);
      expect(result.current.user).toEqual(mockAuthData.user);
      expect(result.current.token).toBe(mockAuthData.accessToken);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it('should handle registration failure', async () => {
      const mockError = new Error('User already exists');
      mockAuthService.register.mockRejectedValue(mockError);

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        try {
          await result.current.register({
            email: 'test@example.com',
            password: 'Password123!',
            name: 'Test User',
          });
        } catch (error) {
          // Expected to throw
        }
      });

      expect(result.current.isAuthenticated).toBe(false);
      expect(result.current.user).toBeNull();
      expect(result.current.token).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBe('User already exists');
    });
  });

  describe('logout', () => {
    it('should logout successfully and clear state', async () => {
      // First login
      const mockAuthData = {
        accessToken: 'mock-access-token',
        refreshToken: 'mock-refresh-token',
        expiresIn: 3600,
        tokenType: 'bearer',
        user: {
          id: 'user-123',
          email: 'test@example.com',
          emailVerified: true,
          name: 'Test User',
          createdAt: '2023-01-01T00:00:00Z',
          updatedAt: '2023-01-01T00:00:00Z',
        },
      };

      mockAuthService.login.mockResolvedValue(mockAuthData);
      mockAuthService.logout.mockResolvedValue(undefined);

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.login({
          email: 'test@example.com',
          password: 'password123',
        });
      });

      expect(result.current.isAuthenticated).toBe(true);

      // Now logout
      await act(async () => {
        await result.current.logout();
      });

      expect(result.current.isAuthenticated).toBe(false);
      expect(result.current.user).toBeNull();
      expect(result.current.token).toBeNull();
      expect(result.current.refreshToken).toBeNull();
      expect(result.current.sessionExpiry).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it('should clear state even if logout service fails', async () => {
      // First login
      const mockAuthData = {
        accessToken: 'mock-access-token',
        refreshToken: 'mock-refresh-token',
        expiresIn: 3600,
        tokenType: 'bearer',
        user: {
          id: 'user-123',
          email: 'test@example.com',
          emailVerified: true,
          name: 'Test User',
          createdAt: '2023-01-01T00:00:00Z',
          updatedAt: '2023-01-01T00:00:00Z',
        },
      };

      mockAuthService.login.mockResolvedValue(mockAuthData);
      mockAuthService.logout.mockRejectedValue(new Error('Logout failed'));

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.login({
          email: 'test@example.com',
          password: 'password123',
        });
      });

      expect(result.current.isAuthenticated).toBe(true);

      // Now logout (should still clear state despite service failure)
      await act(async () => {
        await result.current.logout();
      });

      expect(result.current.isAuthenticated).toBe(false);
      expect(result.current.user).toBeNull();
      expect(result.current.token).toBeNull();
    });
  });

  describe('refreshAuth', () => {
    it('should refresh token successfully', async () => {
      const initialAuthData = {
        accessToken: 'old-access-token',
        refreshToken: 'old-refresh-token',
        expiresIn: 3600,
        tokenType: 'bearer',
        user: {
          id: 'user-123',
          email: 'test@example.com',
          emailVerified: true,
          name: 'Test User',
          createdAt: '2023-01-01T00:00:00Z',
          updatedAt: '2023-01-01T00:00:00Z',
        },
      };

      const refreshedAuthData = {
        ...initialAuthData,
        accessToken: 'new-access-token',
        refreshToken: 'new-refresh-token',
      };

      mockAuthService.login.mockResolvedValue(initialAuthData);
      mockAuthService.refreshToken.mockResolvedValue(refreshedAuthData);

      const { result } = renderHook(() => useAuthStore());

      // First login
      await act(async () => {
        await result.current.login({
          email: 'test@example.com',
          password: 'password123',
        });
      });

      // Now refresh
      await act(async () => {
        await result.current.refreshAuth();
      });

      expect(result.current.token).toBe('new-access-token');
      expect(result.current.refreshToken).toBe('new-refresh-token');
      expect(result.current.isAuthenticated).toBe(true);
    });

    it('should clear state if refresh fails', async () => {
      const initialAuthData = {
        accessToken: 'old-access-token',
        refreshToken: 'old-refresh-token',
        expiresIn: 3600,
        tokenType: 'bearer',
        user: {
          id: 'user-123',
          email: 'test@example.com',
          emailVerified: true,
          name: 'Test User',
          createdAt: '2023-01-01T00:00:00Z',
          updatedAt: '2023-01-01T00:00:00Z',
        },
      };

      mockAuthService.login.mockResolvedValue(initialAuthData);
      mockAuthService.refreshToken.mockRejectedValue(new Error('Token expired'));

      const { result } = renderHook(() => useAuthStore());

      // First login
      await act(async () => {
        await result.current.login({
          email: 'test@example.com',
          password: 'password123',
        });
      });

      expect(result.current.isAuthenticated).toBe(true);

      // Now refresh (should fail and clear state)
      await act(async () => {
        await result.current.refreshAuth();
      });

      expect(result.current.isAuthenticated).toBe(false);
      expect(result.current.token).toBeNull();
      expect(result.current.user).toBeNull();
    });
  });

  describe('forgotPassword', () => {
    it('should send forgot password email successfully', async () => {
      mockAuthService.forgotPassword.mockResolvedValue(undefined);

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.forgotPassword('test@example.com');
      });

      expect(mockAuthService.forgotPassword).toHaveBeenCalledWith('test@example.com');
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it('should handle forgot password failure', async () => {
      const mockError = new Error('Email not found');
      mockAuthService.forgotPassword.mockRejectedValue(mockError);

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        try {
          await result.current.forgotPassword('test@example.com');
        } catch (error) {
          // Expected to throw
        }
      });

      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBe('Email not found');
    });
  });

  describe('restoreSession', () => {
    it('should restore session successfully', async () => {
      const mockAuthData = {
        accessToken: 'restored-access-token',
        refreshToken: 'restored-refresh-token',
        expiresIn: 3600,
        tokenType: 'bearer',
        user: {
          id: 'user-123',
          email: 'test@example.com',
          emailVerified: true,
          name: 'Test User',
          createdAt: '2023-01-01T00:00:00Z',
          updatedAt: '2023-01-01T00:00:00Z',
        },
      };

      mockAuthService.restoreSession.mockResolvedValue(mockAuthData);

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.restoreSession();
      });

      expect(result.current.isAuthenticated).toBe(true);
      expect(result.current.user).toEqual(mockAuthData.user);
      expect(result.current.token).toBe(mockAuthData.accessToken);
      expect(result.current.isLoading).toBe(false);
    });

    it('should handle restore session failure', async () => {
      mockAuthService.restoreSession.mockResolvedValue(null);

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.restoreSession();
      });

      expect(result.current.isAuthenticated).toBe(false);
      expect(result.current.user).toBeNull();
      expect(result.current.token).toBeNull();
      expect(result.current.isLoading).toBe(false);
    });
  });

  describe('clearError', () => {
    it('should clear error state', async () => {
      const mockError = new Error('Test error');
      mockAuthService.login.mockRejectedValue(mockError);

      const { result } = renderHook(() => useAuthStore());

      // Trigger an error
      await act(async () => {
        try {
          await result.current.login({
            email: 'test@example.com',
            password: 'wrongpassword',
          });
        } catch (error) {
          // Expected to throw
        }
      });

      expect(result.current.error).toBe('Test error');

      // Clear the error
      act(() => {
        result.current.clearError();
      });

      expect(result.current.error).toBeNull();
    });
  });
});