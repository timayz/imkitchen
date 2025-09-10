// import NetInfo from '@react-native-community/netinfo';
// Mock types for compilation - would be replaced with actual shared types
export interface Recipe {
  id: string;
  name: string;
  description?: string;
  ingredients: any[];
  instructions: any[];
  createdAt: Date;
  updatedAt: Date;
}

export interface RecipeSearchParams {
  query?: string;
  tags?: string[];
  limit?: number;
  offset?: number;
}

export interface RecipeSearchResponse {
  recipes: Recipe[];
  total: number;
}

export interface CreateRecipeInput {
  name: string;
  description?: string;
  ingredients: any[];
  instructions: any[];
}

export interface UpdateRecipeInput extends Partial<CreateRecipeInput> {}

// Mock API client for compilation
class MockRecipeClient {
  async searchRecipes(params: RecipeSearchParams): Promise<RecipeSearchResponse> {
    throw new Error('API client not implemented');
  }
  async getRecipe(id: string): Promise<Recipe> {
    throw new Error('API client not implemented');
  }
  async createRecipe(input: CreateRecipeInput): Promise<Recipe> {
    throw new Error('API client not implemented');
  }
  async updateRecipe(id: string, input: UpdateRecipeInput): Promise<Recipe> {
    throw new Error('API client not implemented');
  }
  async deleteRecipe(id: string): Promise<void> {
    throw new Error('API client not implemented');
  }
}

const RecipeClient = MockRecipeClient;
import { RecipeCacheService, CachedRecipe } from './recipe_cache_service';

export interface NetworkStatus {
  isConnected: boolean;
  type: string;
  isInternetReachable: boolean | null;
}

export interface OfflineOperationResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  fromCache: boolean;
  requiresSync: boolean;
}

export class OfflineRecipeRepository {
  private recipeClient: RecipeClient;
  private cacheService: RecipeCacheService;
  private networkStatus: NetworkStatus = {
    isConnected: false,
    type: 'unknown',
    isInternetReachable: null,
  };

  constructor(recipeClient: RecipeClient, cacheService: RecipeCacheService) {
    this.recipeClient = recipeClient;
    this.cacheService = cacheService;
    this.initializeNetworkListener();
  }

  private initializeNetworkListener(): void {
    NetInfo.addEventListener(state => {
      this.networkStatus = {
        isConnected: state.isConnected ?? false,
        type: state.type,
        isInternetReachable: state.isInternetReachable,
      };
    });
  }

  async getRecipe(id: string): Promise<OfflineOperationResult<Recipe>> {
    // Always check cache first (cache-first strategy)
    const cached = await this.cacheService.getCachedRecipe(id);
    
    if (cached) {
      // Return cached data immediately
      const cacheResult: OfflineOperationResult<Recipe> = {
        success: true,
        data: cached.data,
        fromCache: true,
        requiresSync: cached.syncStatus === 'pending',
      };

      // If online, try to update cache in background
      if (this.isOnline()) {
        this.refreshCacheInBackground(id);
      }

      return cacheResult;
    }

    // Not in cache, try network if online
    if (this.isOnline()) {
      try {
        const recipe = await this.recipeClient.getRecipe(id);
        
        // Cache the fresh data
        await this.cacheService.cacheRecipe(recipe, {
          offline: false,
          syncStatus: 'synced',
        });

        return {
          success: true,
          data: recipe,
          fromCache: false,
          requiresSync: false,
        };
      } catch (error) {
        return {
          success: false,
          error: error instanceof Error ? error.message : 'Failed to fetch recipe',
          fromCache: false,
          requiresSync: false,
        };
      }
    }

    // Offline and not in cache
    return {
      success: false,
      error: 'Recipe not available offline',
      fromCache: false,
      requiresSync: false,
    };
  }

  async searchRecipes(params: RecipeSearchParams): Promise<OfflineOperationResult<RecipeSearchResponse>> {
    if (this.isOnline()) {
      try {
        const results = await this.recipeClient.searchRecipes(params);
        
        // Cache all recipes from search results
        await Promise.all(
          results.recipes.map(recipe =>
            this.cacheService.cacheRecipe(recipe, {
              offline: false,
              syncStatus: 'synced',
            })
          )
        );

        return {
          success: true,
          data: results,
          fromCache: false,
          requiresSync: false,
        };
      } catch (error) {
        // Fall back to cached results if online fetch fails
        return this.getCachedSearchResults(params);
      }
    }

    // Offline - return cached results
    return this.getCachedSearchResults(params);
  }

  async createRecipe(input: CreateRecipeInput): Promise<OfflineOperationResult<Recipe>> {
    if (this.isOnline()) {
      try {
        const recipe = await this.recipeClient.createRecipe(input);
        
        // Cache the new recipe
        await this.cacheService.cacheRecipe(recipe, {
          offline: false,
          syncStatus: 'synced',
        });

        return {
          success: true,
          data: recipe,
          fromCache: false,
          requiresSync: false,
        };
      } catch (error) {
        // Create offline placeholder and queue for sync
        return this.createOfflineRecipe(input);
      }
    }

    // Offline - create placeholder and queue for sync
    return this.createOfflineRecipe(input);
  }

  async updateRecipe(id: string, input: UpdateRecipeInput): Promise<OfflineOperationResult<Recipe>> {
    if (this.isOnline()) {
      try {
        const recipe = await this.recipeClient.updateRecipe(id, input);
        
        // Update cache with fresh data
        await this.cacheService.cacheRecipe(recipe, {
          offline: false,
          syncStatus: 'synced',
        });

        return {
          success: true,
          data: recipe,
          fromCache: false,
          requiresSync: false,
        };
      } catch (error) {
        // Update cached version and mark for sync
        return this.updateOfflineRecipe(id, input);
      }
    }

    // Offline - update cached version and mark for sync
    return this.updateOfflineRecipe(id, input);
  }

  async deleteRecipe(id: string): Promise<OfflineOperationResult<boolean>> {
    if (this.isOnline()) {
      try {
        await this.recipeClient.deleteRecipe(id);
        
        // Remove from cache
        await this.cacheService.removeCachedRecipe(id);

        return {
          success: true,
          data: true,
          fromCache: false,
          requiresSync: false,
        };
      } catch (error) {
        // Mark as deleted offline (soft delete)
        return this.deleteOfflineRecipe(id);
      }
    }

    // Offline - mark as deleted and queue for sync
    return this.deleteOfflineRecipe(id);
  }

  async syncPendingOperations(): Promise<{ synced: number; failed: number; errors: string[] }> {
    if (!this.isOnline()) {
      return { synced: 0, failed: 0, errors: ['Device is offline'] };
    }

    const pendingRecipes = await this.cacheService.getPendingSyncRecipes();
    let synced = 0;
    let failed = 0;
    const errors: string[] = [];

    for (const cached of pendingRecipes) {
      try {
        // Attempt to sync based on the operation type stored in metadata
        // This is a simplified approach - in a real implementation, you'd store
        // operation metadata with each cached recipe
        
        await this.cacheService.updateCachedRecipeStatus(cached.id, 'synced');
        synced++;
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Unknown sync error';
        errors.push(`Failed to sync recipe ${cached.id}: ${errorMessage}`);
        
        await this.cacheService.updateCachedRecipeStatus(cached.id, 'conflict');
        failed++;
      }
    }

    return { synced, failed, errors };
  }

  async getOfflineRecipes(): Promise<Recipe[]> {
    const offlineRecipes = await this.cacheService.getOfflineRecipes();
    return offlineRecipes.map(cached => cached.data);
  }

  async warmCacheWithFrequentRecipes(): Promise<void> {
    if (!this.isOnline()) {
      return;
    }

    const frequentIds = await this.cacheService.getFrequentlyAccessedRecipes(20);
    
    // Refresh cache for frequently accessed recipes in background
    await Promise.all(
      frequentIds.map(id => this.refreshCacheInBackground(id))
    );
  }

  getNetworkStatus(): NetworkStatus {
    return this.networkStatus;
  }

  isOnline(): boolean {
    return this.networkStatus.isConnected && this.networkStatus.isInternetReachable !== false;
  }

  private async refreshCacheInBackground(recipeId: string): Promise<void> {
    try {
      const recipe = await this.recipeClient.getRecipe(recipeId);
      await this.cacheService.cacheRecipe(recipe, {
        offline: false,
        syncStatus: 'synced',
      });
    } catch (error) {
      // Silently fail background refresh
      console.debug('Background cache refresh failed for recipe:', recipeId, error);
    }
  }

  private async getCachedSearchResults(params: RecipeSearchParams): Promise<OfflineOperationResult<RecipeSearchResponse>> {
    // This is a simplified implementation
    // In practice, you'd need more sophisticated offline search with indexing
    const offlineRecipes = await this.getOfflineRecipes();
    
    // Basic filtering (simplified)
    let filteredRecipes = offlineRecipes;
    
    if (params.search) {
      const searchLower = params.search.toLowerCase();
      filteredRecipes = filteredRecipes.filter(recipe =>
        recipe.title.toLowerCase().includes(searchLower) ||
        recipe.description?.toLowerCase().includes(searchLower)
      );
    }

    if (params.mealType?.length) {
      filteredRecipes = filteredRecipes.filter(recipe =>
        recipe.mealType.some(type => params.mealType!.includes(type))
      );
    }

    if (params.complexity?.length) {
      filteredRecipes = filteredRecipes.filter(recipe =>
        params.complexity!.includes(recipe.complexity)
      );
    }

    // Pagination
    const page = params.page || 1;
    const limit = params.limit || 20;
    const startIndex = (page - 1) * limit;
    const endIndex = startIndex + limit;
    
    const paginatedRecipes = filteredRecipes.slice(startIndex, endIndex);

    return {
      success: true,
      data: {
        recipes: paginatedRecipes,
        total: filteredRecipes.length,
        page,
        limit,
        totalPages: Math.ceil(filteredRecipes.length / limit),
      },
      fromCache: true,
      requiresSync: false,
    };
  }

  private async createOfflineRecipe(input: CreateRecipeInput): Promise<OfflineOperationResult<Recipe>> {
    // Generate temporary ID for offline recipe
    const tempId = `temp_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    const offlineRecipe: Recipe = {
      ...input,
      id: tempId,
      externalId: undefined,
      externalSource: 'user_generated',
      totalTime: input.prepTime + input.cookTime,
      averageRating: 0,
      totalRatings: 0,
      dietaryLabels: input.dietaryLabels || [],
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    await this.cacheService.cacheRecipe(offlineRecipe, {
      offline: true,
      syncStatus: 'pending',
    });

    return {
      success: true,
      data: offlineRecipe,
      fromCache: true,
      requiresSync: true,
    };
  }

  private async updateOfflineRecipe(id: string, input: UpdateRecipeInput): Promise<OfflineOperationResult<Recipe>> {
    const cached = await this.cacheService.getCachedRecipe(id);
    
    if (!cached) {
      return {
        success: false,
        error: 'Recipe not found in cache',
        fromCache: true,
        requiresSync: false,
      };
    }

    const updatedRecipe: Recipe = {
      ...cached.data,
      ...input,
      totalTime: (input.prepTime ?? cached.data.prepTime) + (input.cookTime ?? cached.data.cookTime),
      updatedAt: new Date(),
    };

    await this.cacheService.cacheRecipe(updatedRecipe, {
      offline: cached.offline,
      syncStatus: 'pending',
    });

    return {
      success: true,
      data: updatedRecipe,
      fromCache: true,
      requiresSync: true,
    };
  }

  private async deleteOfflineRecipe(id: string): Promise<OfflineOperationResult<boolean>> {
    const cached = await this.cacheService.getCachedRecipe(id);
    
    if (!cached) {
      return {
        success: false,
        error: 'Recipe not found in cache',
        fromCache: true,
        requiresSync: false,
      };
    }

    // Mark as deleted (soft delete) and queue for sync
    const deletedRecipe: Recipe = {
      ...cached.data,
      deletedAt: new Date(),
    };

    await this.cacheService.cacheRecipe(deletedRecipe, {
      offline: cached.offline,
      syncStatus: 'pending',
    });

    return {
      success: true,
      data: true,
      fromCache: true,
      requiresSync: true,
    };
  }
}