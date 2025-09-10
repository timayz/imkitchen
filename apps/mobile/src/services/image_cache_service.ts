import AsyncStorage from '@react-native-async-storage/async-storage';
import { Platform } from 'react-native';
import * as FileSystem from 'expo-file-system';
import * as ImageManipulator from 'expo-image-manipulator';
import * as Crypto from 'expo-crypto';

export interface CachedImageInfo {
  uri: string;
  localPath: string;
  size: number;
  cachedAt: number;
  ttl: number;
  compressionLevel: number;
  priority: 'high' | 'normal' | 'low';
  accessCount: number;
  lastAccessed: number;
}

export interface ImageCacheMetadata {
  totalSize: number;
  maxSize: number;
  imageCount: number;
  lastCleanup: number;
  version: string;
}

export interface CompressionOptions {
  quality: number;
  width?: number;
  height?: number;
  format: 'jpeg' | 'webp' | 'png';
}

export type CachePriority = 'high' | 'normal' | 'low';

export class ImageCacheService {
  private static readonly CACHE_KEY_PREFIX = 'image_cache_';
  private static readonly METADATA_KEY = 'image_cache_metadata';
  private static readonly DEFAULT_MAX_SIZE = 100 * 1024 * 1024; // 100MB
  private static readonly DEFAULT_TTL = 7 * 24 * 60 * 60 * 1000; // 7 days
  private static readonly HIGH_PRIORITY_TTL = 30 * 24 * 60 * 60 * 1000; // 30 days
  private static readonly CLEANUP_THRESHOLD = 0.8; // Start cleanup at 80% capacity
  private static readonly VERSION = '1.0';

  private cacheDir: string;
  private metadata: ImageCacheMetadata;
  private compressionQueue: Map<string, Promise<string>> = new Map();

  constructor() {
    this.cacheDir = `${FileSystem.documentDirectory}image_cache/`;
    this.metadata = {
      totalSize: 0,
      maxSize: ImageCacheService.DEFAULT_MAX_SIZE,
      imageCount: 0,
      lastCleanup: Date.now(),
      version: ImageCacheService.VERSION,
    };
    this.initializeCache();
  }

  private async initializeCache(): Promise<void> {
    try {
      // Ensure cache directory exists
      const dirInfo = await FileSystem.getInfoAsync(this.cacheDir);
      if (!dirInfo.exists) {
        await FileSystem.makeDirectoryAsync(this.cacheDir, { intermediates: true });
      }

      // Load metadata
      await this.loadMetadata();
      
      // Perform cleanup if needed
      if (this.shouldCleanup()) {
        await this.performCleanup();
      }
    } catch (error) {
      console.error('Failed to initialize image cache:', error);
    }
  }

  private async loadMetadata(): Promise<void> {
    try {
      const metadataJson = await AsyncStorage.getItem(ImageCacheService.METADATA_KEY);
      if (metadataJson) {
        const stored = JSON.parse(metadataJson);
        // Validate version compatibility
        if (stored.version === ImageCacheService.VERSION) {
          this.metadata = { ...this.metadata, ...stored };
        } else {
          // Version mismatch - clear cache
          await this.clearCache();
        }
      }
    } catch (error) {
      console.warn('Failed to load cache metadata:', error);
    }
  }

  private async saveMetadata(): Promise<void> {
    try {
      await AsyncStorage.setItem(
        ImageCacheService.METADATA_KEY,
        JSON.stringify(this.metadata)
      );
    } catch (error) {
      console.error('Failed to save cache metadata:', error);
    }
  }

  private generateCacheKey(uri: string): string {
    return ImageCacheService.CACHE_KEY_PREFIX + Crypto.digestStringAsync(
      Crypto.CryptoDigestAlgorithm.SHA256,
      uri
    );
  }

  private generateFileName(uri: string, compression?: CompressionOptions): string {
    const hash = Crypto.digestStringAsync(Crypto.CryptoDigestAlgorithm.SHA256, uri);
    const extension = compression?.format || 'jpg';
    const sizeInfo = compression?.width && compression?.height 
      ? `_${compression.width}x${compression.height}` 
      : '';
    const qualityInfo = compression?.quality ? `_q${compression.quality}` : '';
    return `${hash}${sizeInfo}${qualityInfo}.${extension}`;
  }

  private async getImageInfo(uri: string): Promise<CachedImageInfo | null> {
    try {
      const cacheKey = await this.generateCacheKey(uri);
      const infoJson = await AsyncStorage.getItem(cacheKey);
      return infoJson ? JSON.parse(infoJson) : null;
    } catch (error) {
      console.warn('Failed to get image info:', error);
      return null;
    }
  }

  private async saveImageInfo(uri: string, info: CachedImageInfo): Promise<void> {
    try {
      const cacheKey = await this.generateCacheKey(uri);
      await AsyncStorage.setItem(cacheKey, JSON.stringify(info));
    } catch (error) {
      console.error('Failed to save image info:', error);
    }
  }

  private async deleteImageInfo(uri: string): Promise<void> {
    try {
      const cacheKey = await this.generateCacheKey(uri);
      await AsyncStorage.removeItem(cacheKey);
    } catch (error) {
      console.warn('Failed to delete image info:', error);
    }
  }

  private shouldCleanup(): boolean {
    return (
      this.metadata.totalSize > this.metadata.maxSize * ImageCacheService.CLEANUP_THRESHOLD ||
      Date.now() - this.metadata.lastCleanup > 24 * 60 * 60 * 1000 // 24 hours
    );
  }

  private async performCleanup(): Promise<void> {
    try {
      console.log('Starting image cache cleanup...');
      
      // Get all cached images
      const allKeys = await AsyncStorage.getAllKeys();
      const cacheKeys = allKeys.filter(key => key.startsWith(ImageCacheService.CACHE_KEY_PREFIX));
      
      const cachedImages: Array<{ uri: string; info: CachedImageInfo }> = [];
      
      for (const key of cacheKeys) {
        try {
          const infoJson = await AsyncStorage.getItem(key);
          if (infoJson) {
            const info: CachedImageInfo = JSON.parse(infoJson);
            // Extract URI from cache key (this is a simplified approach)
            cachedImages.push({ uri: key.replace(ImageCacheService.CACHE_KEY_PREFIX, ''), info });
          }
        } catch (error) {
          // Remove corrupted entries
          await AsyncStorage.removeItem(key);
        }
      }

      // Sort by priority and access patterns for intelligent cleanup
      cachedImages.sort((a, b) => {
        // Higher priority images are kept longer
        const priorityWeight = { high: 3, normal: 2, low: 1 };
        const priorityDiff = priorityWeight[b.info.priority] - priorityWeight[a.info.priority];
        if (priorityDiff !== 0) return priorityDiff;

        // More frequently accessed images are kept
        const accessDiff = b.info.accessCount - a.info.accessCount;
        if (accessDiff !== 0) return accessDiff;

        // More recently accessed images are kept
        return b.info.lastAccessed - a.info.lastAccessed;
      });

      // Remove expired images first
      const now = Date.now();
      let deletedSize = 0;
      let deletedCount = 0;

      for (const { uri, info } of cachedImages) {
        const isExpired = now - info.cachedAt > info.ttl;
        const shouldDelete = isExpired || (
          this.metadata.totalSize - deletedSize > this.metadata.maxSize * 0.7 && // Keep under 70%
          info.priority === 'low'
        );

        if (shouldDelete) {
          try {
            // Delete file
            const fileInfo = await FileSystem.getInfoAsync(info.localPath);
            if (fileInfo.exists) {
              await FileSystem.deleteAsync(info.localPath);
              deletedSize += info.size;
            }

            // Delete metadata
            await this.deleteImageInfo(uri);
            deletedCount++;
          } catch (error) {
            console.warn(`Failed to delete cached image ${uri}:`, error);
          }
        }
      }

      // Update metadata
      this.metadata.totalSize -= deletedSize;
      this.metadata.imageCount -= deletedCount;
      this.metadata.lastCleanup = now;
      await this.saveMetadata();

      console.log(`Cache cleanup completed: deleted ${deletedCount} images (${deletedSize} bytes)`);
    } catch (error) {
      console.error('Cache cleanup failed:', error);
    }
  }

  private async compressImage(
    sourceUri: string,
    options: CompressionOptions
  ): Promise<string> {
    try {
      const result = await ImageManipulator.manipulateAsync(
        sourceUri,
        options.width && options.height 
          ? [{ resize: { width: options.width, height: options.height } }]
          : [],
        {
          compress: options.quality,
          format: options.format === 'jpeg' ? ImageManipulator.SaveFormat.JPEG :
                  options.format === 'png' ? ImageManipulator.SaveFormat.PNG :
                  ImageManipulator.SaveFormat.WEBP,
        }
      );

      return result.uri;
    } catch (error) {
      console.warn('Image compression failed:', error);
      return sourceUri; // Return original if compression fails
    }
  }

  public async getCachedImageUri(uri: string): Promise<string | null> {
    try {
      const info = await this.getImageInfo(uri);
      if (!info) return null;

      // Check if file still exists
      const fileInfo = await FileSystem.getInfoAsync(info.localPath);
      if (!fileInfo.exists) {
        // File was deleted externally, clean up metadata
        await this.deleteImageInfo(uri);
        return null;
      }

      // Check TTL
      if (Date.now() - info.cachedAt > info.ttl) {
        await this.deleteCachedImage(uri);
        return null;
      }

      // Update access tracking
      info.accessCount++;
      info.lastAccessed = Date.now();
      await this.saveImageInfo(uri, info);

      return info.localPath;
    } catch (error) {
      console.warn('Failed to get cached image:', error);
      return null;
    }
  }

  public async cacheImage(
    uri: string,
    priority: CachePriority = 'normal',
    compressionOptions?: CompressionOptions
  ): Promise<string> {
    try {
      // Check if already cached
      const existingPath = await this.getCachedImageUri(uri);
      if (existingPath) return existingPath;

      // Check if compression is already in progress
      const existingCompression = this.compressionQueue.get(uri);
      if (existingCompression) {
        return await existingCompression;
      }

      // Start compression/caching process
      const compressionPromise = this.performImageCaching(uri, priority, compressionOptions);
      this.compressionQueue.set(uri, compressionPromise);

      try {
        const result = await compressionPromise;
        return result;
      } finally {
        this.compressionQueue.delete(uri);
      }
    } catch (error) {
      console.error('Failed to cache image:', error);
      throw error;
    }
  }

  private async performImageCaching(
    uri: string,
    priority: CachePriority,
    compressionOptions?: CompressionOptions
  ): Promise<string> {
    // Download image
    const fileName = await this.generateFileName(uri, compressionOptions);
    const localPath = this.cacheDir + fileName;

    // Download to temporary location first
    const downloadResult = await FileSystem.downloadAsync(uri, localPath);
    if (downloadResult.status !== 200) {
      throw new Error(`Failed to download image: ${downloadResult.status}`);
    }

    let finalPath = localPath;
    let compressionLevel = 1.0;

    // Apply compression if specified
    if (compressionOptions) {
      const compressedPath = await this.compressImage(localPath, compressionOptions);
      
      // If compression created a new file, replace the original
      if (compressedPath !== localPath) {
        await FileSystem.deleteAsync(localPath);
        const compressedFileName = await this.generateFileName(uri + '_compressed', compressionOptions);
        finalPath = this.cacheDir + compressedFileName;
        await FileSystem.moveAsync({ from: compressedPath, to: finalPath });
        compressionLevel = compressionOptions.quality;
      }
    }

    // Get file size
    const fileInfo = await FileSystem.getInfoAsync(finalPath);
    const size = fileInfo.size || 0;

    // Save image info
    const now = Date.now();
    const ttl = priority === 'high' 
      ? ImageCacheService.HIGH_PRIORITY_TTL 
      : ImageCacheService.DEFAULT_TTL;

    const imageInfo: CachedImageInfo = {
      uri,
      localPath: finalPath,
      size,
      cachedAt: now,
      ttl,
      compressionLevel,
      priority,
      accessCount: 1,
      lastAccessed: now,
    };

    await this.saveImageInfo(uri, imageInfo);

    // Update metadata
    this.metadata.totalSize += size;
    this.metadata.imageCount++;
    await this.saveMetadata();

    // Check if cleanup is needed
    if (this.shouldCleanup()) {
      // Perform cleanup in background
      this.performCleanup().catch(error => 
        console.warn('Background cleanup failed:', error)
      );
    }

    return finalPath;
  }

  public async deleteCachedImage(uri: string): Promise<void> {
    try {
      const info = await this.getImageInfo(uri);
      if (!info) return;

      // Delete file
      const fileInfo = await FileSystem.getInfoAsync(info.localPath);
      if (fileInfo.exists) {
        await FileSystem.deleteAsync(info.localPath);
        this.metadata.totalSize -= info.size;
        this.metadata.imageCount--;
      }

      // Delete metadata
      await this.deleteImageInfo(uri);
      await this.saveMetadata();
    } catch (error) {
      console.warn('Failed to delete cached image:', error);
    }
  }

  public async clearCache(): Promise<void> {
    try {
      // Delete all cached files
      const dirInfo = await FileSystem.getInfoAsync(this.cacheDir);
      if (dirInfo.exists) {
        await FileSystem.deleteAsync(this.cacheDir);
        await FileSystem.makeDirectoryAsync(this.cacheDir, { intermediates: true });
      }

      // Clear AsyncStorage entries
      const allKeys = await AsyncStorage.getAllKeys();
      const cacheKeys = allKeys.filter(key => 
        key.startsWith(ImageCacheService.CACHE_KEY_PREFIX) || 
        key === ImageCacheService.METADATA_KEY
      );
      await AsyncStorage.multiRemove(cacheKeys);

      // Reset metadata
      this.metadata = {
        totalSize: 0,
        maxSize: ImageCacheService.DEFAULT_MAX_SIZE,
        imageCount: 0,
        lastCleanup: Date.now(),
        version: ImageCacheService.VERSION,
      };
      await this.saveMetadata();

      console.log('Image cache cleared successfully');
    } catch (error) {
      console.error('Failed to clear cache:', error);
    }
  }

  public async getCacheInfo(): Promise<ImageCacheMetadata> {
    await this.loadMetadata();
    return { ...this.metadata };
  }

  public async preloadImages(uris: string[], priority: CachePriority = 'low'): Promise<void> {
    const compressionOptions: CompressionOptions = {
      quality: 0.8,
      format: 'jpeg',
    };

    const preloadPromises = uris.map(uri => 
      this.cacheImage(uri, priority, compressionOptions).catch(error => 
        console.warn(`Failed to preload image ${uri}:`, error)
      )
    );

    await Promise.allSettled(preloadPromises);
  }
}

// Singleton instance
export const imageCache = new ImageCacheService();