import { useCallback, useEffect, useState } from 'react';
import { imageCache, CachePriority, CompressionOptions } from '../services/image_cache_service';
import { useDeviceCapabilities } from './useDeviceCapabilities';

export interface ImageCacheState {
  isLoading: boolean;
  isCached: boolean;
  error: string | null;
  cacheSize: number;
  cacheCount: number;
}

export const useImageCache = () => {
  const [cacheState, setCacheState] = useState<ImageCacheState>({
    isLoading: false,
    isCached: false,
    error: null,
    cacheSize: 0,
    cacheCount: 0,
  });

  const { getOptimalCompressionSettings, getOptimalImageSize } = useDeviceCapabilities();

  // Load cache information on mount
  useEffect(() => {
    loadCacheInfo();
  }, []);

  const loadCacheInfo = useCallback(async () => {
    try {
      const info = await imageCache.getCacheInfo();
      setCacheState(prev => ({
        ...prev,
        cacheSize: info.totalSize,
        cacheCount: info.imageCount,
      }));
    } catch (error) {
      console.warn('Failed to load cache info:', error);
    }
  }, []);

  const getCachedImageUri = useCallback(async (uri: string): Promise<string | null> => {
    try {
      setCacheState(prev => ({ ...prev, isLoading: true, error: null }));
      
      const cachedUri = await imageCache.getCachedImageUri(uri);
      
      setCacheState(prev => ({
        ...prev,
        isLoading: false,
        isCached: cachedUri !== null,
      }));

      return cachedUri;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      setCacheState(prev => ({
        ...prev,
        isLoading: false,
        error: errorMessage,
      }));
      return null;
    }
  }, []);

  const cacheImage = useCallback(async (
    uri: string,
    priority: CachePriority = 'normal',
    customCompression?: CompressionOptions
  ): Promise<string> => {
    try {
      setCacheState(prev => ({ ...prev, isLoading: true, error: null }));

      // Use device-optimized compression if none provided
      let compressionOptions = customCompression;
      if (!compressionOptions) {
        const optimalSize = getOptimalImageSize(600, 600); // Default size
        compressionOptions = getOptimalCompressionSettings(optimalSize);
      }

      const cachedUri = await imageCache.cacheImage(uri, priority, compressionOptions);
      
      setCacheState(prev => ({
        ...prev,
        isLoading: false,
        isCached: true,
      }));

      // Update cache info
      await loadCacheInfo();

      return cachedUri;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      setCacheState(prev => ({
        ...prev,
        isLoading: false,
        error: errorMessage,
      }));
      throw error;
    }
  }, [getOptimalImageSize, getOptimalCompressionSettings, loadCacheInfo]);

  const deleteCachedImage = useCallback(async (uri: string): Promise<void> => {
    try {
      await imageCache.deleteCachedImage(uri);
      await loadCacheInfo();
    } catch (error) {
      console.warn('Failed to delete cached image:', error);
    }
  }, [loadCacheInfo]);

  const clearCache = useCallback(async (): Promise<void> => {
    try {
      setCacheState(prev => ({ ...prev, isLoading: true, error: null }));
      
      await imageCache.clearCache();
      
      setCacheState(prev => ({
        ...prev,
        isLoading: false,
        cacheSize: 0,
        cacheCount: 0,
      }));
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      setCacheState(prev => ({
        ...prev,
        isLoading: false,
        error: errorMessage,
      }));
    }
  }, []);

  const preloadImages = useCallback(async (
    uris: string[],
    priority: CachePriority = 'low'
  ): Promise<void> => {
    try {
      await imageCache.preloadImages(uris, priority);
      await loadCacheInfo();
    } catch (error) {
      console.warn('Failed to preload images:', error);
    }
  }, [loadCacheInfo]);

  const getCacheInfo = useCallback(async () => {
    return await imageCache.getCacheInfo();
  }, []);

  // Utility function to format cache size
  const formatCacheSize = useCallback((bytes: number): string => {
    if (bytes === 0) return '0 B';
    
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }, []);

  return {
    cacheState,
    getCachedImageUri,
    cacheImage,
    deleteCachedImage,
    clearCache,
    preloadImages,
    getCacheInfo,
    formatCacheSize,
    refreshCacheInfo: loadCacheInfo,
  };
};

// Hook for batch image operations
export const useImageBatchCache = () => {
  const { cacheImage, preloadImages } = useImageCache();
  const [batchState, setBatchState] = useState({
    isProcessing: false,
    progress: 0,
    completed: 0,
    total: 0,
    errors: [] as string[],
  });

  const cacheImageBatch = useCallback(async (
    uris: string[],
    priority: CachePriority = 'normal',
    onProgress?: (progress: number, completed: number, total: number) => void
  ): Promise<{ succeeded: string[]; failed: string[] }> => {
    if (uris.length === 0) return { succeeded: [], failed: [] };

    setBatchState({
      isProcessing: true,
      progress: 0,
      completed: 0,
      total: uris.length,
      errors: [],
    });

    const succeeded: string[] = [];
    const failed: string[] = [];
    const errors: string[] = [];

    for (let i = 0; i < uris.length; i++) {
      try {
        await cacheImage(uris[i], priority);
        succeeded.push(uris[i]);
      } catch (error) {
        failed.push(uris[i]);
        errors.push(`${uris[i]}: ${error}`);
      }

      const completed = i + 1;
      const progress = (completed / uris.length) * 100;

      setBatchState(prev => ({
        ...prev,
        progress,
        completed,
        errors,
      }));

      onProgress?.(progress, completed, uris.length);
    }

    setBatchState(prev => ({
      ...prev,
      isProcessing: false,
    }));

    return { succeeded, failed };
  }, [cacheImage]);

  const preloadImageBatch = useCallback(async (
    uris: string[],
    priority: CachePriority = 'low',
    onProgress?: (progress: number) => void
  ): Promise<void> => {
    setBatchState(prev => ({
      ...prev,
      isProcessing: true,
      total: uris.length,
    }));

    try {
      await preloadImages(uris, priority);
      
      setBatchState(prev => ({
        ...prev,
        isProcessing: false,
        progress: 100,
        completed: uris.length,
      }));

      onProgress?.(100);
    } catch (error) {
      setBatchState(prev => ({
        ...prev,
        isProcessing: false,
        errors: [`Batch preload failed: ${error}`],
      }));
    }
  }, [preloadImages]);

  return {
    batchState,
    cacheImageBatch,
    preloadImageBatch,
  };
};