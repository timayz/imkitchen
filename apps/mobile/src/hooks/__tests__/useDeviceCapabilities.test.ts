import { renderHook, act } from '@testing-library/react-hooks';
import { useDeviceCapabilities } from '../useDeviceCapabilities';
import { Dimensions } from 'react-native';
import NetInfo from '@react-native-community/netinfo';
import * as Device from 'expo-device';

// Mock dependencies
jest.mock('react-native', () => ({
  Dimensions: {
    get: jest.fn(),
    addEventListener: jest.fn(),
  },
  Platform: {
    OS: 'ios',
  },
}));

jest.mock('@react-native-community/netinfo', () => ({
  addEventListener: jest.fn(),
}));

jest.mock('expo-device', () => ({
  deviceYearClass: 2020,
  totalMemory: 4 * 1024 * 1024 * 1024, // 4GB
}));

describe('useDeviceCapabilities', () => {
  const mockDimensions = {
    window: {
      width: 375,
      height: 812,
      scale: 2,
    },
  };

  beforeEach(() => {
    jest.clearAllMocks();
    
    (Dimensions.get as jest.Mock).mockReturnValue(mockDimensions.window);
    (Dimensions.addEventListener as jest.Mock).mockReturnValue({
      remove: jest.fn(),
    });
    (NetInfo.addEventListener as jest.Mock).mockReturnValue(jest.fn());
  });

  describe('device detection', () => {
    it('should detect phone device correctly', () => {
      (Dimensions.get as jest.Mock).mockReturnValue({
        width: 375,
        height: 812,
        scale: 2,
      });

      const { result } = renderHook(() => useDeviceCapabilities());

      expect(result.current.capabilities.isTablet).toBe(false);
      expect(result.current.capabilities.screenWidth).toBe(375);
      expect(result.current.capabilities.screenHeight).toBe(812);
    });

    it('should detect tablet device correctly', () => {
      (Dimensions.get as jest.Mock).mockReturnValue({
        width: 768,
        height: 1024,
        scale: 2,
      });

      const { result } = renderHook(() => useDeviceCapabilities());

      expect(result.current.capabilities.isTablet).toBe(true);
    });

    it('should detect low-end device based on year and memory', () => {
      (Device as any).deviceYearClass = 2016;
      (Device as any).totalMemory = 2 * 1024 * 1024 * 1024; // 2GB

      const { result } = renderHook(() => useDeviceCapabilities());

      expect(result.current.isLowEndDevice()).toBe(true);
    });

    it('should detect high-end device', () => {
      (Device as any).deviceYearClass = 2022;
      (Device as any).totalMemory = 8 * 1024 * 1024 * 1024; // 8GB

      const { result } = renderHook(() => useDeviceCapabilities());

      expect(result.current.isLowEndDevice()).toBe(false);
      expect(result.current.capabilities.memoryLevel).toBe('high');
    });
  });

  describe('network detection', () => {
    it('should detect slow connection', () => {
      const { result } = renderHook(() => useDeviceCapabilities());

      // Simulate NetInfo callback
      const netInfoCallback = (NetInfo.addEventListener as jest.Mock).mock.calls[0][0];
      
      act(() => {
        netInfoCallback({
          isConnected: true,
          type: 'cellular',
          details: {
            effectiveType: '2g',
            downlink: 0.5,
          },
        });
      });

      expect(result.current.hasSlowConnection()).toBe(true);
    });

    it('should detect fast connection', () => {
      const { result } = renderHook(() => useDeviceCapabilities());

      const netInfoCallback = (NetInfo.addEventListener as jest.Mock).mock.calls[0][0];
      
      act(() => {
        netInfoCallback({
          isConnected: true,
          type: 'wifi',
          details: {
            effectiveType: '4g',
            downlink: 10,
          },
        });
      });

      expect(result.current.hasSlowConnection()).toBe(false);
    });

    it('should handle disconnected state', () => {
      const { result } = renderHook(() => useDeviceCapabilities());

      const netInfoCallback = (NetInfo.addEventListener as jest.Mock).mock.calls[0][0];
      
      act(() => {
        netInfoCallback({
          isConnected: false,
          type: 'none',
        });
      });

      expect(result.current.hasSlowConnection()).toBe(true);
    });
  });

  describe('image size optimization', () => {
    it('should recommend appropriate image size for small requests', () => {
      const { result } = renderHook(() => useDeviceCapabilities());

      const size = result.current.getOptimalImageSize(100, 100);
      expect(size).toBe('thumbnail');
    });

    it('should recommend appropriate image size for large requests', () => {
      const { result } = renderHook(() => useDeviceCapabilities());

      const size = result.current.getOptimalImageSize(800, 600);
      expect(size).toBe('large');
    });

    it('should cap image size for low-end devices', () => {
      (Device as any).deviceYearClass = 2016;
      
      const { result } = renderHook(() => useDeviceCapabilities());

      const size = result.current.getOptimalImageSize(800, 600);
      expect(size).toBe('medium'); // Capped at medium for low-end devices
    });

    it('should account for pixel density', () => {
      (Dimensions.get as jest.Mock).mockReturnValue({
        width: 375,
        height: 812,
        scale: 3, // High DPI
      });

      const { result } = renderHook(() => useDeviceCapabilities());

      // 200px at 3x scale = 600px device pixels
      const size = result.current.getOptimalImageSize(200, 200);
      expect(size).toBe('medium');
    });
  });

  describe('compression settings', () => {
    it('should provide appropriate compression for different image sizes', () => {
      const { result } = renderHook(() => useDeviceCapabilities());

      const thumbnailSettings = result.current.getOptimalCompressionSettings('thumbnail');
      expect(thumbnailSettings).toEqual({
        width: 150,
        height: 150,
        quality: 0.6,
        format: 'jpeg',
      });

      const largeSettings = result.current.getOptimalCompressionSettings('large');
      expect(largeSettings).toEqual({
        width: 1200,
        height: 1200,
        quality: 0.8,
        format: 'webp',
      });
    });

    it('should adjust compression for low-end devices', () => {
      (Device as any).deviceYearClass = 2016;
      
      const { result } = renderHook(() => useDeviceCapabilities());

      const settings = result.current.getOptimalCompressionSettings('large');
      expect(settings.quality).toBeLessThan(0.8);
      expect(settings.format).toBe('jpeg'); // Fallback to JPEG for compatibility
    });
  });

  describe('performance recommendations', () => {
    it('should disable preloading for low-end devices', () => {
      (Device as any).deviceYearClass = 2016;
      
      const { result } = renderHook(() => useDeviceCapabilities());

      expect(result.current.shouldPreloadImages()).toBe(false);
    });

    it('should enable preloading for high-end devices', () => {
      (Device as any).deviceYearClass = 2022;
      (Device as any).totalMemory = 8 * 1024 * 1024 * 1024;
      
      const { result } = renderHook(() => useDeviceCapabilities());

      expect(result.current.shouldPreloadImages()).toBe(true);
    });

    it('should provide appropriate cache size limits', () => {
      const { result } = renderHook(() => useDeviceCapabilities());

      const cacheSize = result.current.getMaxCacheSize();
      expect(cacheSize).toBeGreaterThan(0);
      expect(typeof cacheSize).toBe('number');
    });

    it('should provide appropriate concurrent load limits', () => {
      const { result } = renderHook(() => useDeviceCapabilities());

      const concurrentLoads = result.current.getConcurrentImageLoads();
      expect(concurrentLoads).toBeGreaterThan(0);
      expect(concurrentLoads).toBeLessThan(10); // Reasonable upper bound
    });

    it('should reduce concurrent loads for low-end devices', () => {
      (Device as any).deviceYearClass = 2016;
      (Device as any).totalMemory = 2 * 1024 * 1024 * 1024;
      
      const { result } = renderHook(() => useDeviceCapabilities());

      const concurrentLoads = result.current.getConcurrentImageLoads();
      expect(concurrentLoads).toBeLessThanOrEqual(2);
    });
  });

  describe('memory level detection', () => {
    it('should detect low memory devices', () => {
      (Device as any).totalMemory = 2 * 1024 * 1024 * 1024; // 2GB
      
      const { result } = renderHook(() => useDeviceCapabilities());

      expect(result.current.capabilities.memoryLevel).toBe('low');
    });

    it('should detect medium memory devices', () => {
      (Device as any).totalMemory = 4 * 1024 * 1024 * 1024; // 4GB
      
      const { result } = renderHook(() => useDeviceCapabilities());

      expect(result.current.capabilities.memoryLevel).toBe('medium');
    });

    it('should detect high memory devices', () => {
      (Device as any).totalMemory = 8 * 1024 * 1024 * 1024; // 8GB
      
      const { result } = renderHook(() => useDeviceCapabilities());

      expect(result.current.capabilities.memoryLevel).toBe('high');
    });
  });

  describe('dimension changes', () => {
    it('should update capabilities on orientation change', () => {
      const { result } = renderHook(() => useDeviceCapabilities());

      const dimensionCallback = (Dimensions.addEventListener as jest.Mock).mock.calls[0][1];
      
      act(() => {
        dimensionCallback({
          window: {
            width: 812,
            height: 375,
            scale: 2,
          },
        });
      });

      expect(result.current.capabilities.screenWidth).toBe(812);
      expect(result.current.capabilities.screenHeight).toBe(375);
    });
  });

  describe('error handling', () => {
    it('should handle device detection errors gracefully', () => {
      (Device as any).deviceYearClass = undefined;
      (Device as any).totalMemory = undefined;
      
      const { result } = renderHook(() => useDeviceCapabilities());

      expect(result.current.isLowEndDevice()).toBe(false); // Default to false
      expect(result.current.capabilities.memoryLevel).toBe('medium'); // Default fallback
    });

    it('should handle NetInfo errors gracefully', () => {
      const { result } = renderHook(() => useDeviceCapabilities());

      const netInfoCallback = (NetInfo.addEventListener as jest.Mock).mock.calls[0][0];
      
      act(() => {
        netInfoCallback(null); // Simulate error state
      });

      // Should not crash and maintain reasonable defaults
      expect(result.current.hasSlowConnection()).toBeDefined();
    });
  });
});