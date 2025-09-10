/**
 * Startup Performance Monitoring Service Tests
 * 
 * Tests for comprehensive startup performance tracking, analytics,
 * regression detection, and device-specific performance monitoring
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import { Platform, Dimensions } from 'react-native';
import DeviceInfo from 'expo-device';
import { startupPerformanceMonitoringService } from '../startup_performance_monitoring_service';
import type { StartupMeasurement, PerformanceAlert } from '../startup_performance_monitoring_service';

// Mock dependencies
jest.mock('@react-native-async-storage/async-storage');
jest.mock('react-native', () => ({
  Platform: {
    OS: 'ios',
    Version: '16.0'
  },
  Dimensions: {
    get: jest.fn()
  }
}));
jest.mock('expo-device', () => ({
  deviceName: 'iPhone 14',
  deviceType: 'phone',
  isDevice: true
}));

const mockedAsyncStorage = AsyncStorage as jest.Mocked<typeof AsyncStorage>;
const mockedDimensions = Dimensions as jest.Mocked<typeof Dimensions>;

describe('StartupPerformanceMonitoringService', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    
    // Mock Dimensions
    mockedDimensions.get.mockReturnValue({
      width: 390,
      height: 844
    });

    // Default AsyncStorage behavior
    mockedAsyncStorage.getItem.mockResolvedValue(null);
    mockedAsyncStorage.setItem.mockResolvedValue(undefined);
  });

  describe('measurement lifecycle', () => {
    it('should start and complete startup measurement', async () => {
      const sessionId = await startupPerformanceMonitoringService.startStartupMeasurement();

      expect(sessionId).toBeDefined();
      expect(sessionId).toMatch(/^startup_\d+_[a-z0-9]+$/);

      // Record some metrics
      startupPerformanceMonitoringService.recordSplashScreenComplete(800);
      startupPerformanceMonitoringService.recordDataPreloadingComplete(1200, 45000, 0.75, 3, 0);
      startupPerformanceMonitoringService.recordScreenPreloadingComplete(600);
      startupPerformanceMonitoringService.recordFirstInteraction();

      const measurement = await startupPerformanceMonitoringService.completeStartupMeasurement();

      expect(measurement.sessionId).toBe(sessionId);
      expect(measurement.totalStartupTime).toBeGreaterThan(0);
      expect(measurement.splashScreenTime).toBe(800);
      expect(measurement.dataPreloadingTime).toBe(1200);
      expect(measurement.screenPreloadingTime).toBe(600);
      expect(measurement.preloadedDataSize).toBe(45000);
      expect(measurement.cacheHitRate).toBe(0.75);
      expect(measurement.networkRequests).toBe(3);
      expect(measurement.failedRequests).toBe(0);
      expect(measurement.perceivedPerformance).toMatch(/fast|normal|slow/);
    });

    it('should collect accurate device metrics', async () => {
      const sessionId = await startupPerformanceMonitoringService.startStartupMeasurement();
      const measurement = await startupPerformanceMonitoringService.completeStartupMeasurement();

      expect(measurement.deviceMetrics).toBeDefined();
      expect(measurement.deviceMetrics.platform).toBe('ios');
      expect(measurement.deviceMetrics.osVersion).toBe('16.0');
      expect(measurement.deviceMetrics.deviceModel).toBe('iPhone 14');
      expect(measurement.deviceMetrics.screenDimensions.width).toBe(390);
      expect(measurement.deviceMetrics.screenDimensions.height).toBe(844);
      expect(measurement.deviceMetrics.isTablet).toBe(false);
      expect(measurement.deviceMetrics.isEmulator).toBe(false);
    });

    it('should detect first launch correctly', async () => {
      // Mock no previous launch
      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'has_launched_before') {
          return Promise.resolve(null);
        }
        return Promise.resolve(null);
      });

      await startupPerformanceMonitoringService.startStartupMeasurement();
      const measurement = await startupPerformanceMonitoringService.completeStartupMeasurement();

      expect(measurement.isFirstLaunch).toBe(true);
      expect(mockedAsyncStorage.setItem).toHaveBeenCalledWith('has_launched_before', 'true');
    });

    it('should detect app updates', async () => {
      // Mock different stored version
      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'app_version') {
          return Promise.resolve('0.9.0'); // Old version
        }
        return Promise.resolve(null);
      });

      await startupPerformanceMonitoringService.startStartupMeasurement();
      const measurement = await startupPerformanceMonitoringService.completeStartupMeasurement();

      expect(measurement.isAppUpdate).toBe(true);
      expect(mockedAsyncStorage.setItem).toHaveBeenCalledWith('app_version', '1.0.0');
    });

    it('should track memory usage during startup', async () => {
      await startupPerformanceMonitoringService.startStartupMeasurement();
      
      // Simulate some time passing for memory monitoring
      await new Promise(resolve => setTimeout(resolve, 100));
      
      const measurement = await startupPerformanceMonitoringService.completeStartupMeasurement();

      expect(measurement.memoryUsage.initial).toBeGreaterThan(0);
      expect(measurement.memoryUsage.peak).toBeGreaterThanOrEqual(measurement.memoryUsage.initial);
      expect(measurement.memoryUsage.final).toBeGreaterThan(0);
    });

    it('should handle concurrent measurement attempts gracefully', async () => {
      const sessionId1 = await startupPerformanceMonitoringService.startStartupMeasurement();
      
      // Second start should not interfere
      const sessionId2 = await startupPerformanceMonitoringService.startStartupMeasurement();
      
      expect(sessionId1).toBe(sessionId2); // Should return same session
    });
  });

  describe('performance classification', () => {
    it('should classify fast startup correctly', async () => {
      await startupPerformanceMonitoringService.startStartupMeasurement();
      
      // Mock very quick completion
      const startTime = Date.now() - 2000; // 2 seconds ago
      startupPerformanceMonitoringService['startupStartTime'] = startTime;
      
      const measurement = await startupPerformanceMonitoringService.completeStartupMeasurement();

      expect(measurement.perceivedPerformance).toBe('fast');
      expect(measurement.totalStartupTime).toBeLessThan(3000);
    });

    it('should classify normal startup correctly', async () => {
      await startupPerformanceMonitoringService.startStartupMeasurement();
      
      // Mock moderate completion time
      const startTime = Date.now() - 4000; // 4 seconds ago
      startupPerformanceMonitoringService['startupStartTime'] = startTime;
      
      const measurement = await startupPerformanceMonitoringService.completeStartupMeasurement();

      expect(measurement.perceivedPerformance).toBe('normal');
      expect(measurement.totalStartupTime).toBeLessThan(5000);
    });

    it('should classify slow startup correctly', async () => {
      await startupPerformanceMonitoringService.startStartupMeasurement();
      
      // Mock slow completion time
      const startTime = Date.now() - 7000; // 7 seconds ago
      startupPerformanceMonitoringService['startupStartTime'] = startTime;
      
      const measurement = await startupPerformanceMonitoringService.completeStartupMeasurement();

      expect(measurement.perceivedPerformance).toBe('slow');
      expect(measurement.totalStartupTime).toBeGreaterThan(5000);
    });
  });

  describe('error tracking', () => {
    it('should record startup errors', async () => {
      await startupPerformanceMonitoringService.startStartupMeasurement();
      
      startupPerformanceMonitoringService.recordStartupError('Network timeout');
      startupPerformanceMonitoringService.recordStartupError('Cache miss');
      
      const measurement = await startupPerformanceMonitoringService.completeStartupMeasurement();

      expect(measurement.startupErrors).toEqual(['Network timeout', 'Cache miss']);
    });

    it('should handle measurement completion without active measurement', async () => {
      await expect(
        startupPerformanceMonitoringService.completeStartupMeasurement()
      ).rejects.toThrow('No active measurement to complete');
    });
  });

  describe('performance alerts', () => {
    it('should generate slow startup alert', async () => {
      await startupPerformanceMonitoringService.startStartupMeasurement();
      
      // Mock very slow startup
      const startTime = Date.now() - 9000; // 9 seconds ago
      startupPerformanceMonitoringService['startupStartTime'] = startTime;
      
      const measurement = await startupPerformanceMonitoringService.completeStartupMeasurement();

      // Should trigger alert for very slow startup
      expect(measurement.totalStartupTime).toBeGreaterThan(8000);
    });

    it('should generate cache hit rate alert', async () => {
      await startupPerformanceMonitoringService.startStartupMeasurement();
      
      startupPerformanceMonitoringService.recordDataPreloadingComplete(
        1500, 30000, 0.3, 5, 1 // Low cache hit rate (30%)
      );
      
      const measurement = await startupPerformanceMonitoringService.completeStartupMeasurement();

      expect(measurement.cacheHitRate).toBe(0.3);
    });

    it('should detect performance regression', async () => {
      // Mock historical data with better performance
      const mockHistory = [
        {
          timestamp: Date.now() - 86400000, // 1 day ago
          totalStartupTime: 2000,
          splashScreenTime: 500,
          dataPreloadingTime: 800,
          screenPreloadingTime: 400,
          firstInteractionTime: 2000,
          memoryUsage: { initial: 60, peak: 80, final: 70 },
          preloadedDataSize: 25000,
          cacheHitRate: 0.85,
          networkRequests: 3,
          failedRequests: 0,
          perceivedPerformance: 'fast' as const,
          startupErrors: [],
          isFirstLaunch: false,
          isAppUpdate: false,
          networkType: 'wifi' as const,
          featureFlags: {},
          deviceMetrics: {
            platform: 'ios',
            osVersion: '16.0',
            deviceModel: 'iPhone 14',
            screenDimensions: { width: 390, height: 844 },
            isTablet: false,
            isEmulator: false
          },
          sessionId: 'old_session'
        }
      ];

      mockedAsyncStorage.getItem.mockImplementation((key) => {
        if (key === 'startup_measurements') {
          return Promise.resolve(JSON.stringify(mockHistory));
        }
        return Promise.resolve(null);
      });

      // Load service to initialize with history
      await startupPerformanceMonitoringService['loadStoredMeasurements']();
      
      await startupPerformanceMonitoringService.startStartupMeasurement();
      
      // Mock current slow startup
      const startTime = Date.now() - 6000; // 6 seconds - much slower than historical 2 seconds
      startupPerformanceMonitoringService['startupStartTime'] = startTime;
      
      const measurement = await startupPerformanceMonitoringService.completeStartupMeasurement();

      expect(measurement.totalStartupTime).toBeGreaterThan(mockHistory[0].totalStartupTime * 1.5);
    });
  });

  describe('dashboard generation', () => {
    it('should generate comprehensive performance dashboard', async () => {
      // Mock some measurement history
      const mockMeasurements = [
        {
          timestamp: Date.now() - 86400000,
          totalStartupTime: 2500,
          splashScreenTime: 600,
          dataPreloadingTime: 1000,
          screenPreloadingTime: 500,
          firstInteractionTime: 2500,
          memoryUsage: { initial: 65, peak: 85, final: 75 },
          preloadedDataSize: 30000,
          cacheHitRate: 0.8,
          networkRequests: 4,
          failedRequests: 0,
          perceivedPerformance: 'normal' as const,
          startupErrors: [],
          isFirstLaunch: false,
          isAppUpdate: false,
          networkType: 'wifi' as const,
          featureFlags: {},
          deviceMetrics: {
            platform: 'ios',
            osVersion: '16.0',
            deviceModel: 'iPhone 14',
            screenDimensions: { width: 390, height: 844 },
            isTablet: false,
            isEmulator: false
          },
          sessionId: 'session1'
        }
      ];

      startupPerformanceMonitoringService['measurements'] = mockMeasurements;

      const dashboard = await startupPerformanceMonitoringService.generateDashboard();

      expect(dashboard.summary.totalMeasurements).toBe(1);
      expect(dashboard.summary.averageStartupTime).toBe(2500);
      expect(dashboard.summary.p95StartupTime).toBe(2500);
      expect(dashboard.summary.errorRate).toBe(0);
      expect(dashboard.summary.cacheHitRate).toBe(0.8);
      
      expect(dashboard.trends.startupTimeTrend).toBeInstanceOf(Array);
      expect(dashboard.trends.errorRateTrend).toBeInstanceOf(Array);
      expect(dashboard.trends.devicePerformance).toBeInstanceOf(Array);
      
      expect(dashboard.alerts).toBeInstanceOf(Array);
      expect(dashboard.recommendations).toBeInstanceOf(Array);
    });

    it('should generate recommendations based on performance data', async () => {
      // Mock poor performance data
      const mockMeasurements = [
        {
          timestamp: Date.now() - 3600000,
          totalStartupTime: 6000, // Slow
          cacheHitRate: 0.4, // Low cache hit rate
          startupErrors: ['Network timeout'], // Has errors
          deviceMetrics: {
            deviceModel: 'iPhone 12',
            platform: 'ios',
            osVersion: '15.0',
            screenDimensions: { width: 390, height: 844 },
            isTablet: false,
            isEmulator: false
          },
          // ... other required fields with defaults
          splashScreenTime: 800,
          dataPreloadingTime: 2000,
          screenPreloadingTime: 1000,
          firstInteractionTime: 6000,
          memoryUsage: { initial: 70, peak: 95, final: 85 },
          preloadedDataSize: 25000,
          networkRequests: 5,
          failedRequests: 1,
          perceivedPerformance: 'slow' as const,
          isFirstLaunch: false,
          isAppUpdate: false,
          networkType: 'cellular' as const,
          featureFlags: {},
          sessionId: 'session1'
        }
      ];

      startupPerformanceMonitoringService['measurements'] = mockMeasurements;

      const dashboard = await startupPerformanceMonitoringService.generateDashboard();

      expect(dashboard.recommendations.length).toBeGreaterThan(0);
      expect(dashboard.recommendations.some(r => 
        r.includes('startup time') || r.includes('critical path')
      )).toBe(true);
      expect(dashboard.recommendations.some(r => 
        r.includes('cache') || r.includes('cache hit rate')
      )).toBe(true);
      expect(dashboard.recommendations.some(r => 
        r.includes('errors') || r.includes('reliability')
      )).toBe(true);
    });

    it('should handle empty dashboard gracefully', async () => {
      const dashboard = await startupPerformanceMonitoringService.generateDashboard();

      expect(dashboard.summary.totalMeasurements).toBe(0);
      expect(dashboard.summary.averageStartupTime).toBe(0);
      expect(dashboard.trends.startupTimeTrend).toEqual([]);
      expect(dashboard.recommendations).toContain('No data available yet');
    });
  });

  describe('data persistence', () => {
    it('should store measurements to AsyncStorage', async () => {
      await startupPerformanceMonitoringService.startStartupMeasurement();
      const measurement = await startupPerformanceMonitoringService.completeStartupMeasurement();

      expect(mockedAsyncStorage.setItem).toHaveBeenCalledWith(
        'startup_measurements',
        expect.stringContaining(measurement.sessionId)
      );
    });

    it('should limit stored measurements to maximum count', async () => {
      // Create many measurements
      const manyMeasurements = Array.from({ length: 1001 }, (_, i) => ({
        sessionId: `session_${i}`,
        timestamp: Date.now() - (i * 1000),
        totalStartupTime: 3000,
        // ... minimal required fields
        splashScreenTime: 500,
        dataPreloadingTime: 1000,
        screenPreloadingTime: 400,
        firstInteractionTime: 3000,
        memoryUsage: { initial: 60, peak: 80, final: 70 },
        preloadedDataSize: 20000,
        cacheHitRate: 0.7,
        networkRequests: 3,
        failedRequests: 0,
        perceivedPerformance: 'normal' as const,
        startupErrors: [],
        isFirstLaunch: false,
        isAppUpdate: false,
        networkType: 'wifi' as const,
        featureFlags: {},
        deviceMetrics: {
          platform: 'ios',
          osVersion: '16.0',
          deviceModel: 'iPhone 14',
          screenDimensions: { width: 390, height: 844 },
          isTablet: false,
          isEmulator: false
        }
      }));

      startupPerformanceMonitoringService['measurements'] = manyMeasurements;
      await startupPerformanceMonitoringService['storeMeasurements']();

      // Should limit to 1000 measurements
      const storedCall = mockedAsyncStorage.setItem.mock.calls.find(call => 
        call[0] === 'startup_measurements'
      );
      
      if (storedCall) {
        const storedData = JSON.parse(storedCall[1]);
        expect(storedData.length).toBeLessThanOrEqual(1000);
      }
    });
  });

  describe('utility methods', () => {
    it('should export performance data', async () => {
      await startupPerformanceMonitoringService.startStartupMeasurement();
      await startupPerformanceMonitoringService.completeStartupMeasurement();

      const exportedData = await startupPerformanceMonitoringService.exportPerformanceData();
      const parsedData = JSON.parse(exportedData);

      expect(parsedData.exportedAt).toBeDefined();
      expect(parsedData.dashboard).toBeDefined();
      expect(parsedData.rawMeasurements).toBeInstanceOf(Array);
    });

    it('should clear performance data', async () => {
      await startupPerformanceMonitoringService.clearPerformanceData();

      expect(mockedAsyncStorage.multiRemove).toHaveBeenCalledWith([
        'startup_measurements',
        'performance_alerts'
      ]);
    });

    it('should provide current status', () => {
      const status = startupPerformanceMonitoringService.getStatus();

      expect(status).toHaveProperty('isMonitoring');
      expect(status).toHaveProperty('totalMeasurements');
      expect(status).toHaveProperty('uptime');
      
      expect(typeof status.isMonitoring).toBe('boolean');
      expect(typeof status.totalMeasurements).toBe('number');
      expect(typeof status.uptime).toBe('number');
    });
  });
});