/**
 * Performance Benchmark Service
 * 
 * Provides comprehensive performance benchmarking capabilities for
 * startup performance, bundle size validation, and regression testing
 * 
 * Features:
 * - Startup time benchmarking with statistical analysis
 * - Bundle size measurement and validation
 * - Performance regression detection
 * - Device-specific performance profiling
 * - Automated benchmark reporting
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import { Platform, Dimensions } from 'react-native';
import DeviceInfo from 'expo-device';
import { splashScreenService } from './splash_screen_service';
import { bundleAnalyzerService } from './bundle_analyzer_service';
import { dataPreloadingService } from './data_preloading_service';

export interface BenchmarkConfig {
  iterations: number;
  warmupRuns: number;
  maxTimeout: number;
  includeMemoryProfiling: boolean;
  includeBundleAnalysis: boolean;
  deviceProfiling: boolean;
}

export interface StartupBenchmarkResult {
  iterations: number;
  results: Array<{
    iteration: number;
    totalTime: number;
    splashTime: number;
    dataPreloadTime: number;
    screenPreloadTime: number;
    firstInteractionTime: number;
    memoryUsage: {
      initial: number;
      peak: number;
      final: number;
    };
  }>;
  statistics: {
    mean: number;
    median: number;
    min: number;
    max: number;
    standardDeviation: number;
    p95: number;
    p99: number;
  };
  deviceInfo: {
    platform: string;
    osVersion: string;
    deviceModel: string;
    screenSize: { width: number; height: number };
    isEmulator: boolean;
  };
}

export interface BundleBenchmarkResult {
  totalBundleSize: number;
  bundleComposition: Record<string, number>;
  optimizationOpportunities: number;
  regressionDetected: boolean;
  baselineComparison?: {
    previousSize: number;
    sizeChange: number;
    changePercentage: number;
  };
}

export interface PerformanceBenchmarkReport {
  timestamp: number;
  benchmarkConfig: BenchmarkConfig;
  startup: StartupBenchmarkResult;
  bundle: BundleBenchmarkResult;
  performanceScore: number;
  recommendations: string[];
  passedThresholds: {
    startupTime: boolean;
    bundleSize: boolean;
    memoryUsage: boolean;
    regression: boolean;
  };
}

class PerformanceBenchmarkService {
  private benchmarkResults: PerformanceBenchmarkResult[] = [];
  private isRunning = false;

  private defaultConfig: BenchmarkConfig = {
    iterations: 5,
    warmupRuns: 2,
    maxTimeout: 30000, // 30 seconds per iteration
    includeMemoryProfiling: true,
    includeBundleAnalysis: true,
    deviceProfiling: true
  };

  private performanceThresholds = {
    maxStartupTime: 5000, // 5 seconds
    maxBundleSize: 2 * 1024 * 1024, // 2MB
    maxMemoryUsage: 200, // 200MB
    maxRegressionThreshold: 1.2 // 20% performance degradation
  };

  /**
   * Runs comprehensive performance benchmarks
   */
  async runBenchmark(config?: Partial<BenchmarkConfig>): Promise<PerformanceBenchmarkReport> {
    if (this.isRunning) {
      throw new Error('Benchmark is already running');
    }

    this.isRunning = true;
    const benchmarkConfig = { ...this.defaultConfig, ...config };

    console.log('[PerformanceBenchmark] Starting comprehensive performance benchmark...');

    try {
      // Run startup performance benchmark
      const startupResults = await this.benchmarkStartupPerformance(benchmarkConfig);

      // Run bundle size benchmark
      const bundleResults = await this.benchmarkBundlePerformance(benchmarkConfig);

      // Calculate overall performance score
      const performanceScore = this.calculatePerformanceScore(startupResults, bundleResults);

      // Generate recommendations
      const recommendations = this.generateRecommendations(startupResults, bundleResults);

      // Check thresholds
      const passedThresholds = this.evaluateThresholds(startupResults, bundleResults);

      const report: PerformanceBenchmarkReport = {
        timestamp: Date.now(),
        benchmarkConfig,
        startup: startupResults,
        bundle: bundleResults,
        performanceScore,
        recommendations,
        passedThresholds
      };

      // Store benchmark results
      await this.storeBenchmarkResults(report);
      this.benchmarkResults.push(report);

      console.log(`[PerformanceBenchmark] Benchmark completed - Score: ${performanceScore}/100`);
      return report;

    } finally {
      this.isRunning = false;
    }
  }

  /**
   * Benchmarks startup performance with multiple iterations
   */
  private async benchmarkStartupPerformance(config: BenchmarkConfig): Promise<StartupBenchmarkResult> {
    console.log(`[PerformanceBenchmark] Running startup benchmark (${config.iterations} iterations)...`);

    const results: StartupBenchmarkResult['results'] = [];
    const deviceInfo = await this.collectDeviceInfo();

    // Warmup runs
    for (let i = 0; i < config.warmupRuns; i++) {
      console.log(`[PerformanceBenchmark] Warmup run ${i + 1}/${config.warmupRuns}`);
      await this.runSingleStartupBenchmark();
      await this.delay(1000); // Brief pause between runs
    }

    // Actual benchmark runs
    for (let i = 0; i < config.iterations; i++) {
      console.log(`[PerformanceBenchmark] Benchmark iteration ${i + 1}/${config.iterations}`);
      
      const iterationResult = await this.runSingleStartupBenchmark();
      results.push({
        iteration: i + 1,
        ...iterationResult
      });

      // Brief pause between iterations to ensure clean state
      await this.delay(2000);
    }

    const statistics = this.calculateStatistics(results.map(r => r.totalTime));

    return {
      iterations: config.iterations,
      results,
      statistics,
      deviceInfo
    };
  }

  /**
   * Runs a single startup performance measurement
   */
  private async runSingleStartupBenchmark(): Promise<{
    totalTime: number;
    splashTime: number;
    dataPreloadTime: number;
    screenPreloadTime: number;
    firstInteractionTime: number;
    memoryUsage: { initial: number; peak: number; final: number };
  }> {
    return new Promise((resolve, reject) => {
      const startTime = Date.now();
      let splashTime = 0;
      let dataPreloadTime = 0;
      let screenPreloadTime = 0;
      let firstInteractionTime = 0;
      const memoryUsage = { initial: 0, peak: 0, final: 0 };

      const timeout = setTimeout(() => {
        splashScreenService.cleanup();
        reject(new Error('Startup benchmark timeout'));
      }, this.defaultConfig.maxTimeout);

      // Track memory usage
      this.trackMemoryUsage(memoryUsage);

      // Set up splash screen event handlers
      splashScreenService.onProgressUpdate((step, progress) => {
        if (step === 'loading_user_preferences') {
          splashTime = Date.now() - startTime;
        } else if (step === 'syncing_offline_data') {
          dataPreloadTime = Date.now() - startTime - splashTime;
        } else if (step === 'preloading_critical_screens') {
          screenPreloadTime = Date.now() - startTime - splashTime - dataPreloadTime;
        }
      });

      splashScreenService.onComplete(() => {
        clearTimeout(timeout);
        
        const totalTime = Date.now() - startTime;
        firstInteractionTime = totalTime; // First interaction possible when complete
        memoryUsage.final = this.getCurrentMemoryEstimate();

        splashScreenService.cleanup();
        
        resolve({
          totalTime,
          splashTime,
          dataPreloadTime,
          screenPreloadTime,
          firstInteractionTime,
          memoryUsage
        });
      });

      splashScreenService.onError((error) => {
        clearTimeout(timeout);
        splashScreenService.cleanup();
        reject(error);
      });

      // Start the initialization
      splashScreenService.initializeApp({
        enableProgressTracking: true,
        enableCacheWarming: true,
        enableScreenPreloading: true,
        enableOfflineSync: true
      });
    });
  }

  /**
   * Tracks memory usage during benchmark
   */
  private trackMemoryUsage(memoryUsage: { initial: number; peak: number; final: number }): void {
    memoryUsage.initial = this.getCurrentMemoryEstimate();
    memoryUsage.peak = memoryUsage.initial;

    const interval = setInterval(() => {
      const current = this.getCurrentMemoryEstimate();
      if (current > memoryUsage.peak) {
        memoryUsage.peak = current;
      }
    }, 100);

    setTimeout(() => {
      clearInterval(interval);
    }, this.defaultConfig.maxTimeout);
  }

  /**
   * Estimates current memory usage
   */
  private getCurrentMemoryEstimate(): number {
    try {
      if (typeof performance !== 'undefined' && performance.memory) {
        return Math.round(performance.memory.usedJSHeapSize / (1024 * 1024));
      }
      
      // Fallback estimation
      return 70 + Math.random() * 20; // 70-90 MB range
    } catch (error) {
      return 80; // Default estimate
    }
  }

  /**
   * Benchmarks bundle performance and analyzes size
   */
  private async benchmarkBundlePerformance(config: BenchmarkConfig): Promise<BundleBenchmarkResult> {
    console.log('[PerformanceBenchmark] Running bundle analysis benchmark...');

    if (!config.includeBundleAnalysis) {
      return {
        totalBundleSize: 0,
        bundleComposition: {},
        optimizationOpportunities: 0,
        regressionDetected: false
      };
    }

    try {
      const analysis = await bundleAnalyzerService.analyzeBundleSize();
      const regression = await bundleAnalyzerService.monitorBundleRegression();

      let baselineComparison;
      if (regression.previousSize) {
        baselineComparison = {
          previousSize: regression.previousSize,
          sizeChange: regression.sizeIncrease || 0,
          changePercentage: regression.previousSize > 0 
            ? ((regression.sizeIncrease || 0) / regression.previousSize) * 100 
            : 0
        };
      }

      return {
        totalBundleSize: analysis.totalEstimatedSize,
        bundleComposition: analysis.sizeByCategory,
        optimizationOpportunities: analysis.optimizationOpportunities.length,
        regressionDetected: regression.hasRegression,
        baselineComparison
      };

    } catch (error) {
      console.error('[PerformanceBenchmark] Bundle analysis failed:', error);
      throw error;
    }
  }

  /**
   * Collects device information for benchmarking context
   */
  private async collectDeviceInfo() {
    const screenSize = Dimensions.get('screen');
    
    return {
      platform: Platform.OS,
      osVersion: Platform.Version.toString(),
      deviceModel: DeviceInfo.deviceName || 'Unknown',
      screenSize: { width: screenSize.width, height: screenSize.height },
      isEmulator: !DeviceInfo.isDevice
    };
  }

  /**
   * Calculates statistical metrics from timing data
   */
  private calculateStatistics(timings: number[]) {
    const sorted = [...timings].sort((a, b) => a - b);
    const sum = timings.reduce((acc, val) => acc + val, 0);
    const mean = sum / timings.length;
    
    const variance = timings.reduce((acc, val) => acc + Math.pow(val - mean, 2), 0) / timings.length;
    const standardDeviation = Math.sqrt(variance);

    return {
      mean: Math.round(mean),
      median: Math.round(sorted[Math.floor(sorted.length / 2)]),
      min: Math.min(...timings),
      max: Math.max(...timings),
      standardDeviation: Math.round(standardDeviation),
      p95: Math.round(sorted[Math.floor(sorted.length * 0.95)]),
      p99: Math.round(sorted[Math.floor(sorted.length * 0.99)])
    };
  }

  /**
   * Calculates overall performance score (0-100)
   */
  private calculatePerformanceScore(startup: StartupBenchmarkResult, bundle: BundleBenchmarkResult): number {
    let score = 100;

    // Startup time scoring (40% weight)
    const startupScore = Math.max(0, 100 - (startup.statistics.mean / 50)); // 50ms = 1 point
    score = score * 0.6 + startupScore * 0.4;

    // Bundle size scoring (30% weight)
    const bundleScore = Math.max(0, 100 - (bundle.totalBundleSize / 20480)); // 20KB = 1 point
    score = score * 0.7 + bundleScore * 0.3;

    // Memory usage scoring (20% weight)
    const avgMemory = startup.results.reduce((sum, r) => sum + r.memoryUsage.peak, 0) / startup.results.length;
    const memoryScore = Math.max(0, 100 - (avgMemory / 2)); // 2MB = 1 point
    score = score * 0.8 + memoryScore * 0.2;

    // Regression penalty (10% weight)
    if (bundle.regressionDetected) {
      score *= 0.9; // 10% penalty for regression
    }

    return Math.round(Math.max(0, Math.min(100, score)));
  }

  /**
   * Generates performance recommendations
   */
  private generateRecommendations(startup: StartupBenchmarkResult, bundle: BundleBenchmarkResult): string[] {
    const recommendations: string[] = [];

    // Startup performance recommendations
    if (startup.statistics.mean > 4000) {
      recommendations.push('Optimize startup time: Consider lazy loading non-critical components');
    }
    
    if (startup.statistics.standardDeviation > 1000) {
      recommendations.push('Reduce startup time variance: Investigate inconsistent initialization steps');
    }

    const avgMemory = startup.results.reduce((sum, r) => sum + r.memoryUsage.peak, 0) / startup.results.length;
    if (avgMemory > 150) {
      recommendations.push('Optimize memory usage: Consider reducing initial memory footprint');
    }

    // Bundle size recommendations
    if (bundle.totalBundleSize > 1.5 * 1024 * 1024) { // 1.5MB
      recommendations.push('Reduce bundle size: Implement code splitting for large components');
    }

    if (bundle.optimizationOpportunities > 5) {
      recommendations.push('Address bundle optimization opportunities: Run bundle analyzer for specific recommendations');
    }

    if (bundle.regressionDetected) {
      recommendations.push('Bundle size regression detected: Investigate recent changes causing size increase');
    }

    return recommendations;
  }

  /**
   * Evaluates performance against thresholds
   */
  private evaluateThresholds(startup: StartupBenchmarkResult, bundle: BundleBenchmarkResult) {
    const avgMemory = startup.results.reduce((sum, r) => sum + r.memoryUsage.peak, 0) / startup.results.length;

    return {
      startupTime: startup.statistics.p95 <= this.performanceThresholds.maxStartupTime,
      bundleSize: bundle.totalBundleSize <= this.performanceThresholds.maxBundleSize,
      memoryUsage: avgMemory <= this.performanceThresholds.maxMemoryUsage,
      regression: !bundle.regressionDetected
    };
  }

  /**
   * Stores benchmark results for historical analysis
   */
  private async storeBenchmarkResults(report: PerformanceBenchmarkReport): Promise<void> {
    try {
      const key = 'performance_benchmark_history';
      const existing = await AsyncStorage.getItem(key);
      const history: PerformanceBenchmarkReport[] = existing ? JSON.parse(existing) : [];
      
      history.push(report);
      
      // Keep only last 50 benchmarks
      const recentHistory = history.slice(-50);
      
      await AsyncStorage.setItem(key, JSON.stringify(recentHistory));
    } catch (error) {
      console.error('[PerformanceBenchmark] Failed to store benchmark results:', error);
    }
  }

  /**
   * Gets historical benchmark results
   */
  async getBenchmarkHistory(): Promise<PerformanceBenchmarkResult[]> {
    try {
      const stored = await AsyncStorage.getItem('performance_benchmark_history');
      return stored ? JSON.parse(stored) : [];
    } catch (error) {
      console.error('[PerformanceBenchmark] Failed to load benchmark history:', error);
      return [];
    }
  }

  /**
   * Generates detailed benchmark report
   */
  generateReport(report: PerformanceBenchmarkReport): string {
    const { startup, bundle, performanceScore, recommendations } = report;
    
    return `
# Performance Benchmark Report
Generated: ${new Date(report.timestamp).toLocaleString()}

## Summary
**Performance Score**: ${performanceScore}/100
**Device**: ${startup.deviceInfo.deviceModel} (${startup.deviceInfo.platform} ${startup.deviceInfo.osVersion})

## Startup Performance
- **Mean Time**: ${startup.statistics.mean}ms
- **P95 Time**: ${startup.statistics.p95}ms
- **Standard Deviation**: ${startup.statistics.standardDeviation}ms
- **Iterations**: ${startup.iterations}

### Breakdown
- Splash Screen: ${startup.results[0]?.splashTime || 0}ms
- Data Preloading: ${startup.results[0]?.dataPreloadTime || 0}ms
- Screen Preloading: ${startup.results[0]?.screenPreloadTime || 0}ms

## Bundle Analysis
- **Total Size**: ${Math.round(bundle.totalBundleSize / 1024)}KB
- **Optimization Opportunities**: ${bundle.optimizationOpportunities}
- **Regression Detected**: ${bundle.regressionDetected ? 'Yes' : 'No'}

${bundle.baselineComparison ? `
### Size Comparison
- **Previous Size**: ${Math.round(bundle.baselineComparison.previousSize / 1024)}KB
- **Size Change**: ${Math.round(bundle.baselineComparison.sizeChange / 1024)}KB (${bundle.baselineComparison.changePercentage.toFixed(1)}%)
` : ''}

## Threshold Results
- ✅ Startup Time: ${report.passedThresholds.startupTime ? 'PASS' : 'FAIL'}
- ✅ Bundle Size: ${report.passedThresholds.bundleSize ? 'PASS' : 'FAIL'}
- ✅ Memory Usage: ${report.passedThresholds.memoryUsage ? 'PASS' : 'FAIL'}
- ✅ No Regression: ${report.passedThresholds.regression ? 'PASS' : 'FAIL'}

## Recommendations
${recommendations.map(rec => `- ${rec}`).join('\n')}
    `.trim();
  }

  /**
   * Clears benchmark history
   */
  async clearBenchmarkHistory(): Promise<void> {
    try {
      await AsyncStorage.removeItem('performance_benchmark_history');
      this.benchmarkResults = [];
    } catch (error) {
      console.error('[PerformanceBenchmark] Failed to clear benchmark history:', error);
    }
  }

  /**
   * Gets current benchmark status
   */
  getStatus(): { isRunning: boolean; lastBenchmark?: number; totalBenchmarks: number } {
    return {
      isRunning: this.isRunning,
      lastBenchmark: this.benchmarkResults.length > 0 
        ? this.benchmarkResults[this.benchmarkResults.length - 1].timestamp 
        : undefined,
      totalBenchmarks: this.benchmarkResults.length
    };
  }

  /**
   * Utility delay function
   */
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// Export singleton instance
export const performanceBenchmarkService = new PerformanceBenchmarkService();
export default PerformanceBenchmarkService;