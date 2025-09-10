/**
 * Lazy Loading Service Tests
 * 
 * Tests for React.lazy() implementation, code splitting,
 * and screen-level lazy loading functionality
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import { lazyLoadingService } from '../lazy_loading_service';
import type { ScreenMetadata, LazyLoadOptions } from '../lazy_loading_service';

// Mock AsyncStorage
jest.mock('@react-native-async-storage/async-storage');
const mockedAsyncStorage = AsyncStorage as jest.Mocked<typeof AsyncStorage>;

// Mock React Native components
jest.mock('react-native', () => ({
  ActivityIndicator: 'ActivityIndicator',
  View: 'View',
  StyleSheet: {
    create: (styles: any) => styles
  }
}));

describe('LazyLoadingService', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    lazyLoadingService.clearCache();
    mockedAsyncStorage.getItem.mockResolvedValue(null);
    mockedAsyncStorage.setItem.mockResolvedValue();
  });

  describe('createLazyScreen', () => {
    it('should create a lazy screen component with default options', () => {
      const mockComponent = () => null;
      const importFactory = jest.fn().mockResolvedValue({ default: mockComponent });

      const LazyScreen = lazyLoadingService.createLazyScreen(
        'TestScreen',
        importFactory
      );

      expect(LazyScreen).toBeDefined();
      expect(typeof LazyScreen).toBe('function');
    });

    it('should create lazy screen with custom options', () => {
      const mockComponent = () => null;
      const importFactory = jest.fn().mockResolvedValue({ default: mockComponent });
      const customOptions: LazyLoadOptions = {
        priority: 'critical',
        cacheStrategy: 'memory',
        errorBoundary: true,
        preloadTimeout: 500
      };

      const LazyScreen = lazyLoadingService.createLazyScreen(
        'CriticalScreen',
        importFactory,
        customOptions
      );

      expect(LazyScreen).toBeDefined();
    });

    it('should preload critical and high priority screens automatically', async () => {
      const mockComponent = () => null;
      const importFactory = jest.fn().mockResolvedValue({ default: mockComponent });

      lazyLoadingService.createLazyScreen(
        'CriticalScreen',
        importFactory,
        { priority: 'critical', cacheStrategy: 'memory' }
      );

      // Allow preload to complete
      await new Promise(resolve => setTimeout(resolve, 100));

      expect(importFactory).toHaveBeenCalled();
    });
  });

  describe('preloadScreen', () => {
    it('should preload screen and track loading progress', async () => {
      const mockComponent = () => null;
      const importFactory = jest.fn().mockResolvedValue({ default: mockComponent });

      await lazyLoadingService.preloadScreen(
        'PreloadTest',
        importFactory,
        { priority: 'high', cacheStrategy: 'memory' }
      );

      const progress = lazyLoadingService.getLoadingProgress();
      const screenProgress = progress.find(p => p.screenName === 'PreloadTest');

      expect(screenProgress).toBeDefined();
      expect(screenProgress?.loaded).toBe(true);
      expect(screenProgress?.loading).toBe(false);
      expect(screenProgress?.error).toBeNull();
      expect(screenProgress?.progress).toBe(100);
    });

    it('should handle preload failures gracefully', async () => {
      const importFactory = jest.fn().mockRejectedValue(new Error('Import failed'));

      try {
        await lazyLoadingService.preloadScreen(
          'FailingScreen',
          importFactory,
          { priority: 'normal', cacheStrategy: 'none' }
        );
      } catch (error) {
        // Expected to fail
      }

      const progress = lazyLoadingService.getLoadingProgress();
      const screenProgress = progress.find(p => p.screenName === 'FailingScreen');

      expect(screenProgress?.error).toBeDefined();
      expect(screenProgress?.loaded).toBe(false);
    });

    it('should return cached component for already loaded screens', async () => {
      const mockComponent = () => null;
      const importFactory = jest.fn().mockResolvedValue({ default: mockComponent });

      // Load screen first time
      await lazyLoadingService.preloadScreen(
        'CachedScreen',
        importFactory,
        { priority: 'normal', cacheStrategy: 'memory' }
      );

      // Load screen second time (should return cached version)
      await lazyLoadingService.preloadScreen(
        'CachedScreen',
        importFactory,
        { priority: 'normal', cacheStrategy: 'memory' }
      );

      expect(importFactory).toHaveBeenCalledTimes(1);
    });
  });

  describe('preloadCriticalScreens', () => {
    it('should preload screens based on priority', async () => {
      const screenDefinitions: ScreenMetadata[] = [
        {
          name: 'LoginScreen',
          route: 'Login',
          importPath: '../screens/auth/LoginScreen',
          priority: 'critical',
          estimatedSize: 45000
        },
        {
          name: 'ProfileScreen',
          route: 'Profile',
          importPath: '../screens/profile/ProfileScreen',
          priority: 'low',
          estimatedSize: 48000
        },
        {
          name: 'RecipeListScreen',
          route: 'RecipeList',
          importPath: '../screens/recipes/RecipeListScreen',
          priority: 'high',
          estimatedSize: 78000
        }
      ];

      // Mock dynamic imports
      const originalImport = global.__reanimatedWorkletInit;
      global.__reanimatedWorkletInit = jest.fn();
      
      jest.doMock('../screens/auth/LoginScreen', () => ({ default: () => null }), { virtual: true });
      jest.doMock('../screens/recipes/RecipeListScreen', () => ({ default: () => null }), { virtual: true });

      await lazyLoadingService.preloadCriticalScreens(screenDefinitions);

      const progress = lazyLoadingService.getLoadingProgress();
      
      // Should have attempted to load critical and high priority screens
      expect(progress.length).toBeGreaterThan(0);

      if (originalImport) {
        global.__reanimatedWorkletInit = originalImport;
      }
    });
  });

  describe('navigation tracking', () => {
    it('should record navigation patterns', async () => {
      lazyLoadingService.recordNavigation('HomeScreen', 'ProfileScreen');
      lazyLoadingService.recordNavigation('ProfileScreen', 'SettingsScreen');

      expect(mockedAsyncStorage.setItem).toHaveBeenCalledWith(
        'navigation_patterns',
        expect.stringContaining('ProfileScreen>SettingsScreen')
      );
    });

    it('should limit navigation patterns to 100 entries', async () => {
      // Record 150 navigation patterns
      for (let i = 0; i < 150; i++) {
        lazyLoadingService.recordNavigation(`Screen${i}`, `Screen${i + 1}`);
      }

      // Should save only the most recent 100 patterns
      expect(mockedAsyncStorage.setItem).toHaveBeenLastCalledWith(
        'navigation_patterns',
        expect.stringMatching(/.*/)
      );

      const lastCall = mockedAsyncStorage.setItem.mock.calls[mockedAsyncStorage.setItem.mock.calls.length - 1];
      const savedPatterns = JSON.parse(lastCall[1] as string);
      expect(savedPatterns.length).toBeLessThanOrEqual(100);
    });
  });

  describe('performance metrics', () => {
    it('should track performance metrics for loaded screens', async () => {
      const mockComponent = () => null;
      const importFactory = jest.fn().mockResolvedValue({ default: mockComponent });

      await lazyLoadingService.preloadScreen(
        'MetricsTest',
        importFactory,
        { priority: 'normal', cacheStrategy: 'memory' }
      );

      const metrics = lazyLoadingService.getPerformanceMetrics();

      expect(metrics.totalScreens).toBe(1);
      expect(metrics.loadedScreens).toBe(1);
      expect(metrics.averageLoadTime).toBeGreaterThanOrEqual(0);
      expect(metrics.failedLoads).toBe(0);
    });

    it('should track failed loads in performance metrics', async () => {
      const importFactory = jest.fn().mockRejectedValue(new Error('Failed'));

      try {
        await lazyLoadingService.preloadScreen(
          'FailedMetricsTest',
          importFactory,
          { priority: 'normal', cacheStrategy: 'none' }
        );
      } catch {
        // Expected failure
      }

      const metrics = lazyLoadingService.getPerformanceMetrics();
      expect(metrics.failedLoads).toBe(1);
    });
  });

  describe('cache management', () => {
    it('should clear all caches when clearCache is called', async () => {
      const mockComponent = () => null;
      const importFactory = jest.fn().mockResolvedValue({ default: mockComponent });

      // Load a screen to populate cache
      await lazyLoadingService.preloadScreen(
        'CacheClearTest',
        importFactory,
        { priority: 'normal', cacheStrategy: 'memory' }
      );

      expect(lazyLoadingService.getLoadingProgress().length).toBe(1);

      // Clear cache
      lazyLoadingService.clearCache();

      expect(lazyLoadingService.getLoadingProgress().length).toBe(0);
    });
  });

  describe('loading progress tracking', () => {
    it('should initialize loading progress when creating lazy screen', () => {
      const importFactory = jest.fn().mockResolvedValue({ default: () => null });

      lazyLoadingService.createLazyScreen('ProgressTest', importFactory);

      const progress = lazyLoadingService.getLoadingProgress();
      const screenProgress = progress.find(p => p.screenName === 'ProgressTest');

      expect(screenProgress).toBeDefined();
      expect(screenProgress?.loaded).toBe(false);
      expect(screenProgress?.loading).toBe(false);
      expect(screenProgress?.error).toBeNull();
      expect(screenProgress?.progress).toBe(0);
    });
  });
});