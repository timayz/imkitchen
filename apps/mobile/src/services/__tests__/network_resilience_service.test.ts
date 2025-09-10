/**
 * Network Resilience Service Tests
 * 
 * Comprehensive test suite for the network resilience service covering:
 * - Retry mechanisms with exponential backoff
 * - Circuit breaker functionality
 * - Offline mode detection and queue persistence
 * - Network condition monitoring and adaptation
 * - Data integrity verification
 * - Error handling and recovery scenarios
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import {
  networkResilienceService,
  NetworkResilienceService,
  ConnectionQuality,
  CircuitState,
  RetryConfig,
} from '../network_resilience_service';

// Mock AsyncStorage
jest.mock('@react-native-async-storage/async-storage');
const mockAsyncStorage = AsyncStorage as jest.Mocked<typeof AsyncStorage>;

// Mock NetInfo
jest.mock('@react-native-community/netinfo', () => ({
  addEventListener: jest.fn(() => ({ unsubscribe: jest.fn() })),
  fetch: jest.fn(() => Promise.resolve({
    isConnected: true,
    type: 'wifi',
    isInternetReachable: true,
    details: { strength: 4, linkSpeed: 300 }
  }))
}));

// Mock React Native components
jest.mock('react-native', () => ({
  AppState: {
    addEventListener: jest.fn(),
    removeEventListener: jest.fn()
  }
}));

describe('NetworkResilienceService', () => {
  let service: NetworkResilienceService;

  beforeEach(() => {
    jest.clearAllMocks();
    mockAsyncStorage.getItem.mockResolvedValue(null);
    mockAsyncStorage.setItem.mockResolvedValue(undefined);
    service = new NetworkResilienceService();
  });

  afterEach(() => {
    service.destroy();
  });

  describe('Retry Mechanisms', () => {
    test('should retry failed operations with exponential backoff', async () => {
      const mockOperation = jest.fn()
        .mockRejectedValueOnce(new Error('NetworkError'))
        .mockRejectedValueOnce(new Error('TimeoutError'))
        .mockResolvedValueOnce('success');

      const config: Partial<RetryConfig> = {
        maxAttempts: 3,
        baseDelayMs: 100,
        backoffFactor: 2.0
      };

      const result = await service.executeWithResilience(
        'test-operation',
        mockOperation,
        config
      );

      expect(result).toBe('success');
      expect(mockOperation).toHaveBeenCalledTimes(3);
    }, 10000);

    test('should calculate correct retry delays with jitter', async () => {
      const delays: number[] = [];
      const originalSetTimeout = global.setTimeout;
      
      global.setTimeout = jest.fn((callback: Function, delay: number) => {
        delays.push(delay);
        return originalSetTimeout(callback, 0);
      }) as any;

      const mockOperation = jest.fn()
        .mockRejectedValueOnce(new Error('NetworkError'))
        .mockRejectedValueOnce(new Error('NetworkError'))
        .mockResolvedValueOnce('success');

      const config: Partial<RetryConfig> = {
        maxAttempts: 3,
        baseDelayMs: 1000,
        backoffFactor: 2.0,
        jitterFactor: 0.1
      };

      await service.executeWithResilience('test-operation', mockOperation, config);

      // First retry should be around 1000ms (base delay)
      expect(delays[0]).toBeGreaterThanOrEqual(900);
      expect(delays[0]).toBeLessThanOrEqual(1100);

      // Second retry should be around 2000ms (exponential backoff)
      expect(delays[1]).toBeGreaterThanOrEqual(1800);
      expect(delays[1]).toBeLessThanOrEqual(2200);

      global.setTimeout = originalSetTimeout;
    }, 10000);

    test('should stop retrying non-retryable errors', async () => {
      const mockOperation = jest.fn()
        .mockRejectedValue(new Error('AuthenticationError'));

      const config: Partial<RetryConfig> = {
        maxAttempts: 3,
        retryableErrors: ['NetworkError', 'TimeoutError']
      };

      await expect(
        service.executeWithResilience('test-operation', mockOperation, config)
      ).rejects.toThrow('AuthenticationError');

      expect(mockOperation).toHaveBeenCalledTimes(1);
    });

    test('should respect maximum delay limit', async () => {
      const delays: number[] = [];
      const originalSetTimeout = global.setTimeout;
      
      global.setTimeout = jest.fn((callback: Function, delay: number) => {
        delays.push(delay);
        return originalSetTimeout(callback, 0);
      }) as any;

      const mockOperation = jest.fn()
        .mockRejectedValueOnce(new Error('NetworkError'))
        .mockRejectedValueOnce(new Error('NetworkError'))
        .mockResolvedValueOnce('success');

      const config: Partial<RetryConfig> = {
        maxAttempts: 3,
        baseDelayMs: 1000,
        maxDelayMs: 1500,
        backoffFactor: 3.0
      };

      await service.executeWithResilience('test-operation', mockOperation, config);

      // All delays should be capped at maxDelayMs
      delays.forEach(delay => {
        expect(delay).toBeLessThanOrEqual(1500);
      });

      global.setTimeout = originalSetTimeout;
    });
  });

  describe('Circuit Breaker', () => {
    test('should open circuit after threshold failures', async () => {
      const mockOperation = jest.fn()
        .mockRejectedValue(new Error('NetworkError'));

      // Trigger enough failures to open the circuit
      for (let i = 0; i < 5; i++) {
        try {
          await service.executeWithResilience(
            'failing-service',
            mockOperation,
            { maxAttempts: 1 }
          );
        } catch (error) {
          // Expected to fail
        }
      }

      // Circuit should be open now
      await expect(
        service.executeWithResilience(
          'failing-service',
          mockOperation,
          { maxAttempts: 1 }
        )
      ).rejects.toThrow('Circuit breaker is open');
    });

    test('should transition to half-open after reset timeout', async (done) => {
      const mockOperation = jest.fn()
        .mockRejectedValue(new Error('NetworkError'));

      // Open the circuit
      for (let i = 0; i < 5; i++) {
        try {
          await service.executeWithResilience(
            'test-service',
            mockOperation,
            { maxAttempts: 1 }
          );
        } catch (error) {
          // Expected
        }
      }

      // Mock time passing for circuit reset
      setTimeout(async () => {
        const successOperation = jest.fn().mockResolvedValue('success');
        
        const result = await service.executeWithResilience(
          'test-service',
          successOperation,
          { maxAttempts: 1 }
        );

        expect(result).toBe('success');
        done();
      }, 100); // Shorter timeout for testing
    }, 5000);

    test('should close circuit after successful half-open operations', async () => {
      // First, open the circuit
      const failingOperation = jest.fn().mockRejectedValue(new Error('NetworkError'));
      
      for (let i = 0; i < 5; i++) {
        try {
          await service.executeWithResilience(
            'recovery-service',
            failingOperation,
            { maxAttempts: 1 }
          );
        } catch (error) {
          // Expected
        }
      }

      // Wait for reset timeout (mocked shorter)
      await new Promise(resolve => setTimeout(resolve, 100));

      // Now test recovery
      const successOperation = jest.fn().mockResolvedValue('recovered');
      
      const result = await service.executeWithResilience(
        'recovery-service',
        successOperation,
        { maxAttempts: 1 }
      );

      expect(result).toBe('recovered');

      // Circuit should be closed, allowing normal operations
      const normalOperation = jest.fn().mockResolvedValue('normal');
      const normalResult = await service.executeWithResilience(
        'recovery-service',
        normalOperation,
        { maxAttempts: 1 }
      );

      expect(normalResult).toBe('normal');
    });
  });

  describe('Offline Mode and Queue Persistence', () => {
    test('should queue operations when offline', async () => {
      // Mock offline state
      (service as any).isOfflineMode = true;
      (service as any).networkState.isConnected = false;

      const mockOperation = jest.fn().mockResolvedValue('queued-result');

      const result = service.executeWithResilience(
        'offline-operation',
        mockOperation,
        { maxAttempts: 3 }
      );

      await expect(result).rejects.toThrow('operation queued due to network conditions');
      
      // Check that operation was queued
      const queueStatus = service.getQueueStatus();
      expect(queueStatus.size).toBe(1);
    });

    test('should persist queue to AsyncStorage', async () => {
      const mockOperation = jest.fn().mockResolvedValue('test');
      
      // Mock offline state
      (service as any).isOfflineMode = true;
      (service as any).networkState.isConnected = false;

      try {
        await service.executeWithResilience(
          'persist-operation',
          mockOperation,
          { maxAttempts: 1 }
        );
      } catch (error) {
        // Expected to be queued
      }

      expect(mockAsyncStorage.setItem).toHaveBeenCalledWith(
        'network_resilience_queue',
        expect.any(String)
      );
    });

    test('should load persisted queue on initialization', async () => {
      const persistedQueue = JSON.stringify([
        {
          id: 'persisted-op',
          metadata: {
            type: 'queued_operation',
            priority: 1,
            createdAt: new Date().toISOString(),
            attempts: 0
          },
          retryConfig: {
            maxAttempts: 3,
            baseDelayMs: 1000,
            maxDelayMs: 30000,
            backoffFactor: 2.0,
            jitterFactor: 0.1,
            timeoutMs: 10000,
            retryableErrors: ['NetworkError']
          }
        }
      ]);

      mockAsyncStorage.getItem.mockResolvedValue(persistedQueue);

      const newService = new NetworkResilienceService();
      
      // Allow initialization to complete
      await new Promise(resolve => setTimeout(resolve, 100));

      expect(mockAsyncStorage.getItem).toHaveBeenCalledWith('network_resilience_queue');
    });

    test('should process queue when network is restored', async (done) => {
      const mockOperation = jest.fn().mockResolvedValue('processed');
      
      // Start offline
      (service as any).isOfflineMode = true;
      (service as any).networkState.isConnected = false;

      try {
        await service.executeWithResilience(
          'restore-operation',
          mockOperation,
          { maxAttempts: 1 }
        );
      } catch (error) {
        // Expected to be queued
      }

      // Simulate network restoration
      (service as any).isOfflineMode = false;
      (service as any).networkState.isConnected = true;
      (service as any).handleNetworkRestore();

      // Give queue processing time to run
      setTimeout(() => {
        const queueStatus = service.getQueueStatus();
        expect(queueStatus.size).toBeLessThanOrEqual(1); // Should be processing
        done();
      }, 100);
    });
  });

  describe('Network Condition Monitoring', () => {
    test('should evaluate connection quality correctly', async () => {
      const service = new NetworkResilienceService();
      
      // Test excellent quality
      const excellentState = {
        isConnected: true,
        type: 'wifi',
        details: { strength: 4, linkSpeed: 300 }
      };
      
      const excellentQuality = (service as any).evaluateConnectionQuality(excellentState);
      expect(excellentQuality).toBe(ConnectionQuality.EXCELLENT);

      // Test poor quality
      const poorState = {
        isConnected: true,
        type: 'cellular',
        details: { strength: 1, linkSpeed: 2 }
      };
      
      const poorQuality = (service as any).evaluateConnectionQuality(poorState);
      expect(poorQuality).toBe(ConnectionQuality.POOR);

      // Test offline
      const offlineState = {
        isConnected: false,
        type: 'none'
      };
      
      const offlineQuality = (service as any).evaluateConnectionQuality(offlineState);
      expect(offlineQuality).toBe(ConnectionQuality.OFFLINE);
    });

    test('should adapt retry delays based on network quality', async () => {
      const delays: number[] = [];
      const originalSetTimeout = global.setTimeout;
      
      global.setTimeout = jest.fn((callback: Function, delay: number) => {
        delays.push(delay);
        return originalSetTimeout(callback, 0);
      }) as any;

      // Set poor network quality
      (service as any).networkState.connectionQuality = ConnectionQuality.POOR;

      const mockOperation = jest.fn()
        .mockRejectedValueOnce(new Error('NetworkError'))
        .mockResolvedValueOnce('success');

      await service.executeWithResilience(
        'adaptive-operation',
        mockOperation,
        { maxAttempts: 2, baseDelayMs: 1000 }
      );

      // Delay should be increased for poor network quality
      expect(delays[0]).toBeGreaterThan(1000);

      global.setTimeout = originalSetTimeout;
    });
  });

  describe('Data Integrity', () => {
    test('should verify data integrity with hash comparison', async () => {
      const testData = { id: 1, name: 'test' };
      const correctHash = await (service as any).calculateHash(testData);

      const isValid = await service.verifyDataIntegrity(testData, correctHash);
      expect(isValid).toBe(true);

      const invalidHash = 'wrong-hash';
      const isInvalid = await service.verifyDataIntegrity(testData, invalidHash);
      expect(isInvalid).toBe(false);
    });

    test('should validate basic data structure', async () => {
      const validData = { key: 'value' };
      const isValid = await service.verifyDataIntegrity(validData);
      expect(isValid).toBe(true);

      const invalidData = null;
      const isInvalid = await service.verifyDataIntegrity(invalidData);
      expect(isInvalid).toBe(false);
    });
  });

  describe('Statistics and Monitoring', () => {
    test('should track operation statistics', async () => {
      const successOperation = jest.fn().mockResolvedValue('success');
      const failOperation = jest.fn().mockRejectedValue(new Error('TestError'));

      // Execute some operations
      await service.executeWithResilience('success-op', successOperation, { maxAttempts: 1 });
      
      try {
        await service.executeWithResilience('fail-op', failOperation, { 
          maxAttempts: 1, 
          retryableErrors: [] 
        });
      } catch (error) {
        // Expected
      }

      const stats = service.getStats();
      expect(stats.totalOperations).toBeGreaterThan(0);
      expect(stats.successfulOperations).toBeGreaterThan(0);
      expect(stats.failedOperations).toBeGreaterThan(0);
    });

    test('should provide accurate queue status', async () => {
      // Mock offline to queue operations
      (service as any).isOfflineMode = true;
      (service as any).networkState.isConnected = false;

      const mockOperation = jest.fn().mockResolvedValue('test');

      try {
        await service.executeWithResilience('queue-status-1', mockOperation);
        await service.executeWithResilience('queue-status-2', mockOperation);
      } catch (error) {
        // Expected to be queued
      }

      const queueStatus = service.getQueueStatus();
      expect(queueStatus.size).toBe(2);
      expect(queueStatus.failedOperations).toBe(0);
      expect(queueStatus.oldestOperation).toBeDefined();
    });
  });

  describe('Configuration Management', () => {
    test('should update retry configuration', () => {
      const newConfig: Partial<RetryConfig> = {
        maxAttempts: 10,
        baseDelayMs: 2000,
        backoffFactor: 1.5
      };

      service.updateRetryConfig(newConfig);

      // Verify configuration was updated by checking internal state
      const internalConfig = (service as any).defaultRetryConfig;
      expect(internalConfig.maxAttempts).toBe(10);
      expect(internalConfig.baseDelayMs).toBe(2000);
      expect(internalConfig.backoffFactor).toBe(1.5);
    });

    test('should clear operation queue', async () => {
      // Add operations to queue
      (service as any).isOfflineMode = true;
      (service as any).networkState.isConnected = false;

      const mockOperation = jest.fn().mockResolvedValue('test');

      try {
        await service.executeWithResilience('clear-test', mockOperation);
      } catch (error) {
        // Expected
      }

      expect(service.getQueueStatus().size).toBe(1);

      await service.clearQueue();

      expect(service.getQueueStatus().size).toBe(0);
      expect(mockAsyncStorage.removeItem).toHaveBeenCalledWith('network_resilience_queue');
    });
  });

  describe('Error Handling Edge Cases', () => {
    test('should handle timeout errors correctly', async () => {
      const slowOperation = jest.fn(() => new Promise(resolve => setTimeout(resolve, 15000)));

      const config: Partial<RetryConfig> = {
        maxAttempts: 1,
        timeoutMs: 100
      };

      await expect(
        service.executeWithResilience('timeout-test', slowOperation, config)
      ).rejects.toThrow();
    });

    test('should handle AsyncStorage failures gracefully', async () => {
      mockAsyncStorage.setItem.mockRejectedValue(new Error('Storage failed'));

      const mockOperation = jest.fn().mockResolvedValue('test');
      
      (service as any).isOfflineMode = true;
      (service as any).networkState.isConnected = false;

      // Should not throw even if storage fails
      await expect(
        service.executeWithResilience('storage-fail-test', mockOperation)
      ).rejects.toThrow('operation queued due to network conditions');
    });

    test('should handle malformed persisted data', async () => {
      mockAsyncStorage.getItem.mockResolvedValue('invalid-json');

      // Should not crash on invalid persisted data
      const newService = new NetworkResilienceService();
      await new Promise(resolve => setTimeout(resolve, 100));

      expect(newService.getQueueStatus().size).toBe(0);
    });
  });

  describe('Performance and Resource Management', () => {
    test('should limit concurrent operations', async () => {
      const concurrentOperations: Promise<any>[] = [];
      let activeCount = 0;
      let maxConcurrent = 0;

      const mockOperation = jest.fn(() => {
        activeCount++;
        maxConcurrent = Math.max(maxConcurrent, activeCount);
        
        return new Promise(resolve => {
          setTimeout(() => {
            activeCount--;
            resolve('completed');
          }, 100);
        });
      });

      // Start many operations simultaneously
      for (let i = 0; i < 10; i++) {
        concurrentOperations.push(
          service.executeWithResilience(`concurrent-${i}`, mockOperation)
        );
      }

      await Promise.all(concurrentOperations);

      // Should respect concurrency limits (default is usually 3-5)
      expect(maxConcurrent).toBeLessThanOrEqual(5);
    });

    test('should cleanup resources on destroy', () => {
      const clearIntervalSpy = jest.spyOn(global, 'clearInterval');
      
      service.destroy();

      expect(clearIntervalSpy).toHaveBeenCalled();
      clearIntervalSpy.mockRestore();
    });
  });
});