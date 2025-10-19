/**
 * Global TypeScript type definitions for imkitchen PWA
 * Story 5.3 - Offline Recipe Access
 *
 * Defines types for window-exposed utilities used in E2E tests and application code
 */

// IndexedDB data models
interface Recipe {
  id: string;
  title: string;
  ingredients?: string;
  instructions?: string;
  image_url?: string;
  html?: string;
  url?: string;
  cached_at: string;
}

interface MealPlan {
  id: string;
  user_id: string;
  start_date: string;
  meals: Array<{
    date: string;
    meal_type: string;
    recipe_id?: string;
  }>;
  cached_at: string;
}

interface ShoppingList {
  id: string;
  week_start_date: string;
  items: Array<{
    id: string;
    ingredient: string;
    quantity: string;
    is_collected: boolean;
  }>;
  cached_at: string;
}

interface SyncQueueRequest {
  request_id?: number;
  url: string;
  method: string;
  body?: any;
  headers?: Record<string, string>;
  retry_count: number;
  queued_at: string;
}

interface CacheStats {
  recipes: number;
  mealPlans: number;
  shoppingLists: number;
  queuedRequests: number;
  totalCached: number;
}

// Window interface extensions
interface Window {
  /**
   * IndexedDB offline data persistence utilities
   * Exposed globally for E2E testing and non-module scripts
   */
  offlineDB: {
    // Core database operations
    openDatabase(): Promise<IDBDatabase>;
    get(storeName: string, key: string | number): Promise<any | null>;
    put(storeName: string, data: any): Promise<void>;
    remove(storeName: string, key: string | number): Promise<void>;
    getAll(storeName: string): Promise<any[]>;
    clear(storeName: string): Promise<void>;

    // Recipe-specific operations
    cacheRecipe(recipe: Recipe): Promise<void>;
    getCachedRecipe(recipeId: string): Promise<Recipe | null>;
    getAllCachedRecipes(): Promise<Recipe[]>;

    // Meal plan-specific operations
    cacheMealPlan(mealPlan: MealPlan): Promise<void>;
    getCachedMealPlan(mealPlanId: string): Promise<MealPlan | null>;
    getActiveMealPlan(): Promise<MealPlan | null>;

    // Shopping list-specific operations
    cacheShoppingList(shoppingList: ShoppingList): Promise<void>;
    getCachedShoppingList(shoppingListId: string): Promise<ShoppingList | null>;
    getCurrentShoppingList(): Promise<ShoppingList | null>;

    // Sync queue operations
    queueRequest(requestData: {
      url: string;
      method: string;
      body?: any;
      headers?: Record<string, string>;
    }): Promise<number>;
    getQueuedRequests(): Promise<SyncQueueRequest[]>;
    removeQueuedRequest(requestId: number): Promise<void>;
    clearSyncQueue(): Promise<void>;

    // Cache management
    getCacheStats(): Promise<CacheStats>;
    clearAllCache(): Promise<void>;
  };

  /**
   * Shopping list checkoff utilities
   * Exposed globally for E2E testing and non-module scripts
   */
  shoppingCheckoff: {
    initShoppingCheckoff(): void;
    clearCheckoffStates(): void;
  };
}

// Service Worker types
interface ServiceWorkerRegistration {
  sync: SyncManager;
}

interface SyncManager {
  register(tag: string): Promise<void>;
  getTags(): Promise<string[]>;
}
