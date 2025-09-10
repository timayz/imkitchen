import type { ShoppingList } from '../types/shopping';

interface CacheEntry {
  data: ShoppingList;
  timestamp: number;
  expiresAt: number;
}

interface CacheOptions {
  ttl?: number; // Time to live in milliseconds
  maxSize?: number; // Maximum number of entries
}

class ShoppingCacheService {
  private cache = new Map<string, CacheEntry>();
  private accessTimes = new Map<string, number>(); // For LRU eviction
  private defaultTTL = 5 * 60 * 1000; // 5 minutes
  private maxSize = 50; // Maximum 50 cached shopping lists

  constructor(options: CacheOptions = {}) {
    this.defaultTTL = options.ttl || this.defaultTTL;
    this.maxSize = options.maxSize || this.maxSize;
  }

  // Generate cache key for shopping list requests
  private generateCacheKey(mealPlanId: string, mergeExisting: boolean): string {
    return `shopping_list:${mealPlanId}:merge_${mergeExisting}`;
  }

  // Get shopping list from cache
  get(mealPlanId: string, mergeExisting = false): ShoppingList | null {
    const key = this.generateCacheKey(mealPlanId, mergeExisting);
    const entry = this.cache.get(key);

    if (!entry) {
      return null;
    }

    // Check if entry has expired
    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      this.accessTimes.delete(key);
      return null;
    }

    // Update access time for LRU
    this.accessTimes.set(key, Date.now());
    
    return entry.data;
  }

  // Store shopping list in cache
  set(
    mealPlanId: string, 
    shoppingList: ShoppingList, 
    mergeExisting = false,
    customTTL?: number
  ): void {
    const key = this.generateCacheKey(mealPlanId, mergeExisting);
    const ttl = customTTL || this.defaultTTL;
    const now = Date.now();

    // Ensure cache doesn't exceed max size
    if (this.cache.size >= this.maxSize && !this.cache.has(key)) {
      this.evictLRU();
    }

    const entry: CacheEntry = {
      data: shoppingList,
      timestamp: now,
      expiresAt: now + ttl,
    };

    this.cache.set(key, entry);
    this.accessTimes.set(key, now);
  }

  // Remove specific entry from cache
  delete(mealPlanId: string, mergeExisting = false): boolean {
    const key = this.generateCacheKey(mealPlanId, mergeExisting);
    const deleted = this.cache.delete(key);
    this.accessTimes.delete(key);
    return deleted;
  }

  // Clear entire cache
  clear(): void {
    this.cache.clear();
    this.accessTimes.clear();
  }

  // Check if entry exists and is valid
  has(mealPlanId: string, mergeExisting = false): boolean {
    const key = this.generateCacheKey(mealPlanId, mergeExisting);
    const entry = this.cache.get(key);

    if (!entry) {
      return false;
    }

    // Check if expired
    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      this.accessTimes.delete(key);
      return false;
    }

    return true;
  }

  // Evict least recently used entry
  private evictLRU(): void {
    let oldestKey: string | null = null;
    let oldestTime = Infinity;

    for (const [key, accessTime] of this.accessTimes.entries()) {
      if (accessTime < oldestTime) {
        oldestTime = accessTime;
        oldestKey = key;
      }
    }

    if (oldestKey) {
      this.cache.delete(oldestKey);
      this.accessTimes.delete(oldestKey);
    }
  }

  // Clean up expired entries
  cleanup(): void {
    const now = Date.now();
    const expiredKeys: string[] = [];

    for (const [key, entry] of this.cache.entries()) {
      if (now > entry.expiresAt) {
        expiredKeys.push(key);
      }
    }

    for (const key of expiredKeys) {
      this.cache.delete(key);
      this.accessTimes.delete(key);
    }
  }

  // Invalidate cache entries for a specific shopping list ID
  invalidateForShoppingList(shoppingListId: string): void {
    const keysToDelete: string[] = [];
    
    // Find all cache entries that might be affected by this shopping list
    // Since we don't store shopping list ID in cache key, we'll need to invalidate
    // entries that might be related
    for (const [key, entry] of this.cache.entries()) {
      if (entry.data.id === shoppingListId) {
        keysToDelete.push(key);
      }
    }

    for (const key of keysToDelete) {
      this.cache.delete(key);
      this.accessTimes.delete(key);
    }

    console.log(`Invalidated ${keysToDelete.length} cache entries for shopping list ${shoppingListId}`);
  }

  // Invalidate cache entries for a meal plan ID
  invalidateForMealPlan(mealPlanId: string): void {
    const keysToDelete: string[] = [];
    
    // Find all cache entries for this meal plan
    for (const key of this.cache.keys()) {
      if (key.includes(mealPlanId)) {
        keysToDelete.push(key);
      }
    }

    for (const key of keysToDelete) {
      this.cache.delete(key);
      this.accessTimes.delete(key);
    }

    console.log(`Invalidated ${keysToDelete.length} cache entries for meal plan ${mealPlanId}`);
  }

  // Get cache statistics
  getStats(): {
    size: number;
    maxSize: number;
    hitRate: number;
    oldestEntry?: Date;
    newestEntry?: Date;
  } {
    let oldestTimestamp = Infinity;
    let newestTimestamp = 0;

    for (const entry of this.cache.values()) {
      if (entry.timestamp < oldestTimestamp) {
        oldestTimestamp = entry.timestamp;
      }
      if (entry.timestamp > newestTimestamp) {
        newestTimestamp = entry.timestamp;
      }
    }

    return {
      size: this.cache.size,
      maxSize: this.maxSize,
      hitRate: this.calculateHitRate(),
      oldestEntry: oldestTimestamp !== Infinity ? new Date(oldestTimestamp) : undefined,
      newestEntry: newestTimestamp > 0 ? new Date(newestTimestamp) : undefined,
    };
  }

  // Calculate cache hit rate (simplified - would need request tracking in production)
  private calculateHitRate(): number {
    // This is a simplified calculation
    // In production, you'd track hits vs misses
    return this.cache.size > 0 ? 0.85 : 0; // Mock 85% hit rate when cache has entries
  }

  // Invalidate cache entries for a specific meal plan
  invalidateMealPlan(mealPlanId: string): void {
    const keysToDelete: string[] = [];

    for (const key of this.cache.keys()) {
      if (key.includes(`shopping_list:${mealPlanId}:`)) {
        keysToDelete.push(key);
      }
    }

    for (const key of keysToDelete) {
      this.cache.delete(key);
      this.accessTimes.delete(key);
    }
  }

  // Preload cache with commonly requested combinations
  preload(entries: Array<{ 
    mealPlanId: string; 
    shoppingList: ShoppingList; 
    mergeExisting?: boolean;
    ttl?: number;
  }>): void {
    for (const entry of entries) {
      this.set(
        entry.mealPlanId, 
        entry.shoppingList, 
        entry.mergeExisting || false,
        entry.ttl
      );
    }
  }

  // Get cache keys (useful for debugging)
  getKeys(): string[] {
    return Array.from(this.cache.keys());
  }

  // Export cache data (useful for persistence)
  export(): Array<{ 
    key: string; 
    data: ShoppingList; 
    timestamp: number; 
    expiresAt: number; 
  }> {
    const exported: Array<{ 
      key: string; 
      data: ShoppingList; 
      timestamp: number; 
      expiresAt: number; 
    }> = [];

    for (const [key, entry] of this.cache.entries()) {
      exported.push({
        key,
        data: entry.data,
        timestamp: entry.timestamp,
        expiresAt: entry.expiresAt,
      });
    }

    return exported;
  }

  // Import cache data (useful for persistence)
  import(data: Array<{ 
    key: string; 
    data: ShoppingList; 
    timestamp: number; 
    expiresAt: number; 
  }>): void {
    const now = Date.now();

    for (const item of data) {
      // Only import non-expired entries
      if (item.expiresAt > now) {
        this.cache.set(item.key, {
          data: item.data,
          timestamp: item.timestamp,
          expiresAt: item.expiresAt,
        });
        this.accessTimes.set(item.key, item.timestamp);
      }
    }
  }
}

// Create singleton instance
export const shoppingCacheService = new ShoppingCacheService({
  ttl: 10 * 60 * 1000, // 10 minutes for production
  maxSize: 100, // Store up to 100 shopping lists
});

// Auto-cleanup every 5 minutes
setInterval(() => {
  shoppingCacheService.cleanup();
}, 5 * 60 * 1000);