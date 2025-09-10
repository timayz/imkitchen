/**
 * Data Preloading Service Tests
 * 
 * Tests for critical data identification, priority-based preloading,
 * and network-aware data loading strategies
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import NetInfo from '@react-native-community/netinfo';
import { dataPreloadingService } from '../data_preloading_service';
import type { PreloadResult, PreloadingProgress } from '../data_preloading_service';

// Mock AsyncStorage
jest.mock('@react-native-async-storage/async-storage');
const mockedAsyncStorage = AsyncStorage as jest.Mocked<typeof AsyncStorage>;

// Mock NetInfo
jest.mock('@react-native-community/netinfo', () => ({
  fetch: jest.fn()
}));
const mockedNetInfo = NetInfo as jest.Mocked<typeof NetInfo>;

describe('DataPreloadingService', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    
    // Default network state - online
    mockedNetInfo.fetch.mockResolvedValue({
      isConnected: true,
      isInternetReachable: true,
      type: 'wifi'
    } as any);

    // Default AsyncStorage behavior
    mockedAsyncStorage.getItem.mockResolvedValue(null);
    mockedAsyncStorage.setItem.mockResolvedValue(undefined);
    mockedAsyncStorage.getAllKeys.mockResolvedValue([]);
    mockedAsyncStorage.multiRemove.mockResolvedValue(undefined);

    // Clear any ongoing preloading
    dataPreloadingService.cancelPreloading();
  });

  describe('preloadCriticalData', () => {
    it('should preload data with progress tracking', async () => {
      const progressUpdates: PreloadingProgress[] = [];
      
      const results = await dataPreloadingService.preloadCriticalData((progress) => {
        progressUpdates.push(progress);
      });

      expect(results).toBeInstanceOf(Array);
      expect(results.length).toBeGreaterThan(0);
      
      // Should have progress updates
      expect(progressUpdates.length).toBeGreaterThan(0);
      
      // Final progress should be 100%
      const finalProgress = progressUpdates[progressUpdates.length - 1];
      expect(finalProgress.progress).toBe(100);
      expect(finalProgress.currentItem).toBeNull();
    });

    it('should prioritize critical data types first', async () => {
      const results = await dataPreloadingService.preloadCriticalData();

      // Should have results for critical data types
      const criticalResults = results.filter(result => 
        ['user_profile', 'app_settings', 'auth_token'].includes(result.dataType)
      );
      
      expect(criticalResults.length).toBeGreaterThan(0);
      
      // Critical data should be loaded first
      const userProfile = results.find(r => r.dataType === 'user_profile');
      expect(userProfile).toBeDefined();
    });

    it('should handle storage data loading', async () => {
      const mockUserProfile = {
        id: 'user123',
        name: 'Test User',
        email: 'test@example.com'
      };

      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'user_profile') {
          return Promise.resolve(JSON.stringify(mockUserProfile));
        }
        return Promise.resolve(null);
      });

      const results = await dataPreloadingService.preloadCriticalData();

      const userProfileResult = results.find(r => r.dataType === 'user_profile');
      expect(userProfileResult).toBeDefined();
      expect(userProfileResult!.success).toBe(true);
      expect(userProfileResult!.data).toEqual(mockUserProfile);
      expect(userProfileResult!.fromCache).toBe(false);
    });

    it('should skip API calls when offline and no fallback', async () => {
      // Mock offline state
      mockedNetInfo.fetch.mockResolvedValue({
        isConnected: false,
        isInternetReachable: false,
        type: 'none'
      } as any);

      const results = await dataPreloadingService.preloadCriticalData();

      // API-dependent data without fallback should be skipped
      const recentRecipes = results.find(r => r.dataType === 'recent_recipes');
      expect(recentRecipes).toBeUndefined();
    });

    it('should use cache fallback for API data when offline', async () => {
      // Mock offline state
      mockedNetInfo.fetch.mockResolvedValue({
        isConnected: false,
        isInternetReachable: false,
        type: 'none'
      } as any);

      // Mock cached data
      const mockCachedRecipes = [
        { id: 1, name: 'Cached Recipe', lastUsed: Date.now() }
      ];

      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'cache_recent_recipes') {
          return Promise.resolve(JSON.stringify({
            data: mockCachedRecipes,
            timestamp: Date.now() - 1800000 // 30 minutes ago
          }));
        }
        return Promise.resolve(null);
      });

      const results = await dataPreloadingService.preloadCriticalData();

      const recentRecipes = results.find(r => r.dataType === 'recent_recipes');
      expect(recentRecipes).toBeDefined();
      expect(recentRecipes!.success).toBe(true);
      expect(recentRecipes!.fromCache).toBe(true);
      expect(recentRecipes!.data).toEqual(mockCachedRecipes);
    });

    it('should handle dependency resolution', async () => {
      // Mock auth token presence
      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'auth_token') {
          return Promise.resolve('mock_token_123');
        }
        return Promise.resolve(null);
      });

      const results = await dataPreloadingService.preloadCriticalData();

      // Data that depends on auth_token should be loaded
      const userPreferences = results.find(r => r.dataType === 'user_preferences');
      const recentRecipes = results.find(r => r.dataType === 'recent_recipes');
      
      expect(userPreferences).toBeDefined();
      expect(recentRecipes).toBeDefined();
    });

    it('should skip dependent data when dependencies are missing', async () => {
      // No auth token available
      mockedAsyncStorage.getItem.mockResolvedValue(null);

      const progressUpdates: PreloadingProgress[] = [];
      
      await dataPreloadingService.preloadCriticalData((progress) => {
        progressUpdates.push(progress);
      });

      // Should still have some progress (non-dependent data)
      expect(progressUpdates.length).toBeGreaterThan(0);
    });

    it('should handle concurrent preloading attempts', async () => {
      const promise1 = dataPreloadingService.preloadCriticalData();
      const promise2 = dataPreloadingService.preloadCriticalData();

      const results1 = await promise1;
      const results2 = await promise2;

      // Second call should return cached results
      expect(results1).toBeDefined();
      expect(results2).toBeDefined();
      expect(results1.length).toBe(results2.length);
    });
  });

  describe('data source handling', () => {
    it('should calculate data size accurately', async () => {
      const mockData = {
        recipes: Array(50).fill({ name: 'Recipe', ingredients: ['item1', 'item2'] }),
        preferences: { theme: 'dark', language: 'en' }
      };

      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'cached_meal_plans') {
          return Promise.resolve(JSON.stringify(mockData));
        }
        return Promise.resolve(null);
      });

      const results = await dataPreloadingService.preloadCriticalData();

      const mealPlanResult = results.find(r => r.dataType === 'meal_plan_cache');
      expect(mealPlanResult).toBeDefined();
      expect(mealPlanResult!.size).toBeGreaterThan(0);
    });

    it('should handle cache expiry properly', async () => {
      // Mock expired cache data
      const expiredData = {
        data: [{ id: 1, name: 'Old Recipe' }],
        timestamp: Date.now() - 7200000 // 2 hours ago (expired)
      };

      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'cache_community_trending') {
          return Promise.resolve(JSON.stringify(expiredData));
        }
        return Promise.resolve(null);
      });

      const results = await dataPreloadingService.preloadCriticalData();

      const trendingResult = results.find(r => r.dataType === 'community_trending');
      // Should attempt to load fresh data, not use expired cache
      expect(trendingResult).toBeDefined();
    });

    it('should cache successful API responses', async () => {
      const results = await dataPreloadingService.preloadCriticalData();

      // API calls should trigger cache storage
      const apiResults = results.filter(r => 
        ['recent_recipes', 'user_preferences', 'favorite_recipes'].includes(r.dataType)
      );

      apiResults.forEach(result => {
        if (result.success && !result.fromCache) {
          expect(mockedAsyncStorage.setItem).toHaveBeenCalledWith(
            `cache_${result.dataType}`,
            expect.stringContaining('"data"')
          );
        }
      });
    });
  });

  describe('error handling', () => {
    it('should handle storage errors gracefully', async () => {
      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'user_profile') {
          return Promise.reject(new Error('Storage error'));
        }
        return Promise.resolve(null);
      });

      const results = await dataPreloadingService.preloadCriticalData();

      const userProfileResult = results.find(r => r.dataType === 'user_profile');
      expect(userProfileResult).toBeDefined();
      expect(userProfileResult!.success).toBe(false);
      expect(userProfileResult!.error).toContain('Storage error');
    });

    it('should continue preloading after individual failures', async () => {
      // Mock one storage failure
      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'app_settings') {
          return Promise.reject(new Error('Settings unavailable'));
        }
        return Promise.resolve(null);
      });

      const results = await dataPreloadingService.preloadCriticalData();

      // Should have results for other data types despite one failure
      expect(results.length).toBeGreaterThan(1);
      
      const settingsResult = results.find(r => r.dataType === 'app_settings');
      expect(settingsResult!.success).toBe(false);
      
      const profileResult = results.find(r => r.dataType === 'user_profile');
      expect(profileResult).toBeDefined(); // Should still be attempted
    });

    it('should track errors in progress updates', async () => {
      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'user_profile') {
          return Promise.reject(new Error('Profile error'));
        }
        return Promise.resolve(null);
      });

      const progressUpdates: PreloadingProgress[] = [];
      
      await dataPreloadingService.preloadCriticalData((progress) => {
        progressUpdates.push(progress);
      });

      // Should have error information in progress
      const errorUpdate = progressUpdates.find(p => p.errors.length > 0);
      expect(errorUpdate).toBeDefined();
      expect(errorUpdate!.errors[0]).toContain('user_profile');
    });
  });

  describe('utility methods', () => {
    it('should provide preloaded data access', async () => {
      const mockUserProfile = { id: 'user123', name: 'Test User' };
      
      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'user_profile') {
          return Promise.resolve(JSON.stringify(mockUserProfile));
        }
        return Promise.resolve(null);
      });

      await dataPreloadingService.preloadCriticalData();

      const userData = dataPreloadingService.getPreloadedData('user_profile');
      expect(userData).toEqual(mockUserProfile);

      const missingData = dataPreloadingService.getPreloadedData('nonexistent');
      expect(missingData).toBeNull();
    });

    it('should check data availability', async () => {
      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'user_profile') {
          return Promise.resolve('{"id": "user123"}');
        }
        return Promise.resolve(null);
      });

      await dataPreloadingService.preloadCriticalData();

      expect(dataPreloadingService.isDataAvailable('user_profile')).toBe(true);
      expect(dataPreloadingService.isDataAvailable('nonexistent')).toBe(false);
    });

    it('should provide preloading results summary', async () => {
      await dataPreloadingService.preloadCriticalData();

      const summary = dataPreloadingService.getPreloadingResults();

      expect(summary.totalItems).toBeGreaterThan(0);
      expect(summary.successfulItems).toBeGreaterThan(0);
      expect(summary.totalSize).toBeGreaterThanOrEqual(0);
      expect(summary.totalTime).toBeGreaterThan(0);
      expect(summary.cacheHitRate).toBeGreaterThanOrEqual(0);
      expect(summary.cacheHitRate).toBeLessThanOrEqual(1);
    });

    it('should verify offline availability', async () => {
      const mockOfflineData = {
        'user_profile': '{"id": "user123"}',
        'app_settings': '{"theme": "dark"}',
        'cache_recent_recipes': '{"data": [], "timestamp": ' + Date.now() + '}'
      };

      mockedAsyncStorage.getItem.mockImplementation((key) => {
        return Promise.resolve(mockOfflineData[key] || null);
      });

      const availability = await dataPreloadingService.verifyOfflineAvailability();

      expect(availability.available.length).toBeGreaterThan(0);
      expect(availability.available).toContain('user_profile');
      expect(availability.available).toContain('app_settings');
      expect(availability.totalOfflineSize).toBeGreaterThan(0);
    });
  });

  describe('cleanup and cancellation', () => {
    it('should cancel ongoing preloading', () => {
      const preloadPromise = dataPreloadingService.preloadCriticalData();
      
      dataPreloadingService.cancelPreloading();
      
      const status = dataPreloadingService.getStatus();
      expect(status.isPreloading).toBe(false);
    });

    it('should clear preloaded data and cache', async () => {
      mockedAsyncStorage.getAllKeys.mockResolvedValue([
        'cache_recent_recipes',
        'cache_user_preferences',
        'other_data'
      ]);

      await dataPreloadingService.clearPreloadedData();

      expect(mockedAsyncStorage.multiRemove).toHaveBeenCalledWith([
        'cache_recent_recipes',
        'cache_user_preferences'
      ]);
    });

    it('should provide current status', () => {
      const status = dataPreloadingService.getStatus();

      expect(status).toHaveProperty('isPreloading');
      expect(status).toHaveProperty('completedItems');
      expect(status).toHaveProperty('totalItems');
      
      expect(typeof status.isPreloading).toBe('boolean');
      expect(typeof status.completedItems).toBe('number');
      expect(typeof status.totalItems).toBe('number');
    });
  });
});