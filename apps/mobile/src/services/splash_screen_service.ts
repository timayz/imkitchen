/**
 * Splash Screen Service
 * 
 * Manages app initialization process with detailed progress tracking,
 * phase management, and error handling for the progressive splash screen.
 * 
 * Features:
 * - Multi-phase initialization tracking
 * - Step-by-step progress reporting
 * - Timeout and error handling
 * - Initialization priority management
 * - Recovery and retry mechanisms
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import { screenRegistry } from '../navigation/ScreenRegistry';
import { lazyLoadingService } from './lazy_loading_service';

export type InitializationPhase = 
  | 'loading' 
  | 'auth' 
  | 'data' 
  | 'cache' 
  | 'complete' 
  | 'error';

export type InitializationStep =
  | 'initializing_services'
  | 'checking_auth'
  | 'loading_user_preferences'
  | 'warming_recipe_cache'
  | 'preloading_critical_screens'
  | 'syncing_offline_data'
  | 'finalizing';

export interface InitializationConfig {
  enableProgressTracking: boolean;
  enableCacheWarming: boolean;
  enableScreenPreloading: boolean;
  enableOfflineSync: boolean;
  maxInitializationTime: number; // milliseconds
  criticalStepsOnly: boolean; // Skip non-essential steps
}

export interface InitializationMetrics {
  startTime: number;
  endTime?: number;
  totalDuration?: number;
  stepsCompleted: InitializationStep[];
  errors: Array<{ step: InitializationStep; error: string }>;
  performance: Record<InitializationStep, number>; // Duration per step
}

class SplashScreenService {
  private currentPhase: InitializationPhase = 'loading';
  private currentStep: InitializationStep | null = null;
  private progress = 0;
  private isInitializing = false;
  private metrics: InitializationMetrics | null = null;

  // Event handlers
  private progressHandler?: (step: InitializationStep, progress: number) => void;
  private phaseHandler?: (phase: InitializationPhase) => void;
  private errorHandler?: (error: Error) => void;
  private completeHandler?: () => void;

  private config: InitializationConfig = {
    enableProgressTracking: true,
    enableCacheWarming: true,
    enableScreenPreloading: true,
    enableOfflineSync: true,
    maxInitializationTime: 10000, // 10 seconds
    criticalStepsOnly: false
  };

  /**
   * Sets up event handlers for splash screen updates
   */
  onProgressUpdate(handler: (step: InitializationStep, progress: number) => void): void {
    this.progressHandler = handler;
  }

  onPhaseChange(handler: (phase: InitializationPhase) => void): void {
    this.phaseHandler = handler;
  }

  onError(handler: (error: Error) => void): void {
    this.errorHandler = handler;
  }

  onComplete(handler: () => void): void {
    this.completeHandler = handler;
  }

  /**
   * Initializes the app with progress tracking
   */
  async initializeApp(config?: Partial<InitializationConfig>): Promise<void> {
    if (this.isInitializing) {
      console.warn('[SplashScreen] Initialization already in progress');
      return;
    }

    this.isInitializing = true;
    this.config = { ...this.config, ...config };
    
    this.metrics = {
      startTime: Date.now(),
      stepsCompleted: [],
      errors: [],
      performance: {} as Record<InitializationStep, number>
    };

    console.log('[SplashScreen] Starting app initialization...');

    try {
      await this.executeInitializationSequence();
      
      this.metrics.endTime = Date.now();
      this.metrics.totalDuration = this.metrics.endTime - this.metrics.startTime;

      this.setPhase('complete');
      this.setProgress('finalizing', 100);
      
      console.log(`[SplashScreen] Initialization completed in ${this.metrics.totalDuration}ms`);
      
      // Small delay before completing to show 100% progress
      setTimeout(() => {
        this.completeHandler?.();
      }, 500);

    } catch (error) {
      console.error('[SplashScreen] Initialization failed:', error);
      this.handleError(error instanceof Error ? error : new Error('Unknown initialization error'));
    } finally {
      this.isInitializing = false;
    }
  }

  /**
   * Executes the complete initialization sequence
   */
  private async executeInitializationSequence(): Promise<void> {
    const steps: Array<{
      step: InitializationStep;
      phase: InitializationPhase;
      weight: number; // Percentage of total progress
      critical: boolean;
      action: () => Promise<void>;
    }> = [
      {
        step: 'initializing_services',
        phase: 'loading',
        weight: 15,
        critical: true,
        action: () => this.initializeServices()
      },
      {
        step: 'checking_auth',
        phase: 'auth',
        weight: 10,
        critical: true,
        action: () => this.checkAuthentication()
      },
      {
        step: 'loading_user_preferences',
        phase: 'data',
        weight: 15,
        critical: true,
        action: () => this.loadUserPreferences()
      },
      {
        step: 'warming_recipe_cache',
        phase: 'cache',
        weight: 20,
        critical: false,
        action: () => this.warmRecipeCache()
      },
      {
        step: 'preloading_critical_screens',
        phase: 'cache',
        weight: 25,
        critical: false,
        action: () => this.preloadCriticalScreens()
      },
      {
        step: 'syncing_offline_data',
        phase: 'cache',
        weight: 10,
        critical: false,
        action: () => this.syncOfflineData()
      },
      {
        step: 'finalizing',
        phase: 'complete',
        weight: 5,
        critical: true,
        action: () => this.finalize()
      }
    ];

    // Filter steps based on configuration
    const stepsToExecute = this.config.criticalStepsOnly 
      ? steps.filter(s => s.critical)
      : steps.filter(s => this.isStepEnabled(s.step));

    // Calculate cumulative progress for each step
    const totalWeight = stepsToExecute.reduce((sum, step) => sum + step.weight, 0);
    let cumulativeProgress = 0;

    for (const stepConfig of stepsToExecute) {
      const stepStartTime = Date.now();
      
      try {
        this.setPhase(stepConfig.phase);
        this.setProgress(stepConfig.step, cumulativeProgress);

        await stepConfig.action();

        const stepDuration = Date.now() - stepStartTime;
        this.metrics!.performance[stepConfig.step] = stepDuration;
        this.metrics!.stepsCompleted.push(stepConfig.step);

        cumulativeProgress += (stepConfig.weight / totalWeight) * 100;
        this.setProgress(stepConfig.step, Math.min(cumulativeProgress, 95)); // Cap at 95% until complete

        console.log(`[SplashScreen] Completed ${stepConfig.step} in ${stepDuration}ms`);

      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : 'Unknown error';
        this.metrics!.errors.push({ step: stepConfig.step, error: errorMsg });

        if (stepConfig.critical) {
          throw error; // Re-throw critical step failures
        } else {
          console.warn(`[SplashScreen] Non-critical step ${stepConfig.step} failed:`, error);
          // Continue with initialization for non-critical steps
        }
      }
    }
  }

  /**
   * Checks if a step should be executed based on configuration
   */
  private isStepEnabled(step: InitializationStep): boolean {
    switch (step) {
      case 'warming_recipe_cache':
        return this.config.enableCacheWarming;
      case 'preloading_critical_screens':
        return this.config.enableScreenPreloading;
      case 'syncing_offline_data':
        return this.config.enableOfflineSync;
      default:
        return true; // Core steps are always enabled
    }
  }

  /**
   * Initializes core services
   */
  private async initializeServices(): Promise<void> {
    // Simulate service initialization
    await this.delay(300);
    
    // Initialize any core services here
    console.log('[SplashScreen] Core services initialized');
  }

  /**
   * Checks user authentication status
   */
  private async checkAuthentication(): Promise<void> {
    try {
      // Check for stored authentication token
      const authToken = await AsyncStorage.getItem('auth_token');
      
      if (authToken) {
        // Validate token (simulate API call)
        await this.delay(500);
        console.log('[SplashScreen] Authentication validated');
      } else {
        console.log('[SplashScreen] No authentication found, will show login');
      }
    } catch (error) {
      console.warn('[SplashScreen] Authentication check failed:', error);
      // Non-critical error, continue initialization
    }
  }

  /**
   * Loads user preferences and settings
   */
  private async loadUserPreferences(): Promise<void> {
    try {
      // Load user preferences from storage
      const preferences = await AsyncStorage.getItem('user_preferences');
      const settings = await AsyncStorage.getItem('app_settings');
      
      await this.delay(200);
      
      console.log('[SplashScreen] User preferences loaded');
    } catch (error) {
      console.warn('[SplashScreen] Failed to load user preferences:', error);
      // Continue with default preferences
    }
  }

  /**
   * Warms up recipe cache for better performance
   */
  private async warmRecipeCache(): Promise<void> {
    try {
      // Simulate cache warming
      await this.delay(800);
      
      console.log('[SplashScreen] Recipe cache warmed');
    } catch (error) {
      console.warn('[SplashScreen] Cache warming failed:', error);
    }
  }

  /**
   * Preloads critical screens for faster navigation
   */
  private async preloadCriticalScreens(): Promise<void> {
    try {
      await screenRegistry.initialize();
      
      // Additional delay to simulate screen preloading
      await this.delay(600);
      
      console.log('[SplashScreen] Critical screens preloaded');
    } catch (error) {
      console.warn('[SplashScreen] Screen preloading failed:', error);
    }
  }

  /**
   * Syncs offline data in the background
   */
  private async syncOfflineData(): Promise<void> {
    try {
      // Simulate offline data sync
      await this.delay(400);
      
      console.log('[SplashScreen] Offline data synced');
    } catch (error) {
      console.warn('[SplashScreen] Offline sync failed:', error);
    }
  }

  /**
   * Finalizes initialization process
   */
  private async finalize(): Promise<void> {
    // Ensure minimum splash screen display time for UX
    const minDisplayTime = 2000; // 2 seconds minimum
    const elapsedTime = Date.now() - this.metrics!.startTime;
    
    if (elapsedTime < minDisplayTime) {
      await this.delay(minDisplayTime - elapsedTime);
    }

    console.log('[SplashScreen] Initialization finalized');
  }

  /**
   * Updates current phase
   */
  private setPhase(phase: InitializationPhase): void {
    if (this.currentPhase !== phase) {
      this.currentPhase = phase;
      this.phaseHandler?.(phase);
    }
  }

  /**
   * Updates current step and progress
   */
  private setProgress(step: InitializationStep, progress: number): void {
    this.currentStep = step;
    this.progress = Math.max(0, Math.min(100, progress));
    
    if (this.config.enableProgressTracking) {
      this.progressHandler?.(step, this.progress);
    }
  }

  /**
   * Handles initialization errors
   */
  private handleError(error: Error): void {
    this.setPhase('error');
    this.errorHandler?.(error);
  }

  /**
   * Utility delay function
   */
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Gets initialization metrics
   */
  getMetrics(): InitializationMetrics | null {
    return this.metrics;
  }

  /**
   * Gets current initialization state
   */
  getState(): {
    phase: InitializationPhase;
    step: InitializationStep | null;
    progress: number;
    isInitializing: boolean;
  } {
    return {
      phase: this.currentPhase,
      step: this.currentStep,
      progress: this.progress,
      isInitializing: this.isInitializing
    };
  }

  /**
   * Retries initialization with exponential backoff
   */
  async retryInitialization(maxRetries = 3): Promise<void> {
    let retryCount = 0;
    let lastError: Error | null = null;

    while (retryCount < maxRetries) {
      try {
        await this.initializeApp({ criticalStepsOnly: retryCount > 0 });
        return; // Success
      } catch (error) {
        lastError = error instanceof Error ? error : new Error('Unknown retry error');
        retryCount++;
        
        if (retryCount < maxRetries) {
          const delay = Math.pow(2, retryCount) * 1000; // Exponential backoff
          console.log(`[SplashScreen] Retry ${retryCount}/${maxRetries} after ${delay}ms delay`);
          await this.delay(delay);
        }
      }
    }

    // All retries failed
    throw lastError || new Error('All initialization retries failed');
  }

  /**
   * Cleans up resources and event handlers
   */
  cleanup(): void {
    this.progressHandler = undefined;
    this.phaseHandler = undefined;
    this.errorHandler = undefined;
    this.completeHandler = undefined;
    this.isInitializing = false;
    
    console.log('[SplashScreen] Cleanup completed');
  }

  /**
   * Generates initialization report for debugging
   */
  generateReport(): string {
    if (!this.metrics) {
      return 'No initialization metrics available';
    }

    const { totalDuration, stepsCompleted, errors, performance } = this.metrics;

    let report = `
# Splash Screen Initialization Report

## Summary
- **Total Duration**: ${totalDuration}ms
- **Steps Completed**: ${stepsCompleted.length}
- **Errors**: ${errors.length}

## Step Performance
${Object.entries(performance)
  .map(([step, duration]) => `- ${step}: ${duration}ms`)
  .join('\n')}

## Completed Steps
${stepsCompleted.map(step => `- ✓ ${step}`).join('\n')}

${errors.length > 0 ? `
## Errors
${errors.map(({ step, error }) => `- ✗ ${step}: ${error}`).join('\n')}
` : ''}
    `;

    return report.trim();
  }
}

// Export singleton instance
export const splashScreenService = new SplashScreenService();
export default SplashScreenService;