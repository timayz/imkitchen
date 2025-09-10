import { OfflineRecipeRepository, NetworkStatus } from '../offline_recipe_repository';
import { RecipeCacheService, CachedRecipe } from '../recipe_cache_service';
import { RecipeClient } from '@imkitchen/api-client';
import type { Recipe, CreateRecipeInput, UpdateRecipeInput, RecipeSearchParams } from '@imkitchen/shared-types';

// Mock NetInfo
jest.mock('@react-native-community/netinfo', () => ({
  addEventListener: jest.fn(),
}));

// Mock RecipeClient
const mockRecipeClient = {
  getRecipe: jest.fn(),
  searchRecipes: jest.fn(),
  createRecipe: jest.fn(),
  updateRecipe: jest.fn(),
  deleteRecipe: jest.fn(),
} as jest.Mocked<RecipeClient>;

// Mock RecipeCacheService
const mockCacheService = {
  cacheRecipe: jest.fn(),
  getCachedRecipe: jest.fn(),
  removeCachedRecipe: jest.fn(),
  invalidateCache: jest.fn(),
  updateCachedRecipeStatus: jest.fn(),
  getOfflineRecipes: jest.fn(),
  getPendingSyncRecipes: jest.fn(),
  warmCache: jest.fn(),
  getFrequentlyAccessedRecipes: jest.fn(),
} as jest.Mocked<RecipeCacheService>;

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

describe('OfflineRecipeRepository', () => {
  let repository: OfflineRecipeRepository;
  let mockRecipe: Recipe;

  beforeEach(() => {
    jest.clearAllMocks();
    repository = new OfflineRecipeRepository(mockRecipeClient, mockCacheService);
    mockRecipe = createMockRecipe('test-id-1');
    
    // Default to online status
    (repository as any).networkStatus = {
      isConnected: true,
      type: 'wifi',
      isInternetReachable: true,
    } as NetworkStatus;
  });

  describe('getRecipe', () => {
    it('should return cached recipe immediately if available', async () => {
      const cachedRecipe: CachedRecipe = {
        id: mockRecipe.id,
        data: mockRecipe,
        cachedAt: new Date(),
        ttl: 24 * 60 * 60 * 1000,
        offline: false,
        syncStatus: 'synced',
      };

      mockCacheService.getCachedRecipe.mockResolvedValue(cachedRecipe);

      const result = await repository.getRecipe('test-id-1');

      expect(result.success).toBe(true);
      expect(result.data).toEqual(mockRecipe);
      expect(result.fromCache).toBe(true);
      expect(result.requiresSync).toBe(false);
    });

    it('should fetch from network if not cached and online', async () => {
      mockCacheService.getCachedRecipe.mockResolvedValue(null);
      mockRecipeClient.getRecipe.mockResolvedValue(mockRecipe);

      const result = await repository.getRecipe('test-id-1');

      expect(result.success).toBe(true);
      expect(result.data).toEqual(mockRecipe);
      expect(result.fromCache).toBe(false);
      expect(mockCacheService.cacheRecipe).toHaveBeenCalledWith(mockRecipe, {
        offline: false,
        syncStatus: 'synced',
      });
    });

    it('should return error if offline and not cached', async () => {
      (repository as any).networkStatus.isConnected = false;
      mockCacheService.getCachedRecipe.mockResolvedValue(null);

      const result = await repository.getRecipe('test-id-1');

      expect(result.success).toBe(false);
      expect(result.error).toBe('Recipe not available offline');
      expect(result.fromCache).toBe(false);
    });

    it('should indicate sync required for pending recipes', async () => {
      const pendingRecipe: CachedRecipe = {
        id: mockRecipe.id,
        data: mockRecipe,
        cachedAt: new Date(),
        ttl: 24 * 60 * 60 * 1000,
        offline: false,
        syncStatus: 'pending',
      };

      mockCacheService.getCachedRecipe.mockResolvedValue(pendingRecipe);

      const result = await repository.getRecipe('test-id-1');

      expect(result.success).toBe(true);
      expect(result.requiresSync).toBe(true);
    });
  });

  describe('searchRecipes', () => {
    const searchParams: RecipeSearchParams = {
      search: 'test',
      page: 1,
      limit: 10,
    };

    it('should fetch and cache search results when online', async () => {
      const searchResponse = {
        recipes: [mockRecipe],
        total: 1,
        page: 1,
        limit: 10,
        totalPages: 1,
      };

      mockRecipeClient.searchRecipes.mockResolvedValue(searchResponse);

      const result = await repository.searchRecipes(searchParams);

      expect(result.success).toBe(true);
      expect(result.data).toEqual(searchResponse);
      expect(result.fromCache).toBe(false);
      expect(mockCacheService.cacheRecipe).toHaveBeenCalledWith(mockRecipe, {
        offline: false,
        syncStatus: 'synced',
      });
    });

    it('should return cached results when offline', async () => {
      (repository as any).networkStatus.isConnected = false;
      const offlineRecipes = [mockRecipe];
      mockCacheService.getOfflineRecipes.mockResolvedValue(
        offlineRecipes.map(recipe => ({
          id: recipe.id,
          data: recipe,
          cachedAt: new Date(),
          ttl: 24 * 60 * 60 * 1000,
          offline: true,
          syncStatus: 'synced',
        }))
      );

      repository.getOfflineRecipes = jest.fn().mockResolvedValue(offlineRecipes);

      const result = await repository.searchRecipes(searchParams);

      expect(result.success).toBe(true);
      expect(result.fromCache).toBe(true);
      expect(result.data?.recipes).toEqual([mockRecipe]);
    });

    it('should fallback to cached results if network request fails', async () => {
      mockRecipeClient.searchRecipes.mockRejectedValue(new Error('Network error'));
      const offlineRecipes = [mockRecipe];
      repository.getOfflineRecipes = jest.fn().mockResolvedValue(offlineRecipes);

      const result = await repository.searchRecipes(searchParams);

      expect(result.success).toBe(true);
      expect(result.fromCache).toBe(true);
    });
  });

  describe('createRecipe', () => {
    const createInput: CreateRecipeInput = {
      title: 'New Recipe',
      prepTime: 15,
      cookTime: 30,
      mealType: ['dinner'],
      complexity: 'simple',
      servings: 4,
      ingredients: [],
      instructions: [],
    };

    it('should create recipe online and cache it', async () => {
      mockRecipeClient.createRecipe.mockResolvedValue(mockRecipe);

      const result = await repository.createRecipe(createInput);

      expect(result.success).toBe(true);
      expect(result.data).toEqual(mockRecipe);
      expect(result.fromCache).toBe(false);
      expect(mockCacheService.cacheRecipe).toHaveBeenCalledWith(mockRecipe, {
        offline: false,
        syncStatus: 'synced',
      });
    });

    it('should create offline placeholder when offline', async () => {
      (repository as any).networkStatus.isConnected = false;

      const result = await repository.createRecipe(createInput);

      expect(result.success).toBe(true);
      expect(result.fromCache).toBe(true);
      expect(result.requiresSync).toBe(true);
      expect(result.data?.id).toMatch(/^temp_/);
    });

    it('should create offline placeholder when network request fails', async () => {
      mockRecipeClient.createRecipe.mockRejectedValue(new Error('Network error'));

      const result = await repository.createRecipe(createInput);

      expect(result.success).toBe(true);
      expect(result.fromCache).toBe(true);
      expect(result.requiresSync).toBe(true);
    });
  });

  describe('updateRecipe', () => {
    const updateInput: UpdateRecipeInput = {
      title: 'Updated Recipe',
      prepTime: 20,
    };

    it('should update recipe online and cache it', async () => {
      const updatedRecipe = { ...mockRecipe, ...updateInput };
      mockRecipeClient.updateRecipe.mockResolvedValue(updatedRecipe);

      const result = await repository.updateRecipe('test-id-1', updateInput);

      expect(result.success).toBe(true);
      expect(result.data).toEqual(updatedRecipe);
      expect(result.fromCache).toBe(false);
      expect(mockCacheService.cacheRecipe).toHaveBeenCalledWith(updatedRecipe, {
        offline: false,
        syncStatus: 'synced',
      });
    });

    it('should update cached version when offline', async () => {
      (repository as any).networkStatus.isConnected = false;
      const cachedRecipe: CachedRecipe = {
        id: mockRecipe.id,
        data: mockRecipe,
        cachedAt: new Date(),
        ttl: 24 * 60 * 60 * 1000,
        offline: false,
        syncStatus: 'synced',
      };

      mockCacheService.getCachedRecipe.mockResolvedValue(cachedRecipe);

      const result = await repository.updateRecipe('test-id-1', updateInput);

      expect(result.success).toBe(true);
      expect(result.fromCache).toBe(true);
      expect(result.requiresSync).toBe(true);
      expect(result.data?.title).toBe('Updated Recipe');
    });
  });

  describe('deleteRecipe', () => {
    it('should delete recipe online and remove from cache', async () => {
      mockRecipeClient.deleteRecipe.mockResolvedValue(undefined);

      const result = await repository.deleteRecipe('test-id-1');

      expect(result.success).toBe(true);
      expect(result.data).toBe(true);
      expect(result.fromCache).toBe(false);
      expect(mockCacheService.removeCachedRecipe).toHaveBeenCalledWith('test-id-1');
    });

    it('should mark as deleted offline when offline', async () => {
      (repository as any).networkStatus.isConnected = false;
      const cachedRecipe: CachedRecipe = {
        id: mockRecipe.id,
        data: mockRecipe,
        cachedAt: new Date(),
        ttl: 24 * 60 * 60 * 1000,
        offline: false,
        syncStatus: 'synced',
      };

      mockCacheService.getCachedRecipe.mockResolvedValue(cachedRecipe);

      const result = await repository.deleteRecipe('test-id-1');

      expect(result.success).toBe(true);
      expect(result.fromCache).toBe(true);
      expect(result.requiresSync).toBe(true);
    });
  });

  describe('syncPendingOperations', () => {
    it('should return offline status when offline', async () => {
      (repository as any).networkStatus.isConnected = false;

      const result = await repository.syncPendingOperations();

      expect(result.synced).toBe(0);
      expect(result.failed).toBe(0);
      expect(result.errors).toEqual(['Device is offline']);
    });

    it('should sync pending recipes when online', async () => {
      const pendingRecipes: CachedRecipe[] = [
        {
          id: 'pending-1',
          data: mockRecipe,
          cachedAt: new Date(),
          ttl: 24 * 60 * 60 * 1000,
          offline: false,
          syncStatus: 'pending',
        },
      ];

      mockCacheService.getPendingSyncRecipes.mockResolvedValue(pendingRecipes);

      const result = await repository.syncPendingOperations();

      expect(result.synced).toBe(1);
      expect(result.failed).toBe(0);
      expect(mockCacheService.updateCachedRecipeStatus).toHaveBeenCalledWith('pending-1', 'synced');
    });
  });

  describe('warmCacheWithFrequentRecipes', () => {
    it('should warm cache for frequently accessed recipes when online', async () => {
      const frequentIds = ['recipe-1', 'recipe-2', 'recipe-3'];
      mockCacheService.getFrequentlyAccessedRecipes.mockResolvedValue(frequentIds);
      mockRecipeClient.getRecipe.mockResolvedValue(mockRecipe);

      await repository.warmCacheWithFrequentRecipes();

      expect(mockCacheService.getFrequentlyAccessedRecipes).toHaveBeenCalledWith(20);
      expect(mockRecipeClient.getRecipe).toHaveBeenCalledTimes(3);
    });

    it('should not warm cache when offline', async () => {
      (repository as any).networkStatus.isConnected = false;

      await repository.warmCacheWithFrequentRecipes();

      expect(mockCacheService.getFrequentlyAccessedRecipes).not.toHaveBeenCalled();
      expect(mockRecipeClient.getRecipe).not.toHaveBeenCalled();
    });
  });

  describe('network status', () => {
    it('should return correct online status', () => {
      expect(repository.isOnline()).toBe(true);

      (repository as any).networkStatus.isConnected = false;
      expect(repository.isOnline()).toBe(false);

      (repository as any).networkStatus.isConnected = true;
      (repository as any).networkStatus.isInternetReachable = false;
      expect(repository.isOnline()).toBe(false);
    });

    it('should return current network status', () => {
      const status = repository.getNetworkStatus();
      expect(status.isConnected).toBe(true);
      expect(status.type).toBe('wifi');
    });
  });
});