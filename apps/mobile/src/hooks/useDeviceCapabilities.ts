import { useState, useEffect } from 'react';
import { Dimensions, Platform } from 'react-native';
import NetInfo from '@react-native-community/netinfo';
import * as Device from 'expo-device';

export interface DeviceCapabilities {
  screenWidth: number;
  screenHeight: number;
  pixelRatio: number;
  isTablet: boolean;
  isLowEndDevice: boolean;
  hasSlowConnection: boolean;
  connectionType: string;
  memoryLevel: 'low' | 'medium' | 'high';
  storageLevel: 'low' | 'medium' | 'high';
}

export interface ImageSizeRecommendation {
  width: number;
  height: number;
  quality: number;
  format: 'jpeg' | 'webp' | 'png';
}

export const useDeviceCapabilities = () => {
  const [capabilities, setCapabilities] = useState<DeviceCapabilities>({
    screenWidth: Dimensions.get('window').width,
    screenHeight: Dimensions.get('window').height,
    pixelRatio: Dimensions.get('window').scale,
    isTablet: false,
    isLowEndDevice: false,
    hasSlowConnection: false,
    connectionType: 'unknown',
    memoryLevel: 'medium',
    storageLevel: 'medium',
  });

  useEffect(() => {
    const detectDeviceCapabilities = async () => {
      try {
        const { width, height, scale } = Dimensions.get('window');
        
        // Detect if device is tablet
        const isTablet = width >= 768 || height >= 768;
        
        // Detect if device is low-end based on various factors
        const isLowEndDevice = detectLowEndDevice();
        
        // Get memory level estimation
        const memoryLevel = estimateMemoryLevel();
        
        // Get storage level estimation  
        const storageLevel = estimateStorageLevel();

        setCapabilities(prev => ({
          ...prev,
          screenWidth: width,
          screenHeight: height,
          pixelRatio: scale,
          isTablet,
          isLowEndDevice,
          memoryLevel,
          storageLevel,
        }));
      } catch (error) {
        console.warn('Failed to detect device capabilities:', error);
      }
    };

    detectDeviceCapabilities();

    // Listen for dimension changes (orientation)
    const subscription = Dimensions.addEventListener('change', ({ window }) => {
      setCapabilities(prev => ({
        ...prev,
        screenWidth: window.width,
        screenHeight: window.height,
        pixelRatio: window.scale,
      }));
    });

    return () => subscription?.remove();
  }, []);

  useEffect(() => {
    // Monitor network conditions
    const unsubscribe = NetInfo.addEventListener(state => {
      const hasSlowConnection = detectSlowConnection(state);
      const connectionType = state.type || 'unknown';

      setCapabilities(prev => ({
        ...prev,
        hasSlowConnection,
        connectionType,
      }));
    });

    return unsubscribe;
  }, []);

  const detectLowEndDevice = (): boolean => {
    try {
      // On Android, we can estimate based on device year and total RAM
      if (Platform.OS === 'android') {
        const deviceYear = Device.deviceYearClass || 0;
        const totalMemory = Device.totalMemory || 0;
        
        // Devices older than 2018 or with less than 3GB RAM are considered low-end
        return deviceYear < 2018 || totalMemory < 3 * 1024 * 1024 * 1024;
      }
      
      // On iOS, estimate based on device model and year
      if (Platform.OS === 'ios') {
        const deviceYear = Device.deviceYearClass || 0;
        return deviceYear < 2017; // Devices older than iPhone 7/iPad 5th gen
      }
      
      return false;
    } catch (error) {
      console.warn('Failed to detect device performance level:', error);
      return false;
    }
  };

  const detectSlowConnection = (netInfo: any): boolean => {
    if (!netInfo.isConnected) return true;
    
    // Effective connection type detection
    const effectiveType = netInfo.details?.effectiveType;
    if (effectiveType && ['slow-2g', '2g'].includes(effectiveType)) {
      return true;
    }
    
    // Bandwidth-based detection
    const downlink = netInfo.details?.downlink;
    if (downlink && downlink < 1.5) { // Less than 1.5 Mbps
      return true;
    }
    
    return false;
  };

  const estimateMemoryLevel = (): 'low' | 'medium' | 'high' => {
    try {
      const totalMemory = Device.totalMemory || 0;
      const memoryGB = totalMemory / (1024 * 1024 * 1024);
      
      if (memoryGB < 3) return 'low';
      if (memoryGB < 6) return 'medium';
      return 'high';
    } catch (error) {
      return 'medium';
    }
  };

  const estimateStorageLevel = (): 'low' | 'medium' | 'high' => {
    // This is an estimation - actual storage detection requires native modules
    try {
      if (capabilities.isLowEndDevice) return 'low';
      if (capabilities.memoryLevel === 'low') return 'low';
      if (capabilities.memoryLevel === 'high') return 'high';
      return 'medium';
    } catch (error) {
      return 'medium';
    }
  };

  const getOptimalImageSize = (requestedWidth: number, requestedHeight: number): 'thumbnail' | 'small' | 'medium' | 'large' => {
    const { screenWidth, screenHeight, pixelRatio, isLowEndDevice, hasSlowConnection } = capabilities;
    
    // Account for pixel density
    const devicePixelWidth = requestedWidth * pixelRatio;
    const devicePixelHeight = requestedHeight * pixelRatio;
    
    // Reduce quality for low-end devices or slow connections
    if (isLowEndDevice || hasSlowConnection) {
      if (devicePixelWidth <= 150 || devicePixelHeight <= 150) return 'thumbnail';
      if (devicePixelWidth <= 300 || devicePixelHeight <= 300) return 'small';
      return 'medium'; // Cap at medium for performance
    }
    
    // Normal quality selection
    if (devicePixelWidth <= 150 || devicePixelHeight <= 150) return 'thumbnail';
    if (devicePixelWidth <= 300 || devicePixelHeight <= 300) return 'small';
    if (devicePixelWidth <= 600 || devicePixelHeight <= 600) return 'medium';
    return 'large';
  };

  const getOptimalCompressionSettings = (imageSize: 'thumbnail' | 'small' | 'medium' | 'large'): ImageSizeRecommendation => {
    const { isLowEndDevice, hasSlowConnection } = capabilities;
    
    const baseSettings = {
      thumbnail: { width: 150, height: 150, quality: 0.6, format: 'jpeg' as const },
      small: { width: 300, height: 300, quality: 0.7, format: 'jpeg' as const },
      medium: { width: 600, height: 600, quality: 0.75, format: 'webp' as const },
      large: { width: 1200, height: 1200, quality: 0.8, format: 'webp' as const },
    };
    
    const settings = baseSettings[imageSize];
    
    // Adjust for low-end devices
    if (isLowEndDevice || hasSlowConnection) {
      return {
        ...settings,
        quality: Math.max(0.5, settings.quality - 0.1), // Reduce quality by 10%
        format: 'jpeg', // Use JPEG for better compatibility
      };
    }
    
    return settings;
  };

  const shouldPreloadImages = (): boolean => {
    const { hasSlowConnection, isLowEndDevice, memoryLevel } = capabilities;
    
    // Don't preload on slow connections or low-end devices
    if (hasSlowConnection || isLowEndDevice || memoryLevel === 'low') {
      return false;
    }
    
    return true;
  };

  const getMaxCacheSize = (): number => {
    const { memoryLevel, storageLevel, isLowEndDevice } = capabilities;
    
    if (isLowEndDevice || memoryLevel === 'low' || storageLevel === 'low') {
      return 50 * 1024 * 1024; // 50MB
    }
    
    if (memoryLevel === 'medium' || storageLevel === 'medium') {
      return 100 * 1024 * 1024; // 100MB
    }
    
    return 200 * 1024 * 1024; // 200MB for high-end devices
  };

  const getConcurrentImageLoads = (): number => {
    const { isLowEndDevice, hasSlowConnection, memoryLevel } = capabilities;
    
    if (isLowEndDevice || hasSlowConnection || memoryLevel === 'low') {
      return 2; // Limited concurrent loads
    }
    
    if (memoryLevel === 'medium') {
      return 4; // Moderate concurrent loads
    }
    
    return 6; // High concurrent loads for powerful devices
  };

  return {
    capabilities,
    isLowEndDevice: () => capabilities.isLowEndDevice,
    hasSlowConnection: () => capabilities.hasSlowConnection,
    getOptimalImageSize,
    getOptimalCompressionSettings,
    shouldPreloadImages,
    getMaxCacheSize,
    getConcurrentImageLoads,
  };
};