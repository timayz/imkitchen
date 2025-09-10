/**
 * Startup Optimization Test Suite
 * 
 * Comprehensive testing for Task 5: Mobile App Startup Optimization components
 * including lazy loading, bundle optimization, splash screen, data preloading,
 * and performance monitoring services.
 */

import {
  lazyLoadingService,
  ScreenMetadata,
  LazyLoadOptions
} from '../lazy_loading_service';
import {
  bundleOptimizationService,
  DependencyInfo,
  OptimizationRecommendation
} from '../bundle_optimization_service';
import {
  criticalDataPreloader,
  PreloadItem
} from '../critical_data_preloader';
import {
  startupMetricsService,
  StartupMetrics,
  PerformanceRecommendation
} from '../startup_metrics_service';
import { screenRegistry, ScreenPriority } from '../../navigation/ScreenRegistry';

// Mock AsyncStorage
jest.mock('@react-native-async-storage/async-storage', () => ({
  getItem: jest.fn(() => Promise.resolve(null)),
  setItem: jest.fn(() => Promise.resolve()),
  removeItem: jest.fn(() => Promise.resolve()),
}));

// Mock React Native modules
jest.mock('react-native', () => ({
  Platform: { OS: 'ios', Version: '14.0' },
  Dimensions: { get: () => ({ width: 375, height: 812 }) },
  ActivityIndicator: 'ActivityIndicator',
  View: 'View',
  Text: 'Text',
  StyleSheet: { create: (styles: any) => styles },
}));

jest.mock('react-native-device-info', () => ({
  getModel: () => Promise.resolve('iPhone 12'),
  getSystemVersion: () => Promise.resolve('14.0'),
  isEmulator: () => Promise.resolve(false),
  getTotalMemory: () => Promise.resolve(6442450944),
  getUsedMemory: () => Promise.resolve(3221225472),
  getVersion: () => Promise.resolve('1.0.0'),
  getBuildNumber: () => Promise.resolve('1'),
}));

jest.mock('@react-native-netinfo/', () => ({
  fetch: () => Promise.resolve({ isConnected: true, type: 'wifi' }),
  addEventListener: jest.fn(),
}));

describe('Lazy Loading Service', () => {
  beforeEach(() => {
    lazyLoadingService.clearCache();
  });

  test('should create lazy screen component', () => {
    const importFactory = jest.fn(() => Promise.resolve({ default: jest.fn() }));
    const options: LazyLoadOptions = {
      priority: 'high',
      cacheStrategy: 'memory',
      errorBoundary: true
    };

    const LazyScreen = lazyLoadingService.createLazyScreen('TestScreen', importFactory, options);
    
    expect(LazyScreen).toBeDefined();
    expect(typeof LazyScreen).toBe('function');
  });

  test('should preload screen with correct timing', async () => {
    const mockComponent = jest.fn();
    const importFactory = jest.fn(() => Promise.resolve({ default: mockComponent }));
    
    const startTime = Date.now();
    const result = await lazyLoadingService.preloadScreen(
      'PreloadTestScreen',
      importFactory,
      { priority: 'critical', cacheStrategy: 'memory' }
    );
    const endTime = Date.now();

    expect(result).toBe(mockComponent);
    expect(importFactory).toHaveBeenCalledTimes(1);
    expect(endTime - startTime).toBeLessThan(1000); // Should be fast
  });

  test('should handle preload errors gracefully', async () => {
    const importFactory = jest.fn(() => Promise.reject(new Error('Load failed')));
    
    await expect(
      lazyLoadingService.preloadScreen('ErrorScreen', importFactory, {
        priority: 'normal',
        cacheStrategy: 'memory'
      })
    ).rejects.toThrow('Load failed');

    const progress = lazyLoadingService.getLoadingProgress();
    const errorScreen = progress.find(p => p.screenName === 'ErrorScreen');
    expect(errorScreen?.error).toBeTruthy();
  });

  test('should record navigation patterns', () => {
    lazyLoadingService.recordNavigation('HomeScreen', 'RecipeListScreen');
    lazyLoadingService.recordNavigation('RecipeListScreen', 'RecipeDetailScreen');
    
    // Navigation patterns are recorded internally
    // This test ensures the method doesn't throw errors
    expect(true).toBe(true);
  });

  test('should provide performance metrics', () => {
    const metrics = lazyLoadingService.getPerformanceMetrics();
    
    expect(metrics).toHaveProperty('totalScreens');
    expect(metrics).toHaveProperty('loadedScreens');
    expect(metrics).toHaveProperty('averageLoadTime');
    expect(metrics).toHaveProperty('failedLoads');
    expect(typeof metrics.totalScreens).toBe('number');
  });
});

describe('Bundle Optimization Service', () => {
  test('should analyze bundle size', async () => {
    const analysis = await bundleOptimizationService.analyzeBundleSize();
    
    expect(analysis).toHaveProperty('totalSize');
    expect(analysis).toHaveProperty('jsSize');
    expect(analysis).toHaveProperty('assetsSize');
    expect(analysis).toHaveProperty('dependencies');
    expect(analysis).toHaveProperty('recommendations');
    expect(Array.isArray(analysis.dependencies)).toBe(true);
    expect(Array.isArray(analysis.recommendations)).toBe(true);
  });

  test('should generate optimization recommendations', async () => {
    const analysis = await bundleOptimizationService.analyzeBundleSize();
    const recommendations = analysis.recommendations;
    
    expect(recommendations.length).toBeGreaterThan(0);
    
    const criticalRecommendations = recommendations.filter(r => r.severity === 'critical');
    const highRecommendations = recommendations.filter(r => r.severity === 'high');
    
    // Should prioritize recommendations by severity
    if (criticalRecommendations.length > 0 && highRecommendations.length > 0) {
      expect(recommendations.indexOf(criticalRecommendations[0])).toBeLessThan(
        recommendations.indexOf(highRecommendations[0])
      );
    }
  });

  test('should optimize assets', async () => {
    const optimizations = await bundleOptimizationService.optimizeAssets();
    
    expect(Array.isArray(optimizations)).toBe(true);
    
    optimizations.forEach(opt => {
      expect(opt).toHaveProperty('type');
      expect(opt).toHaveProperty('originalSize');
      expect(opt).toHaveProperty('optimizedSize');
      expect(opt).toHaveProperty('compressionRatio');
      expect(opt.optimizedSize).toBeLessThan(opt.originalSize);
    });
  });

  test('should analyze import patterns', async () => {
    const importAnalysis = await bundleOptimizationService.analyzeImports();
    
    expect(importAnalysis).toHaveProperty('unusedImports');
    expect(importAnalysis).toHaveProperty('circularDependencies');
    expect(importAnalysis).toHaveProperty('heavyImports');
    expect(Array.isArray(importAnalysis.unusedImports)).toBe(true);
  });

  test('should generate optimization report', async () => {
    const report = await bundleOptimizationService.generateOptimizationReport();
    
    expect(typeof report).toBe('string');
    expect(report).toContain('Bundle Optimization Report');
    expect(report).toContain('Current Bundle Size');
    expect(report).toContain('Top Recommendations');
  });

  test('should track bundle metrics over time', async () => {
    const metrics = await bundleOptimizationService.getBundleMetrics();
    
    expect(metrics).toHaveProperty('currentSize');
    expect(metrics).toHaveProperty('trend');
    expect(['increasing', 'decreasing', 'stable']).toContain(metrics.trend);
  });
});

describe('Critical Data Preloader', () => {
  test('should register preload items', () => {
    const testItem: PreloadItem = {
      id: 'test_item',
      name: 'Test Item',
      priority: 'high',
      dependencies: [],
      loader: () => Promise.resolve({ data: 'test' })
    };

    criticalDataPreloader.register(testItem);
    
    // Verify registration by checking progress
    const progress = criticalDataPreloader.getProgress();
    const testProgress = progress.find(p => p.itemId === 'test_item');
    expect(testProgress).toBeTruthy();
    expect(testProgress?.status).toBe('pending');
  });

  test('should preload critical data with proper prioritization', async () => {
    const startTime = Date.now();
    const result = await criticalDataPreloader.preloadCriticalData();
    const endTime = Date.now();
    
    expect(result).toHaveProperty('success');
    expect(result).toHaveProperty('completedItems');
    expect(result).toHaveProperty('failedItems');
    expect(result).toHaveProperty('totalTime');
    expect(result.totalTime).toBeLessThan(10000); // Should complete within 10s
    expect(endTime - startTime).toBeGreaterThanOrEqual(result.totalTime);
  });

  test('should handle preload failures gracefully', async () => {
    const failingItem: PreloadItem = {
      id: 'failing_item',
      name: 'Failing Item',
      priority: 'critical',
      dependencies: [],
      loader: () => Promise.reject(new Error('Network error')),
      fallback: () => Promise.resolve({ fallback: true })
    };

    criticalDataPreloader.register(failingItem);
    
    const result = await criticalDataPreloader.preloadCriticalData();
    
    // Should succeed due to fallback
    expect(result.success).toBe(true);
    expect(result.completedItems).toContain('failing_item');
  });

  test('should provide preload statistics', () => {
    const stats = criticalDataPreloader.getStatistics();
    
    expect(stats).toHaveProperty('totalItems');
    expect(stats).toHaveProperty('completedItems');
    expect(stats).toHaveProperty('failedItems');
    expect(stats).toHaveProperty('averageLoadTime');
    expect(stats).toHaveProperty('cacheHitRate');
    expect(typeof stats.totalItems).toBe('number');
    expect(stats.cacheHitRate).toBeGreaterThanOrEqual(0);
    expect(stats.cacheHitRate).toBeLessThanOrEqual(1);
  });
});

describe('Startup Metrics Service', () => {
  beforeEach(() => {
    startupMetricsService.startMeasuring();
  });

  test('should measure startup phases', () => {
    startupMetricsService.startPhase('test_phase');
    startupMetricsService.recordMetric('test_phase', 'operations', 10);
    startupMetricsService.endPhase('test_phase');
    
    const metrics = startupMetricsService.getCurrentMetrics();
    expect(metrics?.phases).toHaveProperty('test_phase');
    expect(metrics?.phases.test_phase).toBeGreaterThan(0);
  });

  test('should record startup completion', () => {
    const startTime = Date.now();
    startupMetricsService.recordStartupTime(1500);
    startupMetricsService.recordFirstScreenRender();
    
    const metrics = startupMetricsService.getCurrentMetrics();
    expect(metrics?.totalStartupTime).toBe(1500);
    expect(metrics?.firstScreenRenderTime).toBeGreaterThan(0);
    expect(metrics?.scores.overall).toBeGreaterThan(0);
    expect(metrics?.scores.overall).toBeLessThanOrEqual(100);
  });

  test('should generate performance recommendations', () => {
    // Simulate poor performance
    startupMetricsService.recordStartupTime(6000); // 6 seconds
    startupMetricsService.recordBundleLoadTime(3000);
    
    const metrics = startupMetricsService.getCurrentMetrics();
    const recommendations = metrics?.recommendations || [];
    
    expect(recommendations.length).toBeGreaterThan(0);
    expect(recommendations[0]).toHaveProperty('category');
    expect(recommendations[0]).toHaveProperty('severity');
    expect(recommendations[0]).toHaveProperty('title');
    expect(recommendations[0]).toHaveProperty('solution');
  });

  test('should calculate performance scores', () => {
    startupMetricsService.recordStartupTime(2000); // Good performance
    
    const metrics = startupMetricsService.getCurrentMetrics();
    expect(metrics?.scores.overall).toBeGreaterThan(70); // Should be good score
    expect(metrics?.scores.loadingSpeed).toBeGreaterThan(70);
    expect(metrics?.scores.responsiveness).toBeGreaterThan(0);
  });

  test('should generate performance report', () => {
    startupMetricsService.recordStartupTime(1800);
    startupMetricsService.recordFirstScreenRender();
    
    const report = startupMetricsService.generatePerformanceReport();
    
    expect(typeof report).toBe('string');
    expect(report).toContain('Startup Performance Report');
    expect(report).toContain('Overall Score');
    expect(report).toContain('Performance Summary');
    expect(report).toContain('Recommendations');
  });

  test('should track performance statistics', () => {
    const stats = startupMetricsService.getStatistics();
    
    expect(stats).toHaveProperty('averageStartupTime');
    expect(stats).toHaveProperty('bestStartupTime');
    expect(stats).toHaveProperty('worstStartupTime');
    expect(stats).toHaveProperty('totalSessions');
    expect(stats).toHaveProperty('averageScore');
    expect(typeof stats.totalSessions).toBe('number');
  });
});

describe('Screen Registry', () => {
  test('should register and retrieve screens', () => {
    const screenNames = screenRegistry.getScreenNames();
    
    expect(screenNames.length).toBeGreaterThan(0);
    expect(screenNames).toContain('LoginScreen');
    expect(screenNames).toContain('RecipeListScreen');
    expect(screenNames).toContain('MealPlanScreen');
  });

  test('should provide bundle analytics', () => {
    const analytics = screenRegistry.getBundleAnalytics();
    
    expect(analytics).toHaveProperty('totalEstimatedSize');
    expect(analytics).toHaveProperty('sizeByCategory');
    expect(analytics).toHaveProperty('sizeByPriority');
    expect(analytics).toHaveProperty('largestScreens');
    expect(analytics.totalEstimatedSize).toBeGreaterThan(0);
    expect(Array.isArray(analytics.largestScreens)).toBe(true);
  });

  test('should prioritize screens correctly', () => {
    const analytics = screenRegistry.getBundleAnalytics();
    const criticalSize = analytics.sizeByPriority[ScreenPriority.CRITICAL] || 0;
    const lowSize = analytics.sizeByPriority[ScreenPriority.LOW] || 0;
    
    // Critical screens should have reasonable size (not too large)
    expect(criticalSize).toBeGreaterThan(0);
    expect(criticalSize).toBeLessThan(200000); // < 200KB for critical screens
  });

  test('should initialize screen registry', async () => {
    const startTime = Date.now();
    await screenRegistry.initialize();
    const endTime = Date.now();
    
    // Initialization should complete reasonably quickly
    expect(endTime - startTime).toBeLessThan(5000);
  });

  test('should provide performance metrics', () => {
    const metrics = screenRegistry.getPerformanceMetrics();
    
    expect(metrics).toHaveProperty('totalScreens');
    expect(metrics).toHaveProperty('loadedScreens');
    expect(metrics).toHaveProperty('averageLoadTime');
    expect(metrics).toHaveProperty('failedLoads');
  });
});

describe('Integration Tests', () => {
  test('should complete full startup optimization flow', async () => {
    // Start performance measurement
    startupMetricsService.startMeasuring();
    
    // Initialize screen registry
    startupMetricsService.startPhase('screen_initialization');
    await screenRegistry.initialize();
    startupMetricsService.endPhase('screen_initialization');
    
    // Preload critical data
    startupMetricsService.startPhase('data_preloading');
    await criticalDataPreloader.preloadCriticalData();
    startupMetricsService.endPhase('data_preloading');
    
    // Analyze bundle
    startupMetricsService.startPhase('bundle_analysis');
    await bundleOptimizationService.analyzeBundleSize();
    startupMetricsService.endPhase('bundle_analysis');
    
    // Complete startup measurement
    startupMetricsService.recordStartupTime(2500);
    startupMetricsService.recordFirstScreenRender();
    
    // Verify end-to-end functionality
    const metrics = startupMetricsService.getCurrentMetrics();
    expect(metrics?.phases).toHaveProperty('screen_initialization');
    expect(metrics?.phases).toHaveProperty('data_preloading');
    expect(metrics?.phases).toHaveProperty('bundle_analysis');
    expect(metrics?.totalStartupTime).toBe(2500);
    expect(metrics?.scores.overall).toBeGreaterThan(0);
  }, 15000); // 15 second timeout for integration test

  test('should meet 3-second startup target', async () => {
    const targetTime = 3000; // 3 seconds
    
    startupMetricsService.startMeasuring();
    
    const startTime = Date.now();
    
    // Simulate optimized startup sequence
    await Promise.all([
      screenRegistry.initialize(),
      criticalDataPreloader.preloadCriticalData()
    ]);
    
    const endTime = Date.now();
    const actualTime = endTime - startTime;
    
    startupMetricsService.recordStartupTime(actualTime);
    
    // Should meet performance target
    expect(actualTime).toBeLessThan(targetTime);
    
    const metrics = startupMetricsService.getCurrentMetrics();
    expect(metrics?.scores.overall).toBeGreaterThan(60); // Acceptable performance
  }, 10000);
});

describe('Performance Validation', () => {
  test('should validate lazy loading performance', async () => {
    const screens = ['TestScreen1', 'TestScreen2', 'TestScreen3'];
    const loadPromises = screens.map((name, index) => 
      lazyLoadingService.preloadScreen(
        name,
        () => Promise.resolve({ default: jest.fn() }),
        { priority: 'normal', cacheStrategy: 'memory' }
      )
    );

    const startTime = Date.now();
    await Promise.all(loadPromises);
    const endTime = Date.now();
    
    const totalTime = endTime - startTime;
    const averageTime = totalTime / screens.length;
    
    // Each screen should load quickly
    expect(averageTime).toBeLessThan(100); // < 100ms per screen
  });

  test('should validate bundle size recommendations', async () => {
    const analysis = await bundleOptimizationService.analyzeBundleSize();
    
    // Should provide meaningful recommendations
    expect(analysis.recommendations.length).toBeGreaterThan(0);
    
    const totalSavings = analysis.recommendations.reduce((sum, rec) => 
      sum + rec.potentialSavings, 0
    );
    
    // Recommendations should offer significant savings
    expect(totalSavings).toBeGreaterThan(100000); // > 100KB potential savings
  });

  test('should validate preload efficiency', async () => {
    const result = await criticalDataPreloader.preloadCriticalData();
    const stats = criticalDataPreloader.getStatistics();
    
    // Should complete successfully with good efficiency
    expect(result.success).toBe(true);
    expect(stats.cacheHitRate).toBeGreaterThan(0.5); // > 50% cache hit rate
    expect(stats.averageLoadTime).toBeLessThan(1000); // < 1s average load time
  });
});