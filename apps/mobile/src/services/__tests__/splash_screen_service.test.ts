/**
 * Splash Screen Service Tests
 * 
 * Tests for multi-phase app initialization, progress tracking,
 * and error handling during startup
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import { splashScreenService } from '../splash_screen_service';
import { screenRegistry } from '../../navigation/ScreenRegistry';
import type { InitializationPhase, InitializationStep } from '../splash_screen_service';

// Mock AsyncStorage
jest.mock('@react-native-async-storage/async-storage');
const mockedAsyncStorage = AsyncStorage as jest.Mocked<typeof AsyncStorage>;

// Mock screen registry
jest.mock('../../navigation/ScreenRegistry', () => ({
  screenRegistry: {
    initialize: jest.fn().mockResolvedValue(undefined)
  }
}));

const mockedScreenRegistry = screenRegistry as jest.Mocked<typeof screenRegistry>;

describe('SplashScreenService', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockedAsyncStorage.getItem.mockResolvedValue(null);
    mockedAsyncStorage.setItem.mockResolvedValue(undefined);
    splashScreenService.cleanup();
  });

  afterEach(() => {
    splashScreenService.cleanup();
  });

  describe('initializeApp', () => {
    it('should execute initialization sequence with progress tracking', async () => {
      const progressUpdates: Array<{ step: InitializationStep; progress: number }> = [];
      const phaseChanges: InitializationPhase[] = [];
      
      splashScreenService.onProgressUpdate((step, progress) => {
        progressUpdates.push({ step, progress });
      });
      
      splashScreenService.onPhaseChange((phase) => {
        phaseChanges.push(phase);
      });

      let completeCalled = false;
      splashScreenService.onComplete(() => {
        completeCalled = true;
      });

      await splashScreenService.initializeApp();

      // Should have progress updates
      expect(progressUpdates.length).toBeGreaterThan(0);
      expect(progressUpdates[progressUpdates.length - 1].progress).toBe(100);
      
      // Should have phase changes
      expect(phaseChanges).toContain('loading');
      expect(phaseChanges).toContain('auth');
      expect(phaseChanges).toContain('data');
      expect(phaseChanges).toContain('complete');
      
      // Should complete
      expect(completeCalled).toBe(true);
    });

    it('should handle critical step failures by stopping initialization', async () => {
      // Mock AsyncStorage failure for critical step
      mockedAsyncStorage.getItem.mockRejectedValueOnce(new Error('Storage failure'));

      let errorCalled = false;
      let capturedError: Error | null = null;
      
      splashScreenService.onError((error) => {
        errorCalled = true;
        capturedError = error;
      });

      await expect(splashScreenService.initializeApp()).rejects.toThrow();
      
      expect(errorCalled).toBe(true);
      expect(capturedError).toBeInstanceOf(Error);
    });

    it('should continue initialization when non-critical steps fail', async () => {
      // Mock screen registry failure (non-critical step)
      mockedScreenRegistry.initialize.mockRejectedValueOnce(new Error('Screen init failed'));

      let completeCalled = false;
      splashScreenService.onComplete(() => {
        completeCalled = true;
      });

      await splashScreenService.initializeApp();

      // Should still complete despite non-critical failure
      expect(completeCalled).toBe(true);
    });

    it('should skip non-critical steps when criticalStepsOnly is enabled', async () => {
      const progressUpdates: Array<{ step: InitializationStep; progress: number }> = [];
      
      splashScreenService.onProgressUpdate((step, progress) => {
        progressUpdates.push({ step, progress });
      });

      await splashScreenService.initializeApp({ criticalStepsOnly: true });

      const steps = progressUpdates.map(update => update.step);
      
      // Should include critical steps
      expect(steps).toContain('initializing_services');
      expect(steps).toContain('checking_auth');
      expect(steps).toContain('loading_user_preferences');
      
      // Should not include non-critical steps
      expect(steps).not.toContain('warming_recipe_cache');
      expect(steps).not.toContain('preloading_critical_screens');
    });

    it('should handle concurrent initialization attempts', async () => {
      const initPromise1 = splashScreenService.initializeApp();
      const initPromise2 = splashScreenService.initializeApp();

      await initPromise1;
      await initPromise2; // Should not throw or cause issues

      // Both should complete successfully
      expect(initPromise1).resolves.toBeUndefined();
      expect(initPromise2).resolves.toBeUndefined();
    });
  });

  describe('step execution', () => {
    it('should initialize services successfully', async () => {
      const progressUpdates: Array<{ step: InitializationStep; progress: number }> = [];
      
      splashScreenService.onProgressUpdate((step, progress) => {
        progressUpdates.push({ step, progress });
      });

      await splashScreenService.initializeApp();

      const servicesStep = progressUpdates.find(
        update => update.step === 'initializing_services'
      );
      
      expect(servicesStep).toBeDefined();
      expect(servicesStep!.progress).toBeGreaterThan(0);
    });

    it('should check authentication and handle missing token', async () => {
      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'auth_token') {
          return Promise.resolve(null); // No token
        }
        return Promise.resolve(null);
      });

      let completeCalled = false;
      splashScreenService.onComplete(() => {
        completeCalled = true;
      });

      await splashScreenService.initializeApp();

      // Should still complete successfully even without auth token
      expect(completeCalled).toBe(true);
    });

    it('should load user preferences from storage', async () => {
      const mockPreferences = { theme: 'dark', language: 'en' };
      const mockSettings = { notifications: true };

      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'user_preferences') {
          return Promise.resolve(JSON.stringify(mockPreferences));
        }
        if (key === 'app_settings') {
          return Promise.resolve(JSON.stringify(mockSettings));
        }
        return Promise.resolve(null);
      });

      const progressUpdates: Array<{ step: InitializationStep; progress: number }> = [];
      
      splashScreenService.onProgressUpdate((step, progress) => {
        progressUpdates.push({ step, progress });
      });

      await splashScreenService.initializeApp();

      const preferencesStep = progressUpdates.find(
        update => update.step === 'loading_user_preferences'
      );
      
      expect(preferencesStep).toBeDefined();
      expect(mockedAsyncStorage.getItem).toHaveBeenCalledWith('user_preferences');
      expect(mockedAsyncStorage.getItem).toHaveBeenCalledWith('app_settings');
    });

    it('should preload critical screens when enabled', async () => {
      await splashScreenService.initializeApp({
        enableScreenPreloading: true
      });

      expect(mockedScreenRegistry.initialize).toHaveBeenCalled();
    });
  });

  describe('configuration options', () => {
    it('should respect enableCacheWarming configuration', async () => {
      const progressUpdates: Array<{ step: InitializationStep; progress: number }> = [];
      
      splashScreenService.onProgressUpdate((step, progress) => {
        progressUpdates.push({ step, progress });
      });

      await splashScreenService.initializeApp({
        enableCacheWarming: false
      });

      const steps = progressUpdates.map(update => update.step);
      expect(steps).not.toContain('warming_recipe_cache');
    });

    it('should respect enableScreenPreloading configuration', async () => {
      const progressUpdates: Array<{ step: InitializationStep; progress: number }> = [];
      
      splashScreenService.onProgressUpdate((step, progress) => {
        progressUpdates.push({ step, progress });
      });

      await splashScreenService.initializeApp({
        enableScreenPreloading: false
      });

      const steps = progressUpdates.map(update => update.step);
      expect(steps).not.toContain('preloading_critical_screens');
      expect(mockedScreenRegistry.initialize).not.toHaveBeenCalled();
    });

    it('should respect enableOfflineSync configuration', async () => {
      const progressUpdates: Array<{ step: InitializationStep; progress: number }> = [];
      
      splashScreenService.onProgressUpdate((step, progress) => {
        progressUpdates.push({ step, progress });
      });

      await splashScreenService.initializeApp({
        enableOfflineSync: false
      });

      const steps = progressUpdates.map(update => update.step);
      expect(steps).not.toContain('syncing_offline_data');
    });
  });

  describe('error handling and recovery', () => {
    it('should retry initialization with exponential backoff', async () => {
      let attemptCount = 0;
      mockedAsyncStorage.getItem.mockImplementation((key) => {
        attemptCount++;
        if (key === 'auth_token' && attemptCount <= 2) {
          return Promise.reject(new Error('Temporary failure'));
        }
        return Promise.resolve(null);
      });

      await splashScreenService.retryInitialization(3);

      expect(attemptCount).toBeGreaterThan(2); // Should have retried
    });

    it('should fail after max retries exceeded', async () => {
      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'auth_token') {
          return Promise.reject(new Error('Persistent failure'));
        }
        return Promise.resolve(null);
      });

      await expect(
        splashScreenService.retryInitialization(2)
      ).rejects.toThrow('Persistent failure');
    });

    it('should use critical steps only during retry', async () => {
      let attemptCount = 0;
      const progressUpdates: Array<{ step: InitializationStep; progress: number }> = [];
      
      splashScreenService.onProgressUpdate((step, progress) => {
        progressUpdates.push({ step, progress });
      });

      mockedAsyncStorage.getItem.mockImplementation((key) => {
        attemptCount++;
        if (key === 'auth_token' && attemptCount === 1) {
          return Promise.reject(new Error('First attempt failure'));
        }
        return Promise.resolve(null);
      });

      await splashScreenService.retryInitialization(2);

      // Second attempt should use critical steps only
      const steps = progressUpdates.map(update => update.step);
      const nonCriticalSteps = ['warming_recipe_cache', 'preloading_critical_screens'];
      
      // Should have fewer non-critical steps in retry
      const nonCriticalCount = steps.filter(step => 
        nonCriticalSteps.includes(step)
      ).length;
      
      expect(nonCriticalCount).toBeLessThan(2); // Reduced non-critical steps
    });
  });

  describe('metrics and reporting', () => {
    it('should track initialization metrics', async () => {
      await splashScreenService.initializeApp();

      const metrics = splashScreenService.getMetrics();
      
      expect(metrics).toBeDefined();
      expect(metrics!.startTime).toBeInstanceOf(Number);
      expect(metrics!.endTime).toBeInstanceOf(Number);
      expect(metrics!.totalDuration).toBeInstanceOf(Number);
      expect(metrics!.stepsCompleted).toBeInstanceOf(Array);
      expect(metrics!.performance).toBeInstanceOf(Object);
    });

    it('should generate initialization report', async () => {
      await splashScreenService.initializeApp();

      const report = splashScreenService.generateReport();
      
      expect(report).toContain('Splash Screen Initialization Report');
      expect(report).toContain('Total Duration');
      expect(report).toContain('Steps Completed');
      expect(report).toContain('Step Performance');
      expect(report).toContain('✓'); // Completed step marker
    });

    it('should track step performance timing', async () => {
      await splashScreenService.initializeApp();

      const metrics = splashScreenService.getMetrics();
      
      expect(metrics!.performance).toBeDefined();
      expect(Object.keys(metrics!.performance).length).toBeGreaterThan(0);
      
      // Each step should have a duration
      Object.values(metrics!.performance).forEach(duration => {
        expect(duration).toBeGreaterThan(0);
      });
    });
  });

  describe('state management', () => {
    it('should provide current state information', () => {
      const state = splashScreenService.getState();
      
      expect(state).toHaveProperty('phase');
      expect(state).toHaveProperty('step');
      expect(state).toHaveProperty('progress');
      expect(state).toHaveProperty('isInitializing');
      
      expect(['loading', 'auth', 'data', 'cache', 'complete', 'error']).toContain(state.phase);
      expect(typeof state.progress).toBe('number');
      expect(typeof state.isInitializing).toBe('boolean');
    });

    it('should cleanup resources properly', () => {
      let progressCalled = false;
      let phaseCalled = false;
      
      splashScreenService.onProgressUpdate(() => {
        progressCalled = true;
      });
      
      splashScreenService.onPhaseChange(() => {
        phaseCalled = true;
      });

      splashScreenService.cleanup();

      // After cleanup, callbacks should be cleared
      const state = splashScreenService.getState();
      expect(state.isInitializing).toBe(false);
    });
  });
});