/**
 * Screen Registry Tests
 * 
 * Tests for screen registration, lazy loading integration,
 * and bundle size analytics
 */

// Test file for Screen Registry functionality
import { screenRegistry, ScreenPriority, ScreenCategory } from '../ScreenRegistry';
import { lazyLoadingService } from '../../services/lazy_loading_service';

// Mock the lazy loading service
jest.mock('../../services/lazy_loading_service', () => ({
  lazyLoadingService: {
    createLazyScreen: jest.fn().mockReturnValue(() => null),
    preloadScreen: jest.fn().mockResolvedValue(() => null),
    preloadCriticalScreens: jest.fn().mockResolvedValue(undefined),
    recordNavigation: jest.fn(),
    getPerformanceMetrics: jest.fn().mockReturnValue({
      totalScreens: 0,
      loadedScreens: 0,
      averageLoadTime: 0,
      failedLoads: 0
    })
  }
}));

// Mock console methods to avoid test output noise
const consoleSpy = jest.spyOn(console, 'log').mockImplementation(() => {});
const consoleWarnSpy = jest.spyOn(console, 'warn').mockImplementation(() => {});

describe('ScreenRegistry', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  afterAll(() => {
    consoleSpy.mockRestore();
    consoleWarnSpy.mockRestore();
  });

  describe('screen registration', () => {
    it('should provide all registered screen names', () => {
      const screenNames = screenRegistry.getScreenNames();
      
      // Should include key screens from the registration
      expect(screenNames).toContain('LoginScreen');
      expect(screenNames).toContain('RecipeListScreen');
      expect(screenNames).toContain('MealPlanScreen');
      expect(screenNames).toContain('ProfileScreen');
      expect(screenNames.length).toBeGreaterThan(10);
    });

    it('should categorize screens correctly', () => {
      const authScreens = screenRegistry.getScreensByCategory(ScreenCategory.AUTH);
      const recipeScreens = screenRegistry.getScreensByCategory(ScreenCategory.RECIPES);
      const profileScreens = screenRegistry.getScreensByCategory(ScreenCategory.PROFILE);

      expect(authScreens.length).toBeGreaterThan(0);
      expect(recipeScreens.length).toBeGreaterThan(0);
      expect(profileScreens.length).toBeGreaterThan(0);

      // Verify auth screens include expected ones
      const authScreenNames = authScreens.map(s => s.name);
      expect(authScreenNames).toContain('LoginScreen');
      expect(authScreenNames).toContain('RegisterScreen');
    });

    it('should assign correct priorities to screens', () => {
      const loginMetadata = screenRegistry.getScreenMetadata('LoginScreen');
      const profileMetadata = screenRegistry.getScreenMetadata('ProfileScreen');
      const recipeListMetadata = screenRegistry.getScreenMetadata('RecipeListScreen');

      expect(loginMetadata?.priority).toBe(ScreenPriority.CRITICAL);
      expect(profileMetadata?.priority).toBe(ScreenPriority.LOW);
      expect(recipeListMetadata?.priority).toBe(ScreenPriority.HIGH);
    });
  });

  describe('lazy screen creation', () => {
    it('should create lazy screen components', () => {
      const mockCreateLazyScreen = lazyLoadingService.createLazyScreen as jest.MockedFunction<typeof lazyLoadingService.createLazyScreen>;
      mockCreateLazyScreen.mockReturnValue(() => null);

      const LazyScreen = screenRegistry.getScreen('LoginScreen');

      expect(LazyScreen).toBeDefined();
      expect(mockCreateLazyScreen).toHaveBeenCalledWith(
        'LoginScreen',
        expect.any(Function), // importFactory
        expect.objectContaining({
          priority: ScreenPriority.CRITICAL,
          cacheStrategy: 'memory',
          errorBoundary: true
        })
      );
    });

    it('should throw error for non-existent screens', () => {
      expect(() => {
        screenRegistry.getScreen('NonExistentScreen');
      }).toThrow("Screen 'NonExistentScreen' not found in registry");
    });

    it('should cache created lazy screens', () => {
      const mockCreateLazyScreen = lazyLoadingService.createLazyScreen as jest.MockedFunction<typeof lazyLoadingService.createLazyScreen>;
      mockCreateLazyScreen.mockReturnValue(() => null);

      // Get screen twice
      screenRegistry.getScreen('LoginScreen');
      screenRegistry.getScreen('LoginScreen');

      // Should only create lazy screen once
      expect(mockCreateLazyScreen).toHaveBeenCalledTimes(1);
    });
  });

  describe('bundle analytics', () => {
    it('should provide comprehensive bundle size analytics', () => {
      const analytics = screenRegistry.getBundleAnalytics();

      expect(analytics.totalEstimatedSize).toBeGreaterThan(0);
      expect(Object.keys(analytics.sizeByCategory)).toContain('auth');
      expect(Object.keys(analytics.sizeByCategory)).toContain('recipes');
      expect(Object.keys(analytics.sizeByPriority)).toContain('critical');
      expect(Object.keys(analytics.sizeByPriority)).toContain('high');
      expect(analytics.largestScreens.length).toBeGreaterThan(0);
    });

    it('should identify largest screens correctly', () => {
      const analytics = screenRegistry.getBundleAnalytics();
      const largestScreen = analytics.largestScreens[0];

      expect(largestScreen).toHaveProperty('name');
      expect(largestScreen).toHaveProperty('size');
      expect(largestScreen.size).toBeGreaterThan(0);

      // Verify screens are sorted by size (largest first)
      for (let i = 0; i < analytics.largestScreens.length - 1; i++) {
        expect(analytics.largestScreens[i].size).toBeGreaterThanOrEqual(
          analytics.largestScreens[i + 1].size
        );
      }
    });

    it('should calculate size by category correctly', () => {
      const analytics = screenRegistry.getBundleAnalytics();
      const authScreens = screenRegistry.getScreensByCategory(ScreenCategory.AUTH);
      const expectedAuthSize = authScreens.reduce((sum, screen) => sum + screen.estimatedSize, 0);

      expect(analytics.sizeByCategory[ScreenCategory.AUTH]).toBe(expectedAuthSize);
    });
  });

  describe('initialization', () => {
    it('should initialize and preload critical screens', async () => {
      const mockPreloadCriticalScreens = lazyLoadingService.preloadCriticalScreens as jest.MockedFunction<typeof lazyLoadingService.preloadCriticalScreens>;
      
      await screenRegistry.initialize();

      expect(mockPreloadCriticalScreens).toHaveBeenCalledWith(
        expect.arrayContaining([
          expect.objectContaining({
            name: expect.any(String),
            priority: expect.any(String),
            estimatedSize: expect.any(Number)
          })
        ])
      );
    });

    it('should handle initialization errors gracefully', async () => {
      const mockPreloadCriticalScreens = lazyLoadingService.preloadCriticalScreens as jest.MockedFunction<typeof lazyLoadingService.preloadCriticalScreens>;
      mockPreloadCriticalScreens.mockRejectedValueOnce(new Error('Initialization failed'));

      // Should not throw error
      await expect(screenRegistry.initialize()).resolves.not.toThrow();
    });
  });

  describe('navigation tracking', () => {
    it('should record navigation patterns', () => {
      const mockRecordNavigation = lazyLoadingService.recordNavigation as jest.MockedFunction<typeof lazyLoadingService.recordNavigation>;

      screenRegistry.recordNavigation('LoginScreen', 'RecipeListScreen');

      expect(mockRecordNavigation).toHaveBeenCalledWith('LoginScreen', 'RecipeListScreen');
    });
  });

  describe('performance metrics integration', () => {
    it('should provide performance metrics from lazy loading service', () => {
      const mockMetrics = {
        totalScreens: 5,
        loadedScreens: 3,
        averageLoadTime: 150,
        failedLoads: 1
      };

      const mockGetPerformanceMetrics = lazyLoadingService.getPerformanceMetrics as jest.MockedFunction<typeof lazyLoadingService.getPerformanceMetrics>;
      mockGetPerformanceMetrics.mockReturnValue(mockMetrics);

      const metrics = screenRegistry.getPerformanceMetrics();

      expect(metrics).toEqual(mockMetrics);
      expect(mockGetPerformanceMetrics).toHaveBeenCalled();
    });
  });

  describe('preload dependencies', () => {
    it('should handle screens with preload dependencies', () => {
      const recipeListMetadata = screenRegistry.getScreenMetadata('RecipeListScreen');
      const mealPlanMetadata = screenRegistry.getScreenMetadata('MealPlanScreen');

      expect(recipeListMetadata?.preloadDependencies).toContain('RecipeDetailScreen');
      expect(mealPlanMetadata?.preloadDependencies).toContain('ShoppingListScreen');
    });
  });

  describe('priority-based caching strategies', () => {
    it('should use memory caching for critical screens', () => {
      const mockCreateLazyScreen = lazyLoadingService.createLazyScreen as jest.MockedFunction<typeof lazyLoadingService.createLazyScreen>;
      mockCreateLazyScreen.mockReturnValue(() => null);

      screenRegistry.getScreen('LoginScreen'); // Critical priority

      expect(mockCreateLazyScreen).toHaveBeenCalledWith(
        'LoginScreen',
        expect.any(Function),
        expect.objectContaining({
          cacheStrategy: 'memory'
        })
      );
    });

    it('should use storage caching for low priority screens', () => {
      const mockCreateLazyScreen = lazyLoadingService.createLazyScreen as jest.MockedFunction<typeof lazyLoadingService.createLazyScreen>;
      mockCreateLazyScreen.mockReturnValue(() => null);

      screenRegistry.getScreen('ProfileScreen'); // Low priority

      expect(mockCreateLazyScreen).toHaveBeenCalledWith(
        'ProfileScreen',
        expect.any(Function),
        expect.objectContaining({
          cacheStrategy: 'storage'
        })
      );
    });
  });
});