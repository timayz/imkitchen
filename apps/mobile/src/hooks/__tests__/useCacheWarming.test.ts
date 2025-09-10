import { renderHook, act } from '@testing-library/react-hooks';
import { AppState } from 'react-native';
import { useCacheWarming } from '../useCacheWarming';
import { useRecipeStore } from '../../store/recipe_store';

// Mock the recipe store
jest.mock('../../store/recipe_store');
const mockUseRecipeStore = useRecipeStore as jest.MockedFunction<typeof useRecipeStore>;

// Mock AppState
jest.mock('react-native', () => ({
  AppState: {
    addEventListener: jest.fn(),
  },
}));

// Mock timers
jest.useFakeTimers();

describe('useCacheWarming', () => {
  const mockWarmCache = jest.fn();
  const mockIsOffline = jest.fn();
  const mockNetworkStatus = {
    isConnected: true,
    type: 'wifi',
    isInternetReachable: true,
  };

  beforeEach(() => {
    jest.clearAllMocks();
    mockUseRecipeStore.mockReturnValue({
      warmCache: mockWarmCache,
      isOffline: mockIsOffline,
      networkStatus: mockNetworkStatus,
    } as any);
    mockIsOffline.mockReturnValue(false);
  });

  afterEach(() => {
    jest.clearAllTimers();
  });

  describe('initialization', () => {
    it('should initialize with default config', () => {
      const { result } = renderHook(() => useCacheWarming());

      expect(result.current.config).toEqual({
        enabled: true,
        onAppForeground: true,
        intervalMinutes: 30,
        maxRecipesToWarm: 20,
      });
    });

    it('should merge custom config with defaults', () => {
      const customConfig = {
        intervalMinutes: 60,
        maxRecipesToWarm: 10,
      };

      const { result } = renderHook(() => useCacheWarming(customConfig));

      expect(result.current.config).toEqual({
        enabled: true,
        onAppForeground: true,
        intervalMinutes: 60,
        maxRecipesToWarm: 10,
      });
    });
  });

  describe('warmFrequentRecipes', () => {
    it('should warm cache when enabled and online', async () => {
      const { result } = renderHook(() => useCacheWarming());

      await act(async () => {
        await result.current.warmFrequentRecipes();
      });

      expect(mockWarmCache).toHaveBeenCalledWith();
    });

    it('should not warm cache when disabled', async () => {
      const { result } = renderHook(() => useCacheWarming({ enabled: false }));

      await act(async () => {
        await result.current.warmFrequentRecipes();
      });

      expect(mockWarmCache).not.toHaveBeenCalled();
    });

    it('should not warm cache when offline', async () => {
      mockIsOffline.mockReturnValue(true);
      const { result } = renderHook(() => useCacheWarming());

      await act(async () => {
        await result.current.warmFrequentRecipes();
      });

      expect(mockWarmCache).not.toHaveBeenCalled();
    });

    it('should handle errors gracefully', async () => {
      mockWarmCache.mockRejectedValue(new Error('Cache warming failed'));
      const consoleSpy = jest.spyOn(console, 'error').mockImplementation();

      const { result } = renderHook(() => useCacheWarming());

      await act(async () => {
        await result.current.warmFrequentRecipes();
      });

      expect(consoleSpy).toHaveBeenCalledWith('Cache warming failed:', expect.any(Error));
      consoleSpy.mockRestore();
    });
  });

  describe('warmSpecificRecipes', () => {
    it('should warm cache for specific recipe IDs', async () => {
      const recipeIds = ['recipe-1', 'recipe-2', 'recipe-3'];
      const { result } = renderHook(() => useCacheWarming());

      await act(async () => {
        await result.current.warmSpecificRecipes(recipeIds);
      });

      expect(mockWarmCache).toHaveBeenCalledWith(recipeIds);
    });

    it('should limit recipes to maxRecipesToWarm', async () => {
      const recipeIds = Array.from({ length: 30 }, (_, i) => `recipe-${i}`);
      const { result } = renderHook(() => useCacheWarming({ maxRecipesToWarm: 10 }));

      await act(async () => {
        await result.current.warmSpecificRecipes(recipeIds);
      });

      expect(mockWarmCache).toHaveBeenCalledWith(recipeIds.slice(0, 10));
    });

    it('should not warm cache when offline', async () => {
      mockIsOffline.mockReturnValue(true);
      const { result } = renderHook(() => useCacheWarming());

      await act(async () => {
        await result.current.warmSpecificRecipes(['recipe-1']);
      });

      expect(mockWarmCache).not.toHaveBeenCalled();
    });
  });

  describe('app state changes', () => {
    let mockAddEventListener: jest.Mock;
    let mockRemoveEventListener: jest.Mock;

    beforeEach(() => {
      mockRemoveEventListener = jest.fn();
      mockAddEventListener = jest.fn().mockReturnValue({
        remove: mockRemoveEventListener,
      });
      (AppState.addEventListener as jest.Mock).mockImplementation(mockAddEventListener);
    });

    it('should register app state listener when onAppForeground is enabled', () => {
      renderHook(() => useCacheWarming({ onAppForeground: true }));

      expect(mockAddEventListener).toHaveBeenCalledWith('change', expect.any(Function));
    });

    it('should not register app state listener when onAppForeground is disabled', () => {
      renderHook(() => useCacheWarming({ onAppForeground: false }));

      expect(mockAddEventListener).not.toHaveBeenCalled();
    });

    it('should warm cache when app comes to foreground', async () => {
      renderHook(() => useCacheWarming({ onAppForeground: true }));

      const stateChangeHandler = mockAddEventListener.mock.calls[0][1];

      await act(async () => {
        stateChangeHandler('active');
      });

      expect(mockWarmCache).toHaveBeenCalled();
    });

    it('should not warm cache for other app state changes', async () => {
      renderHook(() => useCacheWarming({ onAppForeground: true }));

      const stateChangeHandler = mockAddEventListener.mock.calls[0][1];

      await act(async () => {
        stateChangeHandler('background');
      });

      expect(mockWarmCache).not.toHaveBeenCalled();
    });

    it('should remove listener on unmount', () => {
      const { unmount } = renderHook(() => useCacheWarming({ onAppForeground: true }));

      unmount();

      expect(mockRemoveEventListener).toHaveBeenCalled();
    });
  });

  describe('periodic cache warming', () => {
    it('should set up interval when enabled', () => {
      const setIntervalSpy = jest.spyOn(global, 'setInterval');
      const clearIntervalSpy = jest.spyOn(global, 'clearInterval');

      const { unmount } = renderHook(() => useCacheWarming({ 
        enabled: true, 
        intervalMinutes: 1 
      }));

      expect(setIntervalSpy).toHaveBeenCalledWith(expect.any(Function), 60000); // 1 minute

      unmount();
      expect(clearIntervalSpy).toHaveBeenCalled();

      setIntervalSpy.mockRestore();
      clearIntervalSpy.mockRestore();
    });

    it('should not set up interval when disabled', () => {
      const setIntervalSpy = jest.spyOn(global, 'setInterval');

      renderHook(() => useCacheWarming({ enabled: false }));

      expect(setIntervalSpy).not.toHaveBeenCalled();

      setIntervalSpy.mockRestore();
    });

    it('should not set up interval when intervalMinutes is 0', () => {
      const setIntervalSpy = jest.spyOn(global, 'setInterval');

      renderHook(() => useCacheWarming({ intervalMinutes: 0 }));

      expect(setIntervalSpy).not.toHaveBeenCalled();

      setIntervalSpy.mockRestore();
    });

    it('should warm cache on interval', async () => {
      renderHook(() => useCacheWarming({ 
        enabled: true, 
        intervalMinutes: 1 
      }));

      // Fast-forward time to trigger interval
      await act(async () => {
        jest.advanceTimersByTime(60000); // 1 minute
      });

      expect(mockWarmCache).toHaveBeenCalled();
    });
  });

  describe('network status changes', () => {
    it('should warm cache when coming back online', async () => {
      const { rerender } = renderHook(() => useCacheWarming());

      // Simulate coming back online
      mockUseRecipeStore.mockReturnValue({
        warmCache: mockWarmCache,
        isOffline: mockIsOffline,
        networkStatus: {
          isConnected: true,
          type: 'wifi',
          isInternetReachable: true,
        },
      } as any);

      await act(async () => {
        rerender();
        jest.advanceTimersByTime(2000); // Wait for 2-second delay
      });

      expect(mockWarmCache).toHaveBeenCalled();
    });

    it('should not warm cache when network status indicates offline', async () => {
      const { rerender } = renderHook(() => useCacheWarming());

      // Simulate going offline
      mockUseRecipeStore.mockReturnValue({
        warmCache: mockWarmCache,
        isOffline: mockIsOffline,
        networkStatus: {
          isConnected: false,
          type: 'none',
          isInternetReachable: false,
        },
      } as any);

      await act(async () => {
        rerender();
        jest.advanceTimersByTime(2000);
      });

      // warmCache should only be called during initial setup, not from network change
      expect(mockWarmCache).toHaveBeenCalledTimes(0);
    });
  });
});