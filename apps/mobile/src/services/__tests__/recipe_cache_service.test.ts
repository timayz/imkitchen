import AsyncStorage from '@react-native-async-storage/async-storage';
import { RecipeCacheService, CachedRecipe } from '../recipe_cache_service';
import type { Recipe } from '@imkitchen/shared-types';

const mockAsyncStorage = AsyncStorage as jest.Mocked<typeof AsyncStorage>;

const createMockRecipe = (id: string): Recipe => ({
  id,
  title: `Test Recipe ${id}`,
  description: 'A test recipe',
  prepTime: 15,
  cookTime: 30,
  totalTime: 45,
  mealType: ['dinner'],
  complexity: 'simple',
  servings: 4,
  ingredients: [
    { name: 'Test ingredient', amount: 1, unit: 'cup', category: 'pantry' }
  ],
  instructions: [
    { stepNumber: 1, instruction: 'Test step 1' }
  ],
  averageRating: 4.5,
  totalRatings: 10,
  dietaryLabels: [],
  createdAt: new Date('2024-01-01'),
  updatedAt: new Date('2024-01-01'),
});

describe('RecipeCacheService', () => {
  let cacheService: RecipeCacheService;
  let mockRecipe: Recipe;

  beforeEach(() => {
    jest.clearAllMocks();
    cacheService = new RecipeCacheService();
    mockRecipe = createMockRecipe('test-id-1');
  });

  describe('cacheRecipe', () => {
    it('should cache a recipe with default TTL', async () => {
      await cacheService.cacheRecipe(mockRecipe);

      expect(mockAsyncStorage.setItem).toHaveBeenCalledWith(
        'recipe_cache_test-id-1',
        expect.stringContaining('"syncStatus":"synced"')
      );
    });

    it('should cache a recipe with custom options', async () => {
      const customTTL = 12 * 60 * 60 * 1000; // 12 hours
      await cacheService.cacheRecipe(mockRecipe, {
        ttl: customTTL,
        offline: true,
        syncStatus: 'pending',
      });

      expect(mockAsyncStorage.setItem).toHaveBeenCalledWith(
        'recipe_cache_test-id-1',
        expect.stringContaining('"offline":true')
      );
      expect(mockAsyncStorage.setItem).toHaveBeenCalledWith(
        'recipe_cache_test-id-1',
        expect.stringContaining('"syncStatus":"pending"')
      );
    });

    it('should update metadata after caching', async () => {
      await cacheService.cacheRecipe(mockRecipe);

      expect(mockAsyncStorage.setItem).toHaveBeenCalledWith(
        'recipe_cache_metadata',
        expect.stringContaining('"version":"1.0"')
      );
    });
  });

  describe('getCachedRecipe', () => {
    it('should return cached recipe if valid and not expired', async () => {
      const cachedRecipe: CachedRecipe = {
        id: mockRecipe.id,
        data: mockRecipe,
        cachedAt: new Date(),
        ttl: 24 * 60 * 60 * 1000, // 24 hours
        offline: false,
        syncStatus: 'synced',
      };

      mockAsyncStorage.getItem.mockResolvedValue(JSON.stringify(cachedRecipe));

      const result = await cacheService.getCachedRecipe('test-id-1');

      expect(result).toEqual(cachedRecipe);
      expect(mockAsyncStorage.getItem).toHaveBeenCalledWith('recipe_cache_test-id-1');
    });

    it('should return null if recipe is not cached', async () => {
      mockAsyncStorage.getItem.mockResolvedValue(null);

      const result = await cacheService.getCachedRecipe('non-existent-id');

      expect(result).toBeNull();
    });

    it('should remove and return null if recipe is expired', async () => {
      const expiredCachedRecipe: CachedRecipe = {
        id: mockRecipe.id,
        data: mockRecipe,
        cachedAt: new Date(Date.now() - 25 * 60 * 60 * 1000), // 25 hours ago
        ttl: 24 * 60 * 60 * 1000, // 24 hours TTL
        offline: false,
        syncStatus: 'synced',
      };

      mockAsyncStorage.getItem.mockResolvedValue(JSON.stringify(expiredCachedRecipe));

      const result = await cacheService.getCachedRecipe('test-id-1');

      expect(result).toBeNull();
      expect(mockAsyncStorage.removeItem).toHaveBeenCalledWith('recipe_cache_test-id-1');
    });

    it('should handle JSON parse errors gracefully', async () => {
      mockAsyncStorage.getItem.mockResolvedValue('invalid-json');

      const result = await cacheService.getCachedRecipe('test-id-1');

      expect(result).toBeNull();
    });
  });

  describe('removeCachedRecipe', () => {
    it('should remove recipe from cache', async () => {
      await cacheService.removeCachedRecipe('test-id-1');

      expect(mockAsyncStorage.removeItem).toHaveBeenCalledWith('recipe_cache_test-id-1');
    });
  });

  describe('invalidateCache', () => {
    it('should remove specific recipe when ID provided', async () => {
      await cacheService.invalidateCache('test-id-1');

      expect(mockAsyncStorage.removeItem).toHaveBeenCalledWith('recipe_cache_test-id-1');
    });

    it('should clear entire cache when no ID provided', async () => {
      mockAsyncStorage.getAllKeys.mockResolvedValue([
        'recipe_cache_test-id-1',
        'recipe_cache_test-id-2',
        'other_key',
      ]);

      await cacheService.invalidateCache();

      expect(mockAsyncStorage.multiRemove).toHaveBeenCalledWith([
        'recipe_cache_test-id-1',
        'recipe_cache_test-id-2',
      ]);
    });
  });

  describe('updateCachedRecipeStatus', () => {
    it('should update sync status of cached recipe', async () => {
      const cachedRecipe: CachedRecipe = {
        id: mockRecipe.id,
        data: mockRecipe,
        cachedAt: new Date(),
        ttl: 24 * 60 * 60 * 1000,
        offline: false,
        syncStatus: 'synced',
      };

      mockAsyncStorage.getItem.mockResolvedValue(JSON.stringify(cachedRecipe));

      await cacheService.updateCachedRecipeStatus('test-id-1', 'pending');

      expect(mockAsyncStorage.setItem).toHaveBeenCalledWith(
        'recipe_cache_test-id-1',
        expect.stringContaining('"syncStatus":"pending"')
      );
    });
  });

  describe('getOfflineRecipes', () => {
    it('should return only offline recipes', async () => {
      const onlineRecipe: CachedRecipe = {
        id: 'online-1',
        data: createMockRecipe('online-1'),
        cachedAt: new Date(),
        ttl: 24 * 60 * 60 * 1000,
        offline: false,
        syncStatus: 'synced',
      };

      const offlineRecipe: CachedRecipe = {
        id: 'offline-1',
        data: createMockRecipe('offline-1'),
        cachedAt: new Date(),
        ttl: 24 * 60 * 60 * 1000,
        offline: true,
        syncStatus: 'pending',
      };

      mockAsyncStorage.getAllKeys.mockResolvedValue([
        'recipe_cache_online-1',
        'recipe_cache_offline-1',
      ]);

      mockAsyncStorage.getItem
        .mockResolvedValueOnce(JSON.stringify(onlineRecipe))
        .mockResolvedValueOnce(JSON.stringify(offlineRecipe));

      const result = await cacheService.getOfflineRecipes();

      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('offline-1');
      expect(result[0].offline).toBe(true);
    });
  });

  describe('getPendingSyncRecipes', () => {
    it('should return only recipes with pending sync status', async () => {
      const syncedRecipe: CachedRecipe = {
        id: 'synced-1',
        data: createMockRecipe('synced-1'),
        cachedAt: new Date(),
        ttl: 24 * 60 * 60 * 1000,
        offline: false,
        syncStatus: 'synced',
      };

      const pendingRecipe: CachedRecipe = {
        id: 'pending-1',
        data: createMockRecipe('pending-1'),
        cachedAt: new Date(),
        ttl: 24 * 60 * 60 * 1000,
        offline: true,
        syncStatus: 'pending',
      };

      mockAsyncStorage.getAllKeys.mockResolvedValue([
        'recipe_cache_synced-1',
        'recipe_cache_pending-1',
      ]);

      mockAsyncStorage.getItem
        .mockResolvedValueOnce(JSON.stringify(syncedRecipe))
        .mockResolvedValueOnce(JSON.stringify(pendingRecipe));

      const result = await cacheService.getPendingSyncRecipes();

      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('pending-1');
      expect(result[0].syncStatus).toBe('pending');
    });
  });

  describe('cleanupExpiredEntries', () => {
    it('should return count of cleaned entries', async () => {
      const validRecipe: CachedRecipe = {
        id: 'valid-1',
        data: createMockRecipe('valid-1'),
        cachedAt: new Date(),
        ttl: 24 * 60 * 60 * 1000,
        offline: false,
        syncStatus: 'synced',
      };

      const expiredRecipe: CachedRecipe = {
        id: 'expired-1',
        data: createMockRecipe('expired-1'),
        cachedAt: new Date(Date.now() - 25 * 60 * 60 * 1000), // 25 hours ago
        ttl: 24 * 60 * 60 * 1000, // 24 hour TTL
        offline: false,
        syncStatus: 'synced',
      };

      mockAsyncStorage.getAllKeys.mockResolvedValue([
        'recipe_cache_valid-1',
        'recipe_cache_expired-1',
      ]);

      mockAsyncStorage.getItem
        .mockResolvedValueOnce(JSON.stringify(validRecipe))
        .mockResolvedValueOnce(JSON.stringify(expiredRecipe));

      const result = await cacheService.cleanupExpiredEntries();

      expect(result).toBe(1); // One expired entry cleaned
      expect(mockAsyncStorage.removeItem).toHaveBeenCalledWith('recipe_cache_expired-1');
    });
  });

  describe('getCacheSize', () => {
    it('should calculate total cache size', async () => {
      const recipe1Size = JSON.stringify(createMockRecipe('1')).length;
      const recipe2Size = JSON.stringify(createMockRecipe('2')).length;

      mockAsyncStorage.getAllKeys.mockResolvedValue([
        'recipe_cache_1',
        'recipe_cache_2',
        'other_key',
      ]);

      mockAsyncStorage.getItem
        .mockResolvedValueOnce(JSON.stringify(createMockRecipe('1')))
        .mockResolvedValueOnce(JSON.stringify(createMockRecipe('2')));

      const result = await cacheService.getCacheSize();

      expect(result).toBe(recipe1Size + recipe2Size);
    });
  });

  describe('getFrequentlyAccessedRecipes', () => {
    it('should return frequently accessed recipe IDs sorted by count', async () => {
      const accessData = {
        'recipe-1': { count: 10, lastAccessed: new Date() },
        'recipe-2': { count: 5, lastAccessed: new Date() },
        'recipe-3': { count: 15, lastAccessed: new Date() },
      };

      mockAsyncStorage.getItem.mockResolvedValue(JSON.stringify(accessData));

      const result = await cacheService.getFrequentlyAccessedRecipes(2);

      expect(result).toEqual(['recipe-3', 'recipe-1']);
    });

    it('should return empty array if no access data', async () => {
      mockAsyncStorage.getItem.mockResolvedValue(null);

      const result = await cacheService.getFrequentlyAccessedRecipes();

      expect(result).toEqual([]);
    });
  });
});