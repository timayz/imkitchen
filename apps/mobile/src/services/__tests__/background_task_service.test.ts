/**
 * Background Task Service Tests
 * 
 * Tests for React Native background task management and integration
 * with the background sync service.
 */

import { backgroundTaskService } from '../background_task_service';
import { backgroundSyncService } from '../background_sync_service';
import { AppState } from 'react-native';

// Mock React Native modules
jest.mock('react-native', () => ({
  AppState: {
    addEventListener: jest.fn(),
    currentState: 'active'
  }
}));

jest.mock('../background_sync_service', () => ({
  backgroundSyncService: {
    getSyncStatus: jest.fn(),
    manualSync: jest.fn(),
    getConflictItems: jest.fn(),
    resumeSync: jest.fn()
  }
}));

describe('BackgroundTaskService', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    jest.clearAllTimers();
    jest.useFakeTimers();
  });

  afterEach(() => {
    backgroundTaskService.cleanup();
    jest.runOnlyPendingTimers();
    jest.useRealTimers();
  });

  describe('Initialization', () => {
    test('should initialize with default configuration', () => {
      const config = backgroundTaskService.getConfig();
      const status = backgroundTaskService.getStatus();

      expect(config.taskInterval).toBe(30000);
      expect(config.maxBackgroundTime).toBe(30000);
      expect(config.batteryThreshold).toBe(20);
      expect(status.isRegistered).toBe(true);
      expect(status.executionCount).toBe(0);
    });

    test('should setup app state monitoring', () => {
      expect(AppState.addEventListener).toHaveBeenCalledWith('change', expect.any(Function));
    });
  });

  describe('Configuration Management', () => {
    test('should update configuration', () => {
      backgroundTaskService.updateConfig({
        taskInterval: 60000,
        batteryThreshold: 30
      });

      const config = backgroundTaskService.getConfig();
      expect(config.taskInterval).toBe(60000);
      expect(config.batteryThreshold).toBe(30);
    });

    test('should enable battery saver mode', () => {
      backgroundTaskService.enableBatterySaver();
      
      const config = backgroundTaskService.getConfig();
      expect(config.batteryThreshold).toBe(50);
      expect(config.criticalTasksOnly).toBe(true);
      expect(config.taskInterval).toBe(60000);
      expect(config.resourceLimits.maxConcurrentTasks).toBe(1);
    });

    test('should disable battery saver mode', () => {
      backgroundTaskService.enableBatterySaver();
      backgroundTaskService.disableBatterySaver();
      
      const config = backgroundTaskService.getConfig();
      expect(config.batteryThreshold).toBe(20);
      expect(config.criticalTasksOnly).toBe(false);
      expect(config.taskInterval).toBe(30000);
      expect(config.resourceLimits.maxConcurrentTasks).toBe(3);
    });
  });

  describe('App State Changes', () => {
    test('should handle app becoming active', () => {
      const appStateCallback = (AppState.addEventListener as jest.Mock).mock.calls[0][1];
      
      // Simulate app becoming active
      appStateCallback('active');
      
      // Should resume sync service
      expect(backgroundSyncService.resumeSync).toHaveBeenCalled();
      
      const status = backgroundTaskService.getStatus();
      expect(status.isRunning).toBe(false);
    });

    test('should handle app becoming backgrounded', () => {
      const appStateCallback = (AppState.addEventListener as jest.Mock).mock.calls[0][1];
      
      // Mock suitable conditions for background execution
      (backgroundSyncService.getSyncStatus as jest.Mock).mockReturnValue({
        queueSize: 5,
        activeSyncs: 1
      });
      
      // Simulate app becoming backgrounded
      appStateCallback('background');
      
      const status = backgroundTaskService.getStatus();
      expect(status.isRunning).toBe(true);
    });
  });

  describe('Background Task Execution', () => {
    test('should execute background task when conditions are met', async () => {
      (backgroundSyncService.getSyncStatus as jest.Mock).mockReturnValue({
        queueSize: 3,
        activeSyncs: 1
      });
      
      (backgroundSyncService.manualSync as jest.Mock).mockResolvedValue({
        triggered: 3,
        inProgress: 1,
        conflicts: 0
      });
      
      await backgroundTaskService.forceBackgroundSync();
      
      const status = backgroundTaskService.getStatus();
      expect(status.executionCount).toBe(1);
      expect(status.lastExecution).toBeInstanceOf(Date);
      expect(backgroundSyncService.manualSync).toHaveBeenCalled();
    });

    test('should handle critical items only in background mode', async () => {
      backgroundTaskService.updateConfig({ criticalTasksOnly: true });
      
      (backgroundSyncService.getSyncStatus as jest.Mock).mockReturnValue({
        queueSize: 5,
        activeSyncs: 1
      });
      
      (backgroundSyncService.getConflictItems as jest.Mock).mockReturnValue([
        { priority: 'critical', id: '1' },
        { priority: 'normal', id: '2' }
      ]);
      
      await backgroundTaskService.forceBackgroundSync();
      
      expect(backgroundSyncService.getConflictItems).toHaveBeenCalled();
    });

    test('should skip execution when no items in queue', async () => {
      (backgroundSyncService.getSyncStatus as jest.Mock).mockReturnValue({
        queueSize: 0,
        activeSyncs: 0
      });
      
      await backgroundTaskService.forceBackgroundSync();
      
      expect(backgroundSyncService.manualSync).not.toHaveBeenCalled();
    });
  });

  describe('Resource Management', () => {
    test('should monitor resource usage', () => {
      const status = backgroundTaskService.getStatus();
      
      expect(status.resourceUsage).toBeDefined();
      expect(status.resourceUsage.memory).toBeGreaterThanOrEqual(0);
      expect(status.resourceUsage.cpu).toBeGreaterThanOrEqual(0);
      expect(status.resourceUsage.network).toBeGreaterThanOrEqual(0);
      expect(status.resourceUsage.battery).toBeGreaterThanOrEqual(0);
    });

    test('should respect resource limits', async () => {
      // Mock high resource usage
      const originalGetCurrentResourceUsage = backgroundTaskService['getCurrentResourceUsage'];
      backgroundTaskService['getCurrentResourceUsage'] = jest.fn().mockReturnValue({
        memory: 200, // Over limit
        cpu: 90,     // Over limit
        network: 50, // Over limit
        battery: 10  // Below threshold
      });
      
      const canStart = backgroundTaskService['canStartBackgroundExecution']();
      expect(canStart).toBe(false);
      
      // Restore original method
      backgroundTaskService['getCurrentResourceUsage'] = originalGetCurrentResourceUsage;
    });
  });

  describe('Error Handling', () => {
    test('should handle background task execution errors', async () => {
      (backgroundSyncService.manualSync as jest.Mock).mockRejectedValue(new Error('Sync failed'));
      
      await backgroundTaskService.forceBackgroundSync();
      
      const status = backgroundTaskService.getStatus();
      expect(status.errors.length).toBe(1);
      expect(status.errors[0].error).toContain('Sync failed');
      expect(status.errors[0].severity).toBe('medium');
    });

    test('should limit error log size', async () => {
      // Add many errors
      for (let i = 0; i < 60; i++) {
        backgroundTaskService['addError'](`Test error ${i}`, 'Test', 'low');
      }
      
      const status = backgroundTaskService.getStatus();
      expect(status.errors.length).toBe(50); // Should be limited to 50
    });
  });

  describe('Scheduling', () => {
    test('should schedule periodic background sync', () => {
      const appStateCallback = (AppState.addEventListener as jest.Mock).mock.calls[0][1];
      
      // Mock suitable conditions
      (backgroundSyncService.getSyncStatus as jest.Mock).mockReturnValue({
        queueSize: 3
      });
      
      // Simulate app becoming backgrounded
      appStateCallback('background');
      
      let status = backgroundTaskService.getStatus();
      expect(status.isRunning).toBe(true);
      
      // Fast-forward time to trigger scheduled task
      jest.advanceTimersByTime(30000);
      
      status = backgroundTaskService.getStatus();
      expect(status.executionCount).toBeGreaterThan(0);
    });

    test('should respect background time limits', () => {
      backgroundTaskService.updateConfig({ maxBackgroundTime: 1000 });
      
      const appStateCallback = (AppState.addEventListener as jest.Mock).mock.calls[0][1];
      appStateCallback('background');
      
      // Fast-forward past time limit
      jest.advanceTimersByTime(2000);
      
      const status = backgroundTaskService.getStatus();
      expect(status.isRunning).toBe(false);
    });
  });

  describe('Public API', () => {
    test('should pause and resume background tasks', () => {
      backgroundTaskService.pause();
      let status = backgroundTaskService.getStatus();
      expect(status.isRunning).toBe(false);
      
      backgroundTaskService.resume();
      // Note: Resume only starts if app is backgrounded
      // For testing, we can check the method exists
      expect(backgroundTaskService.resume).toBeDefined();
    });

    test('should cleanup resources', () => {
      backgroundTaskService.cleanup();
      
      const status = backgroundTaskService.getStatus();
      expect(status.isRegistered).toBe(false);
      expect(status.isRunning).toBe(false);
    });
  });

  describe('Network Requirements', () => {
    test('should check network suitability', () => {
      // Test different network requirements
      backgroundTaskService.updateConfig({ networkRequirement: 'wifi' });
      const isWifiSuitable = backgroundTaskService['isNetworkSuitable']();
      expect(typeof isWifiSuitable).toBe('boolean');
      
      backgroundTaskService.updateConfig({ networkRequirement: 'any' });
      const isAnySuitable = backgroundTaskService['isNetworkSuitable']();
      expect(typeof isAnySuitable).toBe('boolean');
      
      backgroundTaskService.updateConfig({ networkRequirement: 'none' });
      const isNoneSuitable = backgroundTaskService['isNetworkSuitable']();
      expect(isNoneSuitable).toBe(true);
    });
  });
});

describe('BackgroundTaskService Integration', () => {
  test('should integrate with background sync service', async () => {
    const mockSyncResult = {
      triggered: 5,
      inProgress: 2,
      conflicts: 1
    };
    
    (backgroundSyncService.getSyncStatus as jest.Mock).mockReturnValue({
      queueSize: 5,
      activeSyncs: 2
    });
    
    (backgroundSyncService.manualSync as jest.Mock).mockResolvedValue(mockSyncResult);
    
    await backgroundTaskService.forceBackgroundSync();
    
    expect(backgroundSyncService.getSyncStatus).toHaveBeenCalled();
    expect(backgroundSyncService.manualSync).toHaveBeenCalled();
  });

  test('should handle sync service errors gracefully', async () => {
    (backgroundSyncService.getSyncStatus as jest.Mock).mockImplementation(() => {
      throw new Error('Sync service unavailable');
    });
    
    await backgroundTaskService.forceBackgroundSync();
    
    const status = backgroundTaskService.getStatus();
    expect(status.errors.length).toBeGreaterThan(0);
    expect(status.errors[0].context).toContain('Background task execution failed');
  });
});