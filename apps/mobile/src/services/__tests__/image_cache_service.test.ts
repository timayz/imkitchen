import { ImageCacheService, CachedImageInfo } from '../image_cache_service';
import AsyncStorage from '@react-native-async-storage/async-storage';
import * as FileSystem from 'expo-file-system';
import * as ImageManipulator from 'expo-image-manipulator';
import * as Crypto from 'expo-crypto';

// Mock dependencies
jest.mock('@react-native-async-storage/async-storage', () => ({
  getItem: jest.fn(),
  setItem: jest.fn(),
  removeItem: jest.fn(),
  getAllKeys: jest.fn(),
  multiRemove: jest.fn(),
}));

jest.mock('expo-file-system', () => ({
  documentDirectory: '/mock/documents/',
  getInfoAsync: jest.fn(),
  makeDirectoryAsync: jest.fn(),
  downloadAsync: jest.fn(),
  deleteAsync: jest.fn(),
  moveAsync: jest.fn(),
}));

jest.mock('expo-image-manipulator', () => ({
  manipulateAsync: jest.fn(),
  SaveFormat: {
    JPEG: 'jpeg',
    PNG: 'png',
    WEBP: 'webp',
  },
}));

jest.mock('expo-crypto', () => ({
  digestStringAsync: jest.fn(),
  CryptoDigestAlgorithm: {
    SHA256: 'SHA256',
  },
}));

describe('ImageCacheService', () => {
  let cacheService: ImageCacheService;
  const mockUri = 'https://example.com/image.jpg';
  const mockHash = 'mock-hash-12345';
  const mockCacheKey = 'image_cache_mock-hash-12345';
  const mockLocalPath = '/mock/documents/image_cache/mock-hash-12345.jpg';

  beforeEach(() => {
    jest.clearAllMocks();
    
    // Setup default mocks
    (Crypto.digestStringAsync as jest.Mock).mockResolvedValue(mockHash);
    (FileSystem.getInfoAsync as jest.Mock).mockResolvedValue({ exists: true });
    (AsyncStorage.getItem as jest.Mock).mockResolvedValue(null);
    (AsyncStorage.setItem as jest.Mock).mockResolvedValue(undefined);
    (FileSystem.makeDirectoryAsync as jest.Mock).mockResolvedValue(undefined);
    (FileSystem.downloadAsync as jest.Mock).mockResolvedValue({ status: 200 });
    (ImageManipulator.manipulateAsync as jest.Mock).mockResolvedValue({ uri: mockLocalPath });

    cacheService = new ImageCacheService();
  });

  describe('getCachedImageUri', () => {
    it('should return null for non-cached image', async () => {
      const result = await cacheService.getCachedImageUri(mockUri);
      expect(result).toBeNull();
    });

    it('should return cached image path for valid cached image', async () => {
      const mockImageInfo: CachedImageInfo = {
        uri: mockUri,
        localPath: mockLocalPath,
        size: 1024,
        cachedAt: Date.now(),
        ttl: 7 * 24 * 60 * 60 * 1000,
        compressionLevel: 0.8,
        priority: 'normal',
        accessCount: 1,
        lastAccessed: Date.now(),
      };

      (AsyncStorage.getItem as jest.Mock).mockResolvedValue(JSON.stringify(mockImageInfo));
      (FileSystem.getInfoAsync as jest.Mock).mockResolvedValue({ exists: true });

      const result = await cacheService.getCachedImageUri(mockUri);
      expect(result).toBe(mockLocalPath);
    });

    it('should return null for expired cached image', async () => {
      const expiredImageInfo: CachedImageInfo = {
        uri: mockUri,
        localPath: mockLocalPath,
        size: 1024,
        cachedAt: Date.now() - (8 * 24 * 60 * 60 * 1000), // 8 days ago
        ttl: 7 * 24 * 60 * 60 * 1000, // 7 days TTL
        compressionLevel: 0.8,
        priority: 'normal',
        accessCount: 1,
        lastAccessed: Date.now() - (8 * 24 * 60 * 60 * 1000),
      };

      (AsyncStorage.getItem as jest.Mock).mockResolvedValue(JSON.stringify(expiredImageInfo));
      (FileSystem.getInfoAsync as jest.Mock).mockResolvedValue({ exists: true });

      const result = await cacheService.getCachedImageUri(mockUri);
      expect(result).toBeNull();
    });

    it('should clean up metadata if file does not exist', async () => {
      const mockImageInfo: CachedImageInfo = {
        uri: mockUri,
        localPath: mockLocalPath,
        size: 1024,
        cachedAt: Date.now(),
        ttl: 7 * 24 * 60 * 60 * 1000,
        compressionLevel: 0.8,
        priority: 'normal',
        accessCount: 1,
        lastAccessed: Date.now(),
      };

      (AsyncStorage.getItem as jest.Mock).mockResolvedValue(JSON.stringify(mockImageInfo));
      (FileSystem.getInfoAsync as jest.Mock).mockResolvedValue({ exists: false });

      const result = await cacheService.getCachedImageUri(mockUri);
      expect(result).toBeNull();
      expect(AsyncStorage.removeItem).toHaveBeenCalledWith(mockCacheKey);
    });

    it('should update access tracking for valid cached images', async () => {
      const mockImageInfo: CachedImageInfo = {
        uri: mockUri,
        localPath: mockLocalPath,
        size: 1024,
        cachedAt: Date.now(),
        ttl: 7 * 24 * 60 * 60 * 1000,
        compressionLevel: 0.8,
        priority: 'normal',
        accessCount: 5,
        lastAccessed: Date.now() - 1000,
      };

      (AsyncStorage.getItem as jest.Mock).mockResolvedValue(JSON.stringify(mockImageInfo));
      (FileSystem.getInfoAsync as jest.Mock).mockResolvedValue({ exists: true });

      await cacheService.getCachedImageUri(mockUri);

      expect(AsyncStorage.setItem).toHaveBeenCalledWith(
        mockCacheKey,
        expect.stringContaining('"accessCount":6')
      );
    });
  });

  describe('cacheImage', () => {
    beforeEach(() => {
      (FileSystem.getInfoAsync as jest.Mock).mockResolvedValue({ 
        exists: true, 
        size: 2048 
      });
    });

    it('should cache an image successfully', async () => {
      const result = await cacheService.cacheImage(mockUri, 'normal');
      
      expect(FileSystem.downloadAsync).toHaveBeenCalledWith(
        mockUri,
        expect.stringContaining('mock-hash-12345.jpg')
      );
      expect(AsyncStorage.setItem).toHaveBeenCalledWith(
        mockCacheKey,
        expect.stringContaining(mockUri)
      );
      expect(result).toContain('mock-hash-12345.jpg');
    });

    it('should return existing cached image if already cached', async () => {
      const mockImageInfo: CachedImageInfo = {
        uri: mockUri,
        localPath: mockLocalPath,
        size: 1024,
        cachedAt: Date.now(),
        ttl: 7 * 24 * 60 * 60 * 1000,
        compressionLevel: 0.8,
        priority: 'normal',
        accessCount: 1,
        lastAccessed: Date.now(),
      };

      (AsyncStorage.getItem as jest.Mock).mockResolvedValue(JSON.stringify(mockImageInfo));

      const result = await cacheService.cacheImage(mockUri, 'normal');
      
      expect(result).toBe(mockLocalPath);
      expect(FileSystem.downloadAsync).not.toHaveBeenCalled();
    });

    it('should apply compression when specified', async () => {
      const compressionOptions = {
        quality: 0.7,
        width: 300,
        height: 300,
        format: 'jpeg' as const,
      };

      await cacheService.cacheImage(mockUri, 'normal', compressionOptions);

      expect(ImageManipulator.manipulateAsync).toHaveBeenCalledWith(
        expect.any(String),
        [{ resize: { width: 300, height: 300 } }],
        expect.objectContaining({
          compress: 0.7,
          format: 'jpeg',
        })
      );
    });

    it('should handle download failures gracefully', async () => {
      (FileSystem.downloadAsync as jest.Mock).mockResolvedValue({ status: 404 });

      await expect(cacheService.cacheImage(mockUri, 'normal')).rejects.toThrow();
    });

    it('should prevent duplicate compression requests', async () => {
      const promise1 = cacheService.cacheImage(mockUri, 'normal');
      const promise2 = cacheService.cacheImage(mockUri, 'normal');

      const [result1, result2] = await Promise.all([promise1, promise2]);

      expect(result1).toBe(result2);
      expect(FileSystem.downloadAsync).toHaveBeenCalledTimes(1);
    });
  });

  describe('deleteCachedImage', () => {
    it('should delete cached image and metadata', async () => {
      const mockImageInfo: CachedImageInfo = {
        uri: mockUri,
        localPath: mockLocalPath,
        size: 1024,
        cachedAt: Date.now(),
        ttl: 7 * 24 * 60 * 60 * 1000,
        compressionLevel: 0.8,
        priority: 'normal',
        accessCount: 1,
        lastAccessed: Date.now(),
      };

      (AsyncStorage.getItem as jest.Mock).mockResolvedValue(JSON.stringify(mockImageInfo));

      await cacheService.deleteCachedImage(mockUri);

      expect(FileSystem.deleteAsync).toHaveBeenCalledWith(mockLocalPath);
      expect(AsyncStorage.removeItem).toHaveBeenCalledWith(mockCacheKey);
    });

    it('should handle deletion of non-existent images gracefully', async () => {
      (AsyncStorage.getItem as jest.Mock).mockResolvedValue(null);

      await expect(cacheService.deleteCachedImage(mockUri)).resolves.not.toThrow();
    });
  });

  describe('clearCache', () => {
    it('should clear entire cache', async () => {
      (FileSystem.getInfoAsync as jest.Mock).mockResolvedValue({ exists: true });
      (AsyncStorage.getAllKeys as jest.Mock).mockResolvedValue([
        'image_cache_hash1',
        'image_cache_hash2',
        'image_cache_metadata',
        'other_key',
      ]);

      await cacheService.clearCache();

      expect(FileSystem.deleteAsync).toHaveBeenCalledWith(
        expect.stringContaining('image_cache/')
      );
      expect(FileSystem.makeDirectoryAsync).toHaveBeenCalledWith(
        expect.stringContaining('image_cache/'),
        { intermediates: true }
      );
      expect(AsyncStorage.multiRemove).toHaveBeenCalledWith([
        'image_cache_hash1',
        'image_cache_hash2',
        'image_cache_metadata',
      ]);
    });
  });

  describe('preloadImages', () => {
    it('should preload multiple images', async () => {
      const uris = [
        'https://example.com/image1.jpg',
        'https://example.com/image2.jpg',
        'https://example.com/image3.jpg',
      ];

      await cacheService.preloadImages(uris, 'low');

      expect(FileSystem.downloadAsync).toHaveBeenCalledTimes(3);
    });

    it('should handle partial failures in preloading', async () => {
      const uris = [
        'https://example.com/image1.jpg',
        'https://example.com/image2.jpg',
      ];

      (FileSystem.downloadAsync as jest.Mock)
        .mockResolvedValueOnce({ status: 200 })
        .mockResolvedValueOnce({ status: 404 });

      await expect(cacheService.preloadImages(uris, 'low')).resolves.not.toThrow();
    });
  });

  describe('cache metadata management', () => {
    it('should track cache size and count correctly', async () => {
      (FileSystem.getInfoAsync as jest.Mock).mockResolvedValue({ 
        exists: true, 
        size: 1024 
      });

      await cacheService.cacheImage(mockUri, 'normal');

      const metadata = await cacheService.getCacheInfo();
      expect(metadata.totalSize).toBeGreaterThan(0);
      expect(metadata.imageCount).toBeGreaterThan(0);
    });

    it('should update metadata after deletion', async () => {
      const mockImageInfo: CachedImageInfo = {
        uri: mockUri,
        localPath: mockLocalPath,
        size: 1024,
        cachedAt: Date.now(),
        ttl: 7 * 24 * 60 * 60 * 1000,
        compressionLevel: 0.8,
        priority: 'normal',
        accessCount: 1,
        lastAccessed: Date.now(),
      };

      (AsyncStorage.getItem as jest.Mock).mockResolvedValue(JSON.stringify(mockImageInfo));

      await cacheService.deleteCachedImage(mockUri);

      // Verify metadata was updated in AsyncStorage
      expect(AsyncStorage.setItem).toHaveBeenCalledWith(
        'image_cache_metadata',
        expect.any(String)
      );
    });
  });

  describe('priority-based caching', () => {
    it('should use longer TTL for high priority images', async () => {
      await cacheService.cacheImage(mockUri, 'high');

      const setItemCall = (AsyncStorage.setItem as jest.Mock).mock.calls.find(
        call => call[0] === mockCacheKey
      );
      
      expect(setItemCall).toBeDefined();
      const imageInfo = JSON.parse(setItemCall[1]);
      expect(imageInfo.ttl).toBeGreaterThan(7 * 24 * 60 * 60 * 1000); // More than 7 days
    });

    it('should use standard TTL for normal priority images', async () => {
      await cacheService.cacheImage(mockUri, 'normal');

      const setItemCall = (AsyncStorage.setItem as jest.Mock).mock.calls.find(
        call => call[0] === mockCacheKey
      );
      
      expect(setItemCall).toBeDefined();
      const imageInfo = JSON.parse(setItemCall[1]);
      expect(imageInfo.ttl).toBe(7 * 24 * 60 * 60 * 1000); // Exactly 7 days
    });
  });
});