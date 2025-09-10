/**
 * Startup Integration Service
 * 
 * Orchestrates the complete app startup process by integrating
 * splash screen, data preloading, and initialization services.
 * 
 * Features:
 * - Coordinates splash screen and data preloading
 * - Manages startup timing and priorities
 * - Handles startup failures and recovery
 * - Provides unified startup progress reporting
 * - Optimizes perceived performance
 */

import { splashScreenService, InitializationStep } from './splash_screen_service';
import { dataPreloadingService, PreloadingProgress } from './data_preloading_service';
import { screenRegistry } from '../navigation/ScreenRegistry';

export interface StartupProgress {
  phase: 'splash' | 'preloading' | 'finalizing' | 'complete' | 'error';
  overallProgress: number; // 0-100
  currentTask: string;
  splashProgress: number;
  preloadingProgress: number;
  timeElapsed: number; // seconds
  estimatedTimeRemaining: number; // seconds
}

export interface StartupMetrics {
  startTime: number;
  splashScreenTime: number;
  dataPreloadingTime: number;
  screenPreloadingTime: number;
  totalStartupTime: number;
  preloadedDataSize: number;
  cacheHitRate: number;
  errors: string[];
}

export interface StartupConfig {
  enableDataPreloading: boolean;
  enableScreenPreloading: boolean;
  parallelPreloading: boolean; // Run preloading parallel to splash screen
  minimumSplashTime: number; // Ensure splash shows for minimum time
  maximumStartupTime: number; // Timeout for entire startup
  fallbackMode: boolean; // Skip non-critical operations on failure
}

class StartupIntegrationService {
  private startupMetrics: StartupMetrics | null = null;
  private progressCallback?: (progress: StartupProgress) => void;
  private isStarting = false;
  private startTime = 0;

  private config: StartupConfig = {
    enableDataPreloading: true,
    enableScreenPreloading: true,
    parallelPreloading: true,
    minimumSplashTime: 2000, // 2 seconds minimum splash
    maximumStartupTime: 15000, // 15 seconds maximum
    fallbackMode: false
  };

  /**
   * Starts the complete app initialization process
   */
  async startApp(
    config?: Partial<StartupConfig>,
    progressCallback?: (progress: StartupProgress) => void
  ): Promise<StartupMetrics> {
    if (this.isStarting) {
      console.warn('[StartupIntegration] App startup already in progress');
      return this.startupMetrics || this.createEmptyMetrics();
    }

    this.isStarting = true;
    this.config = { ...this.config, ...config };
    this.progressCallback = progressCallback;
    this.startTime = Date.now();

    console.log('[StartupIntegration] Starting integrated app initialization...');

    this.startupMetrics = {
      startTime: this.startTime,
      splashScreenTime: 0,
      dataPreloadingTime: 0,
      screenPreloadingTime: 0,
      totalStartupTime: 0,
      preloadedDataSize: 0,
      cacheHitRate: 0,
      errors: []
    };

    try {
      if (this.config.parallelPreloading) {
        await this.runParallelStartup();
      } else {
        await this.runSequentialStartup();
      }

      // Ensure minimum splash time
      await this.enforceMinimumSplashTime();

      // Finalize startup
      await this.finalizeStartup();

      this.startupMetrics.totalStartupTime = Date.now() - this.startTime;
      
      this.updateProgress({
        phase: 'complete',
        overallProgress: 100,
        currentTask: 'Startup complete',
        splashProgress: 100,
        preloadingProgress: 100,
        timeElapsed: this.startupMetrics.totalStartupTime / 1000,
        estimatedTimeRemaining: 0
      });

      console.log(`[StartupIntegration] App startup completed in ${this.startupMetrics.totalStartupTime}ms`);
      return this.startupMetrics;

    } catch (error) {
      console.error('[StartupIntegration] App startup failed:', error);
      
      const errorMsg = error instanceof Error ? error.message : 'Unknown startup error';
      this.startupMetrics.errors.push(errorMsg);

      this.updateProgress({
        phase: 'error',
        overallProgress: 0,
        currentTask: `Startup failed: ${errorMsg}`,
        splashProgress: 0,
        preloadingProgress: 0,
        timeElapsed: (Date.now() - this.startTime) / 1000,
        estimatedTimeRemaining: 0
      });

      throw error;
    } finally {
      this.isStarting = false;
    }
  }

  /**
   * Runs startup with parallel preloading (recommended)
   */
  private async runParallelStartup(): Promise<void> {
    console.log('[StartupIntegration] Running parallel startup process...');

    const splashPromise = this.runSplashScreen();
    const preloadingPromise = this.runDataPreloading();
    
    // Start both processes simultaneously
    const [splashMetrics, preloadingMetrics] = await Promise.allSettled([
      splashPromise,
      preloadingPromise
    ]);

    // Handle results
    if (splashMetrics.status === 'fulfilled') {
      this.startupMetrics!.splashScreenTime = splashMetrics.value;
    } else {
      const error = `Splash screen failed: ${splashMetrics.reason}`;
      this.startupMetrics!.errors.push(error);
      if (!this.config.fallbackMode) {
        throw new Error(error);
      }
    }

    if (preloadingMetrics.status === 'fulfilled') {
      this.startupMetrics!.dataPreloadingTime = preloadingMetrics.value.duration;
      this.startupMetrics!.preloadedDataSize = preloadingMetrics.value.dataSize;
      this.startupMetrics!.cacheHitRate = preloadingMetrics.value.cacheHitRate;
    } else {
      const error = `Data preloading failed: ${preloadingMetrics.reason}`;
      this.startupMetrics!.errors.push(error);
      if (!this.config.fallbackMode) {
        console.warn('[StartupIntegration] Data preloading failed, continuing without preloaded data');
      }
    }

    // Screen preloading (if enabled)
    if (this.config.enableScreenPreloading) {
      const screenStart = Date.now();
      try {
        await screenRegistry.initialize();
        this.startupMetrics!.screenPreloadingTime = Date.now() - screenStart;
      } catch (error) {
        const errorMsg = `Screen preloading failed: ${error}`;
        this.startupMetrics!.errors.push(errorMsg);
        if (!this.config.fallbackMode) {
          console.warn('[StartupIntegration] Screen preloading failed, continuing without preloaded screens');
        }
      }
    }
  }

  /**
   * Runs startup with sequential operations
   */
  private async runSequentialStartup(): Promise<void> {
    console.log('[StartupIntegration] Running sequential startup process...');

    // Run splash screen first
    const splashStart = Date.now();
    try {
      await this.runSplashScreen();
      this.startupMetrics!.splashScreenTime = Date.now() - splashStart;
    } catch (error) {
      const errorMsg = `Splash screen failed: ${error}`;
      this.startupMetrics!.errors.push(errorMsg);
      throw new Error(errorMsg);
    }

    // Then run data preloading
    if (this.config.enableDataPreloading) {
      try {
        const preloadingResult = await this.runDataPreloading();
        this.startupMetrics!.dataPreloadingTime = preloadingResult.duration;
        this.startupMetrics!.preloadedDataSize = preloadingResult.dataSize;
        this.startupMetrics!.cacheHitRate = preloadingResult.cacheHitRate;
      } catch (error) {
        const errorMsg = `Data preloading failed: ${error}`;
        this.startupMetrics!.errors.push(errorMsg);
        if (!this.config.fallbackMode) {
          throw new Error(errorMsg);
        }
      }
    }

    // Finally, screen preloading
    if (this.config.enableScreenPreloading) {
      const screenStart = Date.now();
      try {
        await screenRegistry.initialize();
        this.startupMetrics!.screenPreloadingTime = Date.now() - screenStart;
      } catch (error) {
        const errorMsg = `Screen preloading failed: ${error}`;
        this.startupMetrics!.errors.push(errorMsg);
        if (!this.config.fallbackMode) {
          throw new Error(errorMsg);
        }
      }
    }
  }

  /**
   * Runs splash screen initialization
   */
  private async runSplashScreen(): Promise<number> {
    return new Promise((resolve, reject) => {
      const startTime = Date.now();

      splashScreenService.onProgressUpdate((step: InitializationStep, progress: number) => {
        this.updateProgress({
          phase: 'splash',
          overallProgress: progress * 0.6, // Splash takes 60% of overall progress
          currentTask: `Initializing: ${step.replace(/_/g, ' ')}`,
          splashProgress: progress,
          preloadingProgress: 0,
          timeElapsed: (Date.now() - this.startTime) / 1000,
          estimatedTimeRemaining: this.calculateEstimatedTime(progress, 0.6)
        });
      });

      splashScreenService.onComplete(() => {
        const duration = Date.now() - startTime;
        console.log(`[StartupIntegration] Splash screen completed in ${duration}ms`);
        resolve(duration);
      });

      splashScreenService.onError((error: Error) => {
        console.error('[StartupIntegration] Splash screen error:', error);
        reject(error);
      });

      // Start splash screen initialization
      splashScreenService.initializeApp({
        enableProgressTracking: true,
        enableCacheWarming: this.config.enableDataPreloading,
        enableScreenPreloading: this.config.enableScreenPreloading,
        maxInitializationTime: this.config.maximumStartupTime * 0.6 // 60% of total time
      });
    });
  }

  /**
   * Runs data preloading
   */
  private async runDataPreloading(): Promise<{
    duration: number;
    dataSize: number;
    cacheHitRate: number;
  }> {
    if (!this.config.enableDataPreloading) {
      return { duration: 0, dataSize: 0, cacheHitRate: 0 };
    }

    return new Promise((resolve, reject) => {
      const startTime = Date.now();

      dataPreloadingService.preloadCriticalData((progress: PreloadingProgress) => {
        this.updateProgress({
          phase: 'preloading',
          overallProgress: 60 + (progress.progress * 0.3), // Preloading takes 30% of overall progress
          currentTask: progress.currentItem ? `Loading ${progress.currentItem.replace(/_/g, ' ')}` : 'Loading data',
          splashProgress: 100, // Splash should be done by now
          preloadingProgress: progress.progress,
          timeElapsed: (Date.now() - this.startTime) / 1000,
          estimatedTimeRemaining: Math.max(progress.estimatedTimeRemaining, this.calculateEstimatedTime(60 + progress.progress * 0.3, 0.3))
        });
      }).then((results) => {
        const duration = Date.now() - startTime;
        const summary = dataPreloadingService.getPreloadingResults();
        
        console.log(`[StartupIntegration] Data preloading completed in ${duration}ms`);
        console.log(`[StartupIntegration] Loaded ${summary.totalSize} bytes, cache hit rate: ${Math.round(summary.cacheHitRate * 100)}%`);

        resolve({
          duration,
          dataSize: summary.totalSize,
          cacheHitRate: summary.cacheHitRate
        });
      }).catch((error) => {
        console.error('[StartupIntegration] Data preloading failed:', error);
        reject(error);
      });
    });
  }

  /**
   * Ensures minimum splash screen display time for UX
   */
  private async enforceMinimumSplashTime(): Promise<void> {
    const elapsed = Date.now() - this.startTime;
    if (elapsed < this.config.minimumSplashTime) {
      const remainingTime = this.config.minimumSplashTime - elapsed;
      console.log(`[StartupIntegration] Ensuring minimum splash time: ${remainingTime}ms remaining`);
      await this.delay(remainingTime);
    }
  }

  /**
   * Finalizes the startup process
   */
  private async finalizeStartup(): Promise<void> {
    this.updateProgress({
      phase: 'finalizing',
      overallProgress: 95,
      currentTask: 'Finalizing startup',
      splashProgress: 100,
      preloadingProgress: 100,
      timeElapsed: (Date.now() - this.startTime) / 1000,
      estimatedTimeRemaining: 0.5
    });

    // Small delay for finalization
    await this.delay(200);

    // Verify critical data availability
    const offlineAvailability = await dataPreloadingService.verifyOfflineAvailability();
    console.log(`[StartupIntegration] Offline availability: ${offlineAvailability.available.length} items, ${Math.round(offlineAvailability.totalOfflineSize / 1024)}KB`);

    if (offlineAvailability.missing.length > 0) {
      console.warn('[StartupIntegration] Missing offline data:', offlineAvailability.missing);
    }
  }

  /**
   * Calculates estimated time remaining based on progress
   */
  private calculateEstimatedTime(currentProgress: number, totalWeight: number): number {
    if (currentProgress <= 0) return 0;
    
    const elapsed = (Date.now() - this.startTime) / 1000;
    const progressRatio = (currentProgress / 100) / totalWeight;
    
    if (progressRatio <= 0) return 0;
    
    const estimatedTotal = elapsed / progressRatio;
    return Math.max(0, estimatedTotal - elapsed);
  }

  /**
   * Updates progress and calls callback
   */
  private updateProgress(progress: StartupProgress): void {
    this.progressCallback?.(progress);
  }

  /**
   * Utility delay function
   */
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Creates empty metrics object
   */
  private createEmptyMetrics(): StartupMetrics {
    return {
      startTime: 0,
      splashScreenTime: 0,
      dataPreloadingTime: 0,
      screenPreloadingTime: 0,
      totalStartupTime: 0,
      preloadedDataSize: 0,
      cacheHitRate: 0,
      errors: []
    };
  }

  /**
   * Gets current startup metrics
   */
  getMetrics(): StartupMetrics | null {
    return this.startupMetrics;
  }

  /**
   * Gets startup status
   */
  getStatus(): {
    isStarting: boolean;
    timeElapsed: number;
    phase: string;
  } {
    return {
      isStarting: this.isStarting,
      timeElapsed: this.startTime > 0 ? (Date.now() - this.startTime) / 1000 : 0,
      phase: this.isStarting ? 'running' : 'idle'
    };
  }

  /**
   * Generates comprehensive startup report
   */
  generateStartupReport(): string {
    if (!this.startupMetrics) {
      return 'No startup metrics available';
    }

    const { 
      totalStartupTime, 
      splashScreenTime, 
      dataPreloadingTime, 
      screenPreloadingTime,
      preloadedDataSize,
      cacheHitRate,
      errors 
    } = this.startupMetrics;

    let report = `
# App Startup Performance Report

## Summary
- **Total Startup Time**: ${totalStartupTime}ms
- **Splash Screen Time**: ${splashScreenTime}ms (${Math.round((splashScreenTime / totalStartupTime) * 100)}%)
- **Data Preloading Time**: ${dataPreloadingTime}ms (${Math.round((dataPreloadingTime / totalStartupTime) * 100)}%)
- **Screen Preloading Time**: ${screenPreloadingTime}ms (${Math.round((screenPreloadingTime / totalStartupTime) * 100)}%)

## Data Preloading
- **Preloaded Data Size**: ${Math.round(preloadedDataSize / 1024)}KB
- **Cache Hit Rate**: ${Math.round(cacheHitRate * 100)}%

## Performance Analysis
${this.generatePerformanceAnalysis()}

${errors.length > 0 ? `
## Errors
${errors.map(error => `- ${error}`).join('\n')}
` : '## No Errors'}

## Recommendations
${this.generateRecommendations()}
    `;

    return report.trim();
  }

  /**
   * Generates performance analysis
   */
  private generatePerformanceAnalysis(): string {
    if (!this.startupMetrics) return 'No metrics available';

    const { totalStartupTime, splashScreenTime, dataPreloadingTime } = this.startupMetrics;
    const analysis: string[] = [];

    if (totalStartupTime < 3000) {
      analysis.push('✅ Excellent startup performance (<3 seconds)');
    } else if (totalStartupTime < 5000) {
      analysis.push('⚠️ Good startup performance (3-5 seconds)');
    } else {
      analysis.push('❌ Slow startup performance (>5 seconds) - optimization needed');
    }

    if (this.config.parallelPreloading) {
      const efficiency = Math.max(splashScreenTime, dataPreloadingTime) / (splashScreenTime + dataPreloadingTime);
      if (efficiency > 0.7) {
        analysis.push('✅ Efficient parallel processing');
      } else {
        analysis.push('⚠️ Parallel processing could be more efficient');
      }
    }

    return analysis.join('\n');
  }

  /**
   * Generates optimization recommendations
   */
  private generateRecommendations(): string {
    if (!this.startupMetrics) return 'No metrics available for recommendations';

    const { totalStartupTime, cacheHitRate, errors } = this.startupMetrics;
    const recommendations: string[] = [];

    if (totalStartupTime > 5000) {
      recommendations.push('- Consider reducing initialization steps or implementing more aggressive caching');
    }

    if (cacheHitRate < 0.5) {
      recommendations.push('- Improve cache warming strategies to increase cache hit rate');
    }

    if (errors.length > 0) {
      recommendations.push('- Address startup errors to improve reliability');
    }

    if (!this.config.parallelPreloading) {
      recommendations.push('- Enable parallel preloading to improve perceived performance');
    }

    if (recommendations.length === 0) {
      recommendations.push('- Startup performance is optimal, no immediate improvements needed');
    }

    return recommendations.join('\n');
  }
}

// Export singleton instance
export const startupIntegrationService = new StartupIntegrationService();
export default StartupIntegrationService;