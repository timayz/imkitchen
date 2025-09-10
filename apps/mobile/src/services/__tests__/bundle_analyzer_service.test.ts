/**
 * Bundle Analyzer Service Tests
 * 
 * Tests for bundle size analysis, optimization opportunities detection,
 * and bundle size monitoring
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import { bundleAnalyzerService } from '../bundle_analyzer_service';
import type { BundleAnalysis, OptimizationOpportunity } from '../bundle_analyzer_service';

// Mock AsyncStorage
jest.mock('@react-native-async-storage/async-storage');
const mockedAsyncStorage = AsyncStorage as jest.Mocked<typeof AsyncStorage>;

// Mock screen registry
jest.mock('../../navigation/ScreenRegistry', () => ({
  screenRegistry: {
    getBundleAnalytics: jest.fn().mockReturnValue({
      totalEstimatedSize: 450000,
      sizeByCategory: {
        auth: 125000,
        recipes: 180000,
        profile: 95000,
        preferences: 50000
      },
      sizeByPriority: {
        critical: 70000,
        high: 260000,
        normal: 85000,
        low: 35000
      },
      largestScreens: [
        { name: 'MealPlanScreen', size: 120000 },
        { name: 'RecipeDetailScreen', size: 95000 },
        { name: 'AddRecipeScreen', size: 87000 },
        { name: 'CommunityRecipesScreen', size: 82000 },
        { name: 'AccountSettingsScreen', size: 55000 }
      ]
    })
  }
}));

describe('BundleAnalyzerService', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    bundleAnalyzerService.clearCache();
    mockedAsyncStorage.getItem.mockResolvedValue(null);
    mockedAsyncStorage.setItem.mockResolvedValue();
  });

  describe('analyzeBundleSize', () => {
    it('should analyze bundle size and identify optimization opportunities', async () => {
      const analysis = await bundleAnalyzerService.analyzeBundleSize();

      expect(analysis.totalEstimatedSize).toBe(450000);
      expect(analysis.sizeByCategory).toBeDefined();
      expect(analysis.sizeByPriority).toBeDefined();
      expect(analysis.largestComponents).toBeDefined();
      expect(analysis.optimizationOpportunities).toBeDefined();
      expect(analysis.optimizationOpportunities.length).toBeGreaterThan(0);
    });

    it('should calculate compression ratio when baseline is available', async () => {
      // Mock metrics history with baseline
      mockedAsyncStorage.getItem.mockResolvedValueOnce(JSON.stringify([
        {
          timestamp: Date.now() - 86400000, // 24 hours ago
          totalSize: 600000, // Larger baseline
          screenCount: 15,
          averageScreenSize: 40000,
          compressionRatio: 0,
          unusedDependencies: [],
          optimizationScore: 75
        }
      ]));

      const analysis = await bundleAnalyzerService.analyzeBundleSize();

      expect(analysis.baselineSize).toBe(600000);
      expect(analysis.compressionRatio).toBeGreaterThan(0);
    });

    it('should identify large component optimization opportunities', async () => {
      const analysis = await bundleAnalyzerService.analyzeBundleSize();

      const largeComponentOpportunities = analysis.optimizationOpportunities.filter(
        opp => opp.type === 'large_component'
      );

      expect(largeComponentOpportunities.length).toBeGreaterThan(0);
      
      const mealPlanOpportunity = largeComponentOpportunities.find(
        opp => opp.component === 'MealPlanScreen'
      );
      
      expect(mealPlanOpportunity).toBeDefined();
      expect(mealPlanOpportunity?.severity).toBe('high');
      expect(mealPlanOpportunity?.potentialSavings).toBeGreaterThan(0);
    });

    it('should identify unused dependency opportunities', async () => {
      const analysis = await bundleAnalyzerService.analyzeBundleSize();

      const unusedDepOpportunities = analysis.optimizationOpportunities.filter(
        opp => opp.type === 'unused_dependency'
      );

      expect(unusedDepOpportunities.length).toBeGreaterThan(0);
      
      const unusedDep = unusedDepOpportunities.find(
        opp => opp.component === 'unused-utility-lib'
      );
      
      expect(unusedDep).toBeDefined();
      expect(unusedDep?.potentialSavings).toBe(unusedDep?.currentSize);
    });

    it('should identify image optimization opportunities', async () => {
      const analysis = await bundleAnalyzerService.analyzeBundleSize();

      const imageOpportunities = analysis.optimizationOpportunities.filter(
        opp => opp.type === 'image_optimization'
      );

      expect(imageOpportunities.length).toBeGreaterThan(0);
      
      const imageOpp = imageOpportunities[0];
      expect(imageOpp.potentialSavings).toBeGreaterThan(0);
      expect(imageOpp.recommendation).toContain('WebP');
    });
  });

  describe('applyOptimizations', () => {
    it('should apply optimization opportunities and track results', async () => {
      const opportunities: OptimizationOpportunity[] = [
        {
          type: 'large_component',
          severity: 'high',
          component: 'MealPlanScreen',
          currentSize: 120000,
          potentialSavings: 36000,
          description: 'MealPlanScreen is 120KB, consider code splitting',
          recommendation: 'Implement lazy loading, reduce dependencies'
        },
        {
          type: 'unused_dependency',
          severity: 'medium',
          component: 'unused-lib',
          currentSize: 45000,
          potentialSavings: 45000,
          description: 'unused-lib appears to be unused',
          recommendation: 'Remove unused dependency'
        }
      ];

      const results = await bundleAnalyzerService.applyOptimizations(opportunities);

      expect(results.applied).toBe(2);
      expect(results.estimatedSavings).toBe(81000);
      expect(results.errors).toHaveLength(0);
    });

    it('should handle optimization failures gracefully', async () => {
      const opportunities: OptimizationOpportunity[] = [
        {
          type: 'large_component',
          severity: 'high',
          component: 'FailingScreen',
          currentSize: 100000,
          potentialSavings: 30000,
          description: 'This will fail',
          recommendation: 'This will fail'
        }
      ];

      // Mock a failure in the apply process
      const originalConsoleLog = console.log;
      console.log = jest.fn().mockImplementation((msg) => {
        if (msg.includes('FailingScreen')) {
          throw new Error('Optimization failed');
        }
      });

      const results = await bundleAnalyzerService.applyOptimizations(opportunities);

      expect(results.applied).toBe(0);
      expect(results.errors.length).toBeGreaterThan(0);

      console.log = originalConsoleLog;
    });
  });

  describe('monitorBundleRegression', () => {
    it('should detect bundle size regression', async () => {
      // Mock metrics history with smaller previous size
      mockedAsyncStorage.getItem.mockResolvedValueOnce(JSON.stringify([
        {
          timestamp: Date.now() - 86400000,
          totalSize: 300000, // Much smaller previous size
          screenCount: 12,
          averageScreenSize: 25000,
          compressionRatio: 0.1,
          unusedDependencies: [],
          optimizationScore: 85
        },
        {
          timestamp: Date.now() - 43200000,
          totalSize: 320000, // Slightly larger
          screenCount: 13,
          averageScreenSize: 24615,
          compressionRatio: 0.08,
          unusedDependencies: [],
          optimizationScore: 82
        }
      ]));

      const regression = await bundleAnalyzerService.monitorBundleRegression();

      expect(regression.hasRegression).toBe(true);
      expect(regression.sizeIncrease).toBeGreaterThan(0);
      expect(regression.previousSize).toBe(320000);
      expect(regression.currentSize).toBe(450000);
    });

    it('should not detect regression when size increase is within threshold', async () => {
      // Mock metrics history with similar previous size
      mockedAsyncStorage.getItem.mockResolvedValueOnce(JSON.stringify([
        {
          timestamp: Date.now() - 86400000,
          totalSize: 440000, // Similar previous size (within 5% threshold)
          screenCount: 15,
          averageScreenSize: 29333,
          compressionRatio: 0.02,
          unusedDependencies: [],
          optimizationScore: 78
        }
      ]));

      const regression = await bundleAnalyzerService.monitorBundleRegression();

      expect(regression.hasRegression).toBe(false);
      expect(regression.currentSize).toBe(450000);
    });

    it('should handle no previous metrics gracefully', async () => {
      // Empty metrics history
      mockedAsyncStorage.getItem.mockResolvedValueOnce(null);

      const regression = await bundleAnalyzerService.monitorBundleRegression();

      expect(regression.hasRegression).toBe(false);
      expect(regression.previousSize).toBeUndefined();
      expect(regression.sizeIncrease).toBeUndefined();
      expect(regression.currentSize).toBe(450000);
    });
  });

  describe('generateOptimizationReport', () => {
    it('should generate comprehensive optimization report', async () => {
      const analysis = await bundleAnalyzerService.analyzeBundleSize();
      const report = bundleAnalyzerService.generateOptimizationReport(analysis);

      expect(report).toContain('Bundle Size Optimization Report');
      expect(report).toContain('Total Bundle Size');
      expect(report).toContain('Potential Savings');
      expect(report).toContain('Largest Components');
      expect(report).toContain('Optimization Opportunities');
      expect(report).toContain('Size by Category');
      
      // Should include specific data
      expect(report).toContain('440KB'); // Total size in KB
      expect(report).toContain('MealPlanScreen');
      expect(report).toContain('auth:');
      expect(report).toContain('recipes:');
    });

    it('should include optimization score in report', async () => {
      const analysis = await bundleAnalyzerService.analyzeBundleSize();
      const report = bundleAnalyzerService.generateOptimizationReport(analysis);

      expect(report).toMatch(/Optimization Score.*\/100/);
    });
  });

  describe('metrics tracking', () => {
    it('should load and save metrics history', async () => {
      const mockHistory = [
        {
          timestamp: Date.now() - 86400000,
          totalSize: 400000,
          screenCount: 14,
          averageScreenSize: 28571,
          compressionRatio: 0.05,
          unusedDependencies: ['old-lib'],
          optimizationScore: 80
        }
      ];

      mockedAsyncStorage.getItem.mockResolvedValueOnce(JSON.stringify(mockHistory));

      // Trigger analysis to load and save metrics
      await bundleAnalyzerService.analyzeBundleSize();

      expect(mockedAsyncStorage.getItem).toHaveBeenCalledWith('bundle_metrics_history');
      expect(mockedAsyncStorage.setItem).toHaveBeenCalledWith(
        'bundle_metrics_history',
        expect.stringContaining('"totalSize":450000')
      );
    });

    it('should limit metrics history to 30 entries', async () => {
      // Create 31 mock entries
      const mockHistory = Array.from({ length: 31 }, (_, i) => ({
        timestamp: Date.now() - (i * 86400000),
        totalSize: 400000 + (i * 1000),
        screenCount: 14,
        averageScreenSize: 28571,
        compressionRatio: 0.05,
        unusedDependencies: [],
        optimizationScore: 80
      }));

      mockedAsyncStorage.getItem.mockResolvedValueOnce(JSON.stringify(mockHistory));

      await bundleAnalyzerService.analyzeBundleSize();

      // Should save only the most recent 30 entries
      const savedHistory = JSON.parse(
        (mockedAsyncStorage.setItem as jest.Mock).mock.calls[0][1]
      );
      expect(savedHistory.length).toBe(30);
    });
  });

  describe('cache management', () => {
    it('should cache analysis results', async () => {
      const analysis1 = await bundleAnalyzerService.getBundleAnalysis();
      const analysis2 = await bundleAnalyzerService.getBundleAnalysis();

      expect(analysis1).toBe(analysis2); // Should return same cached instance
    });

    it('should clear cache when requested', async () => {
      await bundleAnalyzerService.getBundleAnalysis();
      bundleAnalyzerService.clearCache();
      
      const newAnalysis = await bundleAnalyzerService.getBundleAnalysis();
      expect(newAnalysis).toBeDefined(); // Should generate new analysis
    });
  });
});