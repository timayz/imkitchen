import { renderHook, act } from '@testing-library/react-hooks';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { usePreferenceStore } from '../../src/store/preference_store';
import preferenceService from '../../src/services/preference_service';

// Mock AsyncStorage
jest.mock('@react-native-async-storage/async-storage', () => ({
  getItem: jest.fn(),
  setItem: jest.fn(),
  removeItem: jest.fn(),
}));

// Mock preference service
jest.mock('../../src/services/preference_service', () => ({
  getUserPreferences: jest.fn(),
  updateUserPreferences: jest.fn(),
  resetUserPreferences: jest.fn(),
}));

const mockAsyncStorage = AsyncStorage as jest.Mocked<typeof AsyncStorage>;
const mockPreferenceService = preferenceService as jest.Mocked<typeof preferenceService>;

describe('usePreferenceStore', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    // Reset store state
    usePreferenceStore.getState().preferences = {
      maxCookTime: 60,
      preferredComplexity: 'moderate',
    };
    usePreferenceStore.getState().isLoading = false;
    usePreferenceStore.getState().error = null;
    usePreferenceStore.getState().lastUpdated = null;
  });

  describe('loadPreferences', () => {
    it('loads preferences successfully', async () => {
      const mockResponse = {
        data: {
          maxCookTime: 90,
          preferredComplexity: 'complex' as const,
        },
        metadata: {
          retrievedAt: '2025-09-08T10:00:00Z',
        },
      };

      mockPreferenceService.getUserPreferences.mockResolvedValueOnce(mockResponse);

      const { result } = renderHook(() => usePreferenceStore());

      await act(async () => {
        await result.current.loadPreferences();
      });

      expect(result.current.preferences).toEqual(mockResponse.data);
      expect(result.current.lastUpdated).toBe(mockResponse.metadata.retrievedAt);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBe(null);
    });

    it('handles loading error', async () => {
      const mockError = new Error('Network error');
      mockPreferenceService.getUserPreferences.mockRejectedValueOnce(mockError);

      const { result } = renderHook(() => usePreferenceStore());

      await act(async () => {
        await result.current.loadPreferences();
      });

      expect(result.current.error).toBe(mockError.message);
      expect(result.current.isLoading).toBe(false);
    });

    it('sets loading state during request', async () => {
      let resolvePromise: (value: any) => void;
      const mockPromise = new Promise((resolve) => {
        resolvePromise = resolve;
      });

      mockPreferenceService.getUserPreferences.mockReturnValueOnce(mockPromise);

      const { result } = renderHook(() => usePreferenceStore());

      act(() => {
        result.current.loadPreferences();
      });

      expect(result.current.isLoading).toBe(true);

      await act(async () => {
        resolvePromise!({
          data: { maxCookTime: 60, preferredComplexity: 'moderate' },
          metadata: { retrievedAt: '2025-09-08T10:00:00Z' },
        });
        await mockPromise;
      });

      expect(result.current.isLoading).toBe(false);
    });
  });

  describe('updatePreferences', () => {
    it('updates preferences successfully', async () => {
      const updates = { maxCookTime: 45 };
      const mockResponse = {
        data: {
          maxCookTime: 45,
          preferredComplexity: 'moderate' as const,
        },
        metadata: {
          updatedAt: '2025-09-08T10:00:00Z',
        },
      };

      mockPreferenceService.updateUserPreferences.mockResolvedValueOnce(mockResponse);

      const { result } = renderHook(() => usePreferenceStore());

      await act(async () => {
        await result.current.updatePreferences(updates);
      });

      expect(result.current.preferences.maxCookTime).toBe(45);
      expect(result.current.lastUpdated).toBe(mockResponse.metadata.updatedAt);
      expect(result.current.error).toBe(null);
    });

    it('validates maxCookTime range', async () => {
      const { result } = renderHook(() => usePreferenceStore());

      await act(async () => {
        await result.current.updatePreferences({ maxCookTime: 200 });
      });

      expect(result.current.error).toBe('Max cook time must be between 15 and 180 minutes');
      expect(mockPreferenceService.updateUserPreferences).not.toHaveBeenCalled();
    });

    it('validates complexity values', async () => {
      const { result } = renderHook(() => usePreferenceStore());

      await act(async () => {
        await result.current.updatePreferences({ preferredComplexity: 'invalid' as any });
      });

      expect(result.current.error).toBe('Invalid complexity level');
      expect(mockPreferenceService.updateUserPreferences).not.toHaveBeenCalled();
    });

    it('handles update error', async () => {
      const mockError = new Error('Update failed');
      mockPreferenceService.updateUserPreferences.mockRejectedValueOnce(mockError);

      const { result } = renderHook(() => usePreferenceStore());

      await act(async () => {
        await result.current.updatePreferences({ maxCookTime: 45 });
      });

      expect(result.current.error).toBe(mockError.message);
      expect(result.current.isLoading).toBe(false);
    });
  });

  describe('resetPreferences', () => {
    it('resets preferences to defaults', async () => {
      const mockResponse = {
        data: {
          maxCookTime: 60,
          preferredComplexity: 'moderate' as const,
        },
        metadata: {
          resetAt: '2025-09-08T10:00:00Z',
        },
      };

      mockPreferenceService.resetUserPreferences.mockResolvedValueOnce(mockResponse);

      const { result } = renderHook(() => usePreferenceStore());

      await act(async () => {
        await result.current.resetPreferences();
      });

      expect(result.current.preferences).toEqual(mockResponse.data);
      expect(result.current.lastUpdated).toBe(mockResponse.metadata.resetAt);
      expect(result.current.error).toBe(null);
    });
  });

  describe('local state updates', () => {
    it('updates maxCookTime locally', () => {
      const { result } = renderHook(() => usePreferenceStore());

      act(() => {
        result.current.setMaxCookTime(75);
      });

      expect(result.current.preferences.maxCookTime).toBe(75);
    });

    it('updates preferredComplexity locally', () => {
      const { result } = renderHook(() => usePreferenceStore());

      act(() => {
        result.current.setPreferredComplexity('complex');
      });

      expect(result.current.preferences.preferredComplexity).toBe('complex');
    });
  });

  describe('error handling', () => {
    it('clears error state', () => {
      const { result } = renderHook(() => usePreferenceStore());

      act(() => {
        result.current.error = 'Some error';
        result.current.clearError();
      });

      expect(result.current.error).toBe(null);
    });
  });

  describe('persistence', () => {
    it('loads from AsyncStorage on initialization', async () => {
      const mockStoredData = {
        state: {
          preferences: {
            maxCookTime: 120,
            preferredComplexity: 'complex',
          },
          lastUpdated: '2025-09-08T09:00:00Z',
        },
        version: 0,
      };

      mockAsyncStorage.getItem.mockResolvedValueOnce(JSON.stringify(mockStoredData));

      // Re-initialize store to trigger persistence load
      const { result } = renderHook(() => usePreferenceStore());

      // Wait for persistence to load
      await new Promise(resolve => setTimeout(resolve, 0));

      expect(result.current.preferences.maxCookTime).toBe(120);
      expect(result.current.preferences.preferredComplexity).toBe('complex');
    });

    it('saves to AsyncStorage on state changes', async () => {
      const { result } = renderHook(() => usePreferenceStore());

      act(() => {
        result.current.setMaxCookTime(90);
      });

      // Wait for persistence to save
      await new Promise(resolve => setTimeout(resolve, 100));

      expect(mockAsyncStorage.setItem).toHaveBeenCalledWith(
        'imkitchen-preferences-store',
        expect.stringContaining('"maxCookTime":90')
      );
    });
  });
});