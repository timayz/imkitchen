import AsyncStorage from '@react-native-async-storage/async-storage';
import type { Recipe } from '@imkitchen/shared-types';

export interface CachedRecipe {
  id: string;
  data: Recipe;
  cachedAt: Date;
  ttl: number;
  offline: boolean;
  syncStatus: 'synced' | 'pending' | 'conflict';
}

export interface CacheMetadata {
  version: string;
  lastSync: Date;
  totalSize: number;
  maxSize: number;
}

export class RecipeCacheService {
  private static readonly CACHE_KEY_PREFIX = 'recipe_cache_';
  private static readonly METADATA_KEY = 'recipe_cache_metadata';
  private static readonly FREQUENTLY_ACCESSED_KEY = 'recipe_frequently_accessed';
  private static readonly DEFAULT_TTL = 24 * 60 * 60 * 1000; // 24 hours
  private static readonly MAX_CACHE_SIZE = 50 * 1024 * 1024; // 50MB
  private static readonly CACHE_VERSION = '1.0';

  async cacheRecipe(recipe: Recipe, options?: {
    ttl?: number;
    offline?: boolean;
    syncStatus?: 'synced' | 'pending' | 'conflict';
  }): Promise<void> {
    const cachedRecipe: CachedRecipe = {
      id: recipe.id,
      data: recipe,
      cachedAt: new Date(),
      ttl: options?.ttl ?? RecipeCacheService.DEFAULT_TTL,
      offline: options?.offline ?? false,
      syncStatus: options?.syncStatus ?? 'synced',
    };

    const key = this.getCacheKey(recipe.id);
    const serialized = JSON.stringify(cachedRecipe);
    
    await AsyncStorage.setItem(key, serialized);
    await this.updateMetadata(serialized.length);
    await this.trackAccess(recipe.id);
  }

  async getCachedRecipe(id: string): Promise<CachedRecipe | null> {
    try {
      const key = this.getCacheKey(id);
      const serialized = await AsyncStorage.getItem(key);
      
      if (!serialized) {
        return null;
      }

      const cached: CachedRecipe = JSON.parse(serialized);
      
      // Check TTL
      const now = new Date().getTime();
      const cachedAt = new Date(cached.cachedAt).getTime();
      
      if (now - cachedAt > cached.ttl) {
        // Expired, remove from cache
        await this.removeCachedRecipe(id);
        return null;
      }

      await this.trackAccess(id);
      return cached;
    } catch (error) {
      console.error('Error getting cached recipe:', error);
      return null;
    }
  }

  async removeCachedRecipe(id: string): Promise<void> {
    const key = this.getCacheKey(id);
    await AsyncStorage.removeItem(key);
    await this.updateMetadata(-1); // Approximate size reduction
  }

  async invalidateCache(recipeId?: string): Promise<void> {
    if (recipeId) {
      await this.removeCachedRecipe(recipeId);
    } else {
      // Clear entire cache
      const keys = await AsyncStorage.getAllKeys();
      const cacheKeys = keys.filter(key => key.startsWith(RecipeCacheService.CACHE_KEY_PREFIX));
      await AsyncStorage.multiRemove(cacheKeys);
      await this.resetMetadata();
    }
  }

  async updateCachedRecipeStatus(id: string, syncStatus: 'synced' | 'pending' | 'conflict'): Promise<void> {
    const cached = await this.getCachedRecipe(id);
    if (cached) {
      cached.syncStatus = syncStatus;
      await this.cacheRecipe(cached.data, {
        ttl: cached.ttl,
        offline: cached.offline,
        syncStatus,
      });
    }
  }

  async getOfflineRecipes(): Promise<CachedRecipe[]> {
    const keys = await AsyncStorage.getAllKeys();
    const cacheKeys = keys.filter(key => key.startsWith(RecipeCacheService.CACHE_KEY_PREFIX));
    
    const recipes: CachedRecipe[] = [];
    for (const key of cacheKeys) {
      const cached = await this.getCachedRecipe(key.replace(RecipeCacheService.CACHE_KEY_PREFIX, ''));
      if (cached && cached.offline) {
        recipes.push(cached);
      }
    }
    
    return recipes;
  }

  async getPendingSyncRecipes(): Promise<CachedRecipe[]> {
    const keys = await AsyncStorage.getAllKeys();
    const cacheKeys = keys.filter(key => key.startsWith(RecipeCacheService.CACHE_KEY_PREFIX));
    
    const recipes: CachedRecipe[] = [];
    for (const key of cacheKeys) {
      const cached = await this.getCachedRecipe(key.replace(RecipeCacheService.CACHE_KEY_PREFIX, ''));
      if (cached && cached.syncStatus === 'pending') {
        recipes.push(cached);
      }
    }
    
    return recipes;
  }

  async warmCache(recipeIds: string[]): Promise<void> {
    // This would typically fetch recipes from API and cache them
    // For now, we'll mark frequently accessed recipes for priority caching
    await this.updateFrequentlyAccessed(recipeIds);
  }

  async getCacheMetadata(): Promise<CacheMetadata> {
    try {
      const metadata = await AsyncStorage.getItem(RecipeCacheService.METADATA_KEY);
      if (metadata) {
        return JSON.parse(metadata);
      }
    } catch (error) {
      console.error('Error getting cache metadata:', error);
    }

    // Return default metadata
    return {
      version: RecipeCacheService.CACHE_VERSION,
      lastSync: new Date(),
      totalSize: 0,
      maxSize: RecipeCacheService.MAX_CACHE_SIZE,
    };
  }

  async cleanupExpiredEntries(): Promise<number> {
    const keys = await AsyncStorage.getAllKeys();
    const cacheKeys = keys.filter(key => key.startsWith(RecipeCacheService.CACHE_KEY_PREFIX));
    
    let cleanedCount = 0;
    for (const key of cacheKeys) {
      const id = key.replace(RecipeCacheService.CACHE_KEY_PREFIX, '');
      const cached = await this.getCachedRecipe(id);
      if (!cached) {
        cleanedCount++;
      }
    }
    
    return cleanedCount;
  }

  async getCacheSize(): Promise<number> {
    const keys = await AsyncStorage.getAllKeys();
    const cacheKeys = keys.filter(key => key.startsWith(RecipeCacheService.CACHE_KEY_PREFIX));
    
    let totalSize = 0;
    for (const key of cacheKeys) {
      try {
        const item = await AsyncStorage.getItem(key);
        if (item) {
          totalSize += item.length;
        }
      } catch (error) {
        console.error('Error calculating cache size for key:', key, error);
      }
    }
    
    return totalSize;
  }

  private getCacheKey(recipeId: string): string {
    return `${RecipeCacheService.CACHE_KEY_PREFIX}${recipeId}`;
  }

  private async updateMetadata(sizeChange: number): Promise<void> {
    const metadata = await this.getCacheMetadata();
    metadata.totalSize = Math.max(0, metadata.totalSize + sizeChange);
    metadata.lastSync = new Date();
    
    await AsyncStorage.setItem(RecipeCacheService.METADATA_KEY, JSON.stringify(metadata));
  }

  private async resetMetadata(): Promise<void> {
    const metadata: CacheMetadata = {
      version: RecipeCacheService.CACHE_VERSION,
      lastSync: new Date(),
      totalSize: 0,
      maxSize: RecipeCacheService.MAX_CACHE_SIZE,
    };
    
    await AsyncStorage.setItem(RecipeCacheService.METADATA_KEY, JSON.stringify(metadata));
  }

  private async trackAccess(recipeId: string): Promise<void> {
    try {
      const accessData = await AsyncStorage.getItem(RecipeCacheService.FREQUENTLY_ACCESSED_KEY);
      const accessed: Record<string, { count: number; lastAccessed: Date }> = accessData ? JSON.parse(accessData) : {};
      
      accessed[recipeId] = {
        count: (accessed[recipeId]?.count ?? 0) + 1,
        lastAccessed: new Date(),
      };
      
      await AsyncStorage.setItem(RecipeCacheService.FREQUENTLY_ACCESSED_KEY, JSON.stringify(accessed));
    } catch (error) {
      console.error('Error tracking recipe access:', error);
    }
  }

  private async updateFrequentlyAccessed(recipeIds: string[]): Promise<void> {
    try {
      const accessData = await AsyncStorage.getItem(RecipeCacheService.FREQUENTLY_ACCESSED_KEY);
      const accessed: Record<string, { count: number; lastAccessed: Date }> = accessData ? JSON.parse(accessData) : {};
      
      for (const id of recipeIds) {
        accessed[id] = {
          count: (accessed[id]?.count ?? 0) + 10, // Boost priority
          lastAccessed: new Date(),
        };
      }
      
      await AsyncStorage.setItem(RecipeCacheService.FREQUENTLY_ACCESSED_KEY, JSON.stringify(accessed));
    } catch (error) {
      console.error('Error updating frequently accessed recipes:', error);
    }
  }

  async getFrequentlyAccessedRecipes(limit: number = 10): Promise<string[]> {
    try {
      const accessData = await AsyncStorage.getItem(RecipeCacheService.FREQUENTLY_ACCESSED_KEY);
      if (!accessData) {
        return [];
      }
      
      const accessed: Record<string, { count: number; lastAccessed: Date }> = JSON.parse(accessData);
      
      return Object.entries(accessed)
        .sort(([, a], [, b]) => b.count - a.count)
        .slice(0, limit)
        .map(([id]) => id);
    } catch (error) {
      console.error('Error getting frequently accessed recipes:', error);
      return [];
    }
  }
}