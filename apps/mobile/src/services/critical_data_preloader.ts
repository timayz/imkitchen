/**
 * Critical Data Preloader
 * 
 * Intelligently preloads essential app data during startup to minimize
 * time to interactive and improve user experience.
 * 
 * Features:
 * - Priority-based data loading with dependency management
 * - Parallel and sequential loading strategies
 * - Fallback mechanisms and error handling
 * - Cache-first approach with network fallback
 * - Progressive loading with partial success handling
 * - Performance metrics and optimization
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import NetInfo from '@react-native-netinfo/';
import { recipeCacheService } from './recipe_cache_service';
import { imageCacheService } from './image_cache_service';

export interface PreloadItem {
  id: string;
  name: string;
  priority: 'critical' | 'high' | 'normal' | 'low';
  dependencies: string[];
  loader: () => Promise<any>;
  cacheKey?: string;
  maxAge?: number; // milliseconds
  fallback?: () => Promise<any>;
  timeout?: number; // milliseconds
}

export interface PreloadProgress {
  itemId: string;
  status: 'pending' | 'loading' | 'completed' | 'error' | 'timeout';
  progress: number;
  startTime: number;
  endTime?: number;
  error?: string;
  data?: any;
}

export interface PreloadStrategy {
  maxConcurrency: number;
  timeoutMs: number;
  retryAttempts: number;
  retryDelayMs: number;
  fallbackToCache: boolean;
  offlineMode: boolean;
}

class CriticalDataPreloader {
  private preloadItems: Map<string, PreloadItem> = new Map();
  private preloadProgress: Map<string, PreloadProgress> = new Map();
  private loadedData: Map<string, any> = new Map();
  private strategy: PreloadStrategy;
  private isNetworkAvailable = true;

  constructor() {
    this.strategy = {
      maxConcurrency: 3,
      timeoutMs: 5000,
      retryAttempts: 2,
      retryDelayMs: 1000,
      fallbackToCache: true,
      offlineMode: false
    };

    this.initializeNetworkMonitoring();
    this.registerCriticalDataItems();
  }

  private async initializeNetworkMonitoring() {
    const netInfo = await NetInfo.fetch();
    this.isNetworkAvailable = netInfo.isConnected ?? false;

    NetInfo.addEventListener(state => {
      const wasOffline = !this.isNetworkAvailable;
      this.isNetworkAvailable = state.isConnected ?? false;
      
      if (wasOffline && this.isNetworkAvailable) {
        console.log('[CriticalDataPreloader] Network restored, triggering refresh');
        this.refreshStaleData();
      }
    });
  }

  private registerCriticalDataItems() {
    // User Session & Authentication
    this.register({
      id: 'user_session',
      name: 'User Session',
      priority: 'critical',
      dependencies: [],
      loader: this.loadUserSession.bind(this),
      cacheKey: 'user_session',
      maxAge: 60 * 60 * 1000, // 1 hour
      timeout: 3000
    });

    // User Preferences & Settings
    this.register({
      id: 'user_preferences',
      name: 'User Preferences',
      priority: 'critical',
      dependencies: ['user_session'],
      loader: this.loadUserPreferences.bind(this),
      cacheKey: 'user_preferences',
      maxAge: 30 * 60 * 1000, // 30 minutes
      fallback: this.loadDefaultPreferences.bind(this)
    });

    // Recent Recipes (First Screen Content)
    this.register({
      id: 'recent_recipes',
      name: 'Recent Recipes',
      priority: 'high',
      dependencies: ['user_preferences'],
      loader: this.loadRecentRecipes.bind(this),
      cacheKey: 'recent_recipes',
      maxAge: 15 * 60 * 1000, // 15 minutes
    });

    // Active Meal Plan
    this.register({
      id: 'active_meal_plan',
      name: 'Active Meal Plan',
      priority: 'high',
      dependencies: ['user_session'],
      loader: this.loadActiveMealPlan.bind(this),
      cacheKey: 'active_meal_plan',
      maxAge: 10 * 60 * 1000, // 10 minutes
    });

    // Recipe Categories & Tags
    this.register({
      id: 'recipe_metadata',
      name: 'Recipe Categories',
      priority: 'normal',
      dependencies: [],
      loader: this.loadRecipeMetadata.bind(this),
      cacheKey: 'recipe_metadata',
      maxAge: 60 * 60 * 1000, // 1 hour
    });

    // Shopping List
    this.register({
      id: 'shopping_list',
      name: 'Shopping List',
      priority: 'normal',
      dependencies: ['active_meal_plan'],
      loader: this.loadShoppingList.bind(this),
      cacheKey: 'shopping_list',
      maxAge: 30 * 60 * 1000, // 30 minutes
    });

    // Notification Settings & Badges
    this.register({
      id: 'notifications',
      name: 'Notifications',
      priority: 'low',
      dependencies: ['user_session'],
      loader: this.loadNotifications.bind(this),
      cacheKey: 'notifications',
      maxAge: 5 * 60 * 1000, // 5 minutes
    });

    // App Configuration & Feature Flags
    this.register({
      id: 'app_config',
      name: 'App Configuration',
      priority: 'normal',
      dependencies: [],
      loader: this.loadAppConfiguration.bind(this),
      cacheKey: 'app_config',
      maxAge: 4 * 60 * 60 * 1000, // 4 hours
    });
  }

  /**
   * Registers a preload item
   */
  register(item: PreloadItem): void {
    this.preloadItems.set(item.id, item);
    this.preloadProgress.set(item.id, {
      itemId: item.id,
      status: 'pending',
      progress: 0,
      startTime: 0
    });
  }

  /**
   * Starts the preloading process with intelligent priority handling
   */
  async preloadCriticalData(): Promise<{
    success: boolean;
    completedItems: string[];
    failedItems: Array<{ id: string; error: string }>;
    totalTime: number;
  }> {
    const startTime = Date.now();
    console.log('[CriticalDataPreloader] Starting critical data preload...');

    // Update strategy based on network conditions
    this.adjustStrategyForNetwork();

    const items = Array.from(this.preloadItems.values());
    const priorityGroups = this.groupItemsByPriority(items);

    const completedItems: string[] = [];
    const failedItems: Array<{ id: string; error: string }> = [];

    try {
      // Load critical items first (blocking)
      if (priorityGroups.critical.length > 0) {
        const criticalResults = await this.loadItemsConcurrently(priorityGroups.critical);
        completedItems.push(...criticalResults.completed);
        failedItems.push(...criticalResults.failed);
      }

      // Load high priority items (blocking)
      if (priorityGroups.high.length > 0) {
        const highResults = await this.loadItemsConcurrently(priorityGroups.high);
        completedItems.push(...highResults.completed);
        failedItems.push(...highResults.failed);
      }

      // Load normal and low priority items in background (non-blocking)
      if (priorityGroups.normal.length > 0 || priorityGroups.low.length > 0) {
        const backgroundItems = [...priorityGroups.normal, ...priorityGroups.low];
        this.loadItemsInBackground(backgroundItems);
      }

    } catch (error) {
      console.error('[CriticalDataPreloader] Critical loading failed:', error);
    }

    const totalTime = Date.now() - startTime;
    const success = failedItems.length === 0 || completedItems.length > 0;

    console.log(`[CriticalDataPreloader] Preload completed in ${totalTime}ms`);
    console.log(`[CriticalDataPreloader] Success: ${completedItems.length}, Failed: ${failedItems.length}`);

    return { success, completedItems, failedItems, totalTime };
  }

  private adjustStrategyForNetwork() {
    if (!this.isNetworkAvailable) {
      this.strategy.offlineMode = true;
      this.strategy.fallbackToCache = true;
      this.strategy.timeoutMs = 1000; // Quick timeout for offline mode
      console.log('[CriticalDataPreloader] Adjusted strategy for offline mode');
    } else {
      // Check network type and adjust accordingly
      // For now, use default online strategy
      this.strategy.offlineMode = false;
      this.strategy.timeoutMs = 5000;
    }
  }

  private groupItemsByPriority(items: PreloadItem[]): {
    critical: PreloadItem[];
    high: PreloadItem[];
    normal: PreloadItem[];
    low: PreloadItem[];
  } {
    return items.reduce((groups, item) => {
      groups[item.priority].push(item);
      return groups;
    }, {
      critical: [] as PreloadItem[],
      high: [] as PreloadItem[],
      normal: [] as PreloadItem[],
      low: [] as PreloadItem[]
    });
  }

  private async loadItemsConcurrently(items: PreloadItem[]): Promise<{
    completed: string[];
    failed: Array<{ id: string; error: string }>;
  }> {
    const sortedItems = this.sortItemsByDependencies(items);
    const completed: string[] = [];
    const failed: Array<{ id: string; error: string }> = [];

    // Process items respecting dependencies and concurrency limits
    const processing = new Set<string>();
    const results = new Map<string, any>();

    while (sortedItems.length > 0 || processing.size > 0) {
      // Find items ready to process
      const readyItems = sortedItems.filter(item => 
        item.dependencies.every(dep => results.has(dep) || failed.some(f => f.id === dep))
      );

      // Start processing up to maxConcurrency items
      const itemsToProcess = readyItems.slice(0, this.strategy.maxConcurrency - processing.size);
      
      for (const item of itemsToProcess) {
        processing.add(item.id);
        sortedItems.splice(sortedItems.indexOf(item), 1);
        
        this.loadSingleItem(item).then(result => {
          processing.delete(item.id);
          if (result.success) {
            results.set(item.id, result.data);
            completed.push(item.id);
          } else {
            failed.push({ id: item.id, error: result.error });
          }
        });
      }

      if (processing.size === 0 && readyItems.length === 0) {
        break; // No more items to process or circular dependencies
      }

      // Wait a bit before checking again
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    return { completed, failed };
  }

  private async loadItemsInBackground(items: PreloadItem[]): Promise<void> {
    console.log(`[CriticalDataPreloader] Loading ${items.length} items in background`);
    
    // Fire and forget background loading
    const backgroundPromises = items.map(item => 
      this.loadSingleItem(item).catch(error => 
        console.warn(`[CriticalDataPreloader] Background load failed for ${item.id}:`, error)
      )
    );

    Promise.allSettled(backgroundPromises);
  }

  private async loadSingleItem(item: PreloadItem): Promise<{
    success: boolean;
    data?: any;
    error: string;
  }> {
    const progress = this.preloadProgress.get(item.id)!;
    progress.status = 'loading';
    progress.startTime = Date.now();

    try {
      // Check cache first if offline or fallback enabled
      if (this.strategy.fallbackToCache || this.strategy.offlineMode) {
        const cachedData = await this.getCachedData(item);
        if (cachedData) {
          progress.status = 'completed';
          progress.endTime = Date.now();
          progress.progress = 100;
          this.loadedData.set(item.id, cachedData);
          console.log(`[CriticalDataPreloader] Loaded ${item.name} from cache`);
          return { success: true, data: cachedData, error: '' };
        }
      }

      // Attempt network load with timeout and retries
      const data = await this.loadWithTimeout(item);
      
      // Cache the loaded data
      if (item.cacheKey) {
        await this.cacheData(item.cacheKey, data, item.maxAge);
      }

      progress.status = 'completed';
      progress.endTime = Date.now();
      progress.progress = 100;
      progress.data = data;
      this.loadedData.set(item.id, data);

      console.log(`[CriticalDataPreloader] Loaded ${item.name} successfully`);
      return { success: true, data, error: '' };

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      
      // Try fallback if available
      if (item.fallback) {
        try {
          const fallbackData = await item.fallback();
          progress.status = 'completed';
          progress.endTime = Date.now();
          progress.progress = 100;
          this.loadedData.set(item.id, fallbackData);
          
          console.log(`[CriticalDataPreloader] Loaded ${item.name} from fallback`);
          return { success: true, data: fallbackData, error: '' };
        } catch (fallbackError) {
          console.error(`[CriticalDataPreloader] Fallback failed for ${item.name}:`, fallbackError);
        }
      }

      progress.status = 'error';
      progress.endTime = Date.now();
      progress.error = errorMessage;

      console.error(`[CriticalDataPreloader] Failed to load ${item.name}:`, error);
      return { success: false, error: errorMessage };
    }
  }

  private async loadWithTimeout(item: PreloadItem): Promise<any> {
    const timeout = item.timeout || this.strategy.timeoutMs;
    
    return Promise.race([
      item.loader(),
      new Promise((_, reject) => 
        setTimeout(() => reject(new Error('Timeout')), timeout)
      )
    ]);
  }

  private sortItemsByDependencies(items: PreloadItem[]): PreloadItem[] {
    // Simple topological sort for dependencies
    const sorted: PreloadItem[] = [];
    const visited = new Set<string>();
    const visiting = new Set<string>();

    const visit = (item: PreloadItem) => {
      if (visiting.has(item.id)) {
        throw new Error(`Circular dependency detected: ${item.id}`);
      }
      
      if (!visited.has(item.id)) {
        visiting.add(item.id);
        
        for (const depId of item.dependencies) {
          const depItem = items.find(i => i.id === depId);
          if (depItem) {
            visit(depItem);
          }
        }
        
        visiting.delete(item.id);
        visited.add(item.id);
        sorted.push(item);
      }
    };

    for (const item of items) {
      visit(item);
    }

    return sorted;
  }

  private async getCachedData(item: PreloadItem): Promise<any> {
    if (!item.cacheKey) return null;

    try {
      const cachedStr = await AsyncStorage.getItem(`preload_${item.cacheKey}`);
      if (!cachedStr) return null;

      const cached = JSON.parse(cachedStr);
      const age = Date.now() - cached.timestamp;
      
      if (item.maxAge && age > item.maxAge) {
        return null; // Expired
      }

      return cached.data;
    } catch (error) {
      console.warn(`[CriticalDataPreloader] Cache read error for ${item.id}:`, error);
      return null;
    }
  }

  private async cacheData(key: string, data: any, maxAge?: number): Promise<void> {
    try {
      const cacheEntry = {
        data,
        timestamp: Date.now(),
        maxAge: maxAge || 0
      };
      
      await AsyncStorage.setItem(`preload_${key}`, JSON.stringify(cacheEntry));
    } catch (error) {
      console.warn(`[CriticalDataPreloader] Cache write error for ${key}:`, error);
    }
  }

  // Data loader implementations
  private async loadUserSession(): Promise<any> {
    // Mock implementation - replace with actual API call
    await new Promise(resolve => setTimeout(resolve, 300));
    return {
      userId: '123',
      email: 'user@example.com',
      isAuthenticated: true,
      token: 'mock-token'
    };
  }

  private async loadUserPreferences(): Promise<any> {
    // Mock implementation
    await new Promise(resolve => setTimeout(resolve, 200));
    return {
      dietaryRestrictions: ['vegetarian'],
      cuisinePreferences: ['italian', 'mediterranean'],
      mealPlanDuration: 7,
      servingSize: 4
    };
  }

  private async loadDefaultPreferences(): Promise<any> {
    return {
      dietaryRestrictions: [],
      cuisinePreferences: [],
      mealPlanDuration: 7,
      servingSize: 2
    };
  }

  private async loadRecentRecipes(): Promise<any> {
    // Try to get from recipe cache service first
    try {
      // This would use the actual recipe cache service
      await new Promise(resolve => setTimeout(resolve, 400));
      return [
        { id: 1, title: 'Spaghetti Carbonara', image: 'pasta.jpg' },
        { id: 2, title: 'Chicken Stir Fry', image: 'stirfry.jpg' },
        { id: 3, title: 'Caesar Salad', image: 'salad.jpg' }
      ];
    } catch (error) {
      console.warn('[CriticalDataPreloader] Recent recipes API failed:', error);
      return [];
    }
  }

  private async loadActiveMealPlan(): Promise<any> {
    await new Promise(resolve => setTimeout(resolve, 350));
    return {
      id: 'plan-123',
      weekStart: new Date().toISOString(),
      meals: [
        { day: 'monday', breakfast: 'Oatmeal', lunch: 'Sandwich', dinner: 'Pasta' },
        { day: 'tuesday', breakfast: 'Toast', lunch: 'Salad', dinner: 'Chicken' }
      ]
    };
  }

  private async loadRecipeMetadata(): Promise<any> {
    await new Promise(resolve => setTimeout(resolve, 250));
    return {
      categories: ['breakfast', 'lunch', 'dinner', 'snack'],
      cuisines: ['italian', 'mexican', 'asian', 'american'],
      difficulties: ['easy', 'medium', 'hard']
    };
  }

  private async loadShoppingList(): Promise<any> {
    await new Promise(resolve => setTimeout(resolve, 200));
    return {
      id: 'list-123',
      items: [
        { name: 'Eggs', quantity: 12, unit: 'pieces', checked: false },
        { name: 'Milk', quantity: 1, unit: 'liter', checked: true }
      ]
    };
  }

  private async loadNotifications(): Promise<any> {
    await new Promise(resolve => setTimeout(resolve, 150));
    return {
      unreadCount: 3,
      notifications: [
        { id: 1, message: 'Your meal plan is ready!', timestamp: new Date().toISOString() }
      ]
    };
  }

  private async loadAppConfiguration(): Promise<any> {
    await new Promise(resolve => setTimeout(resolve, 300));
    return {
      features: {
        communityRecipes: true,
        premiumFeatures: false,
        analytics: true
      },
      version: '1.0.0',
      apiEndpoints: {
        recipes: 'https://api.imkitchen.com/recipes',
        mealPlans: 'https://api.imkitchen.com/meal-plans'
      }
    };
  }

  private async refreshStaleData(): Promise<void> {
    const staleItems = Array.from(this.preloadItems.values()).filter(item => {
      if (!item.cacheKey || !item.maxAge) return false;
      
      const cached = this.loadedData.get(item.id);
      if (!cached) return true;
      
      // Check if data is stale (simplified check)
      return Math.random() > 0.7; // 30% chance to refresh
    });

    if (staleItems.length > 0) {
      console.log(`[CriticalDataPreloader] Refreshing ${staleItems.length} stale items`);
      this.loadItemsInBackground(staleItems);
    }
  }

  /**
   * Gets preloaded data by ID
   */
  getData(itemId: string): any {
    return this.loadedData.get(itemId);
  }

  /**
   * Gets loading progress for all items
   */
  getProgress(): PreloadProgress[] {
    return Array.from(this.preloadProgress.values());
  }

  /**
   * Gets overall loading statistics
   */
  getStatistics(): {
    totalItems: number;
    completedItems: number;
    failedItems: number;
    averageLoadTime: number;
    cacheHitRate: number;
  } {
    const progress = this.getProgress();
    const completed = progress.filter(p => p.status === 'completed');
    const failed = progress.filter(p => p.status === 'error');
    
    const totalLoadTime = completed.reduce((sum, p) => 
      sum + ((p.endTime || 0) - p.startTime), 0
    );
    
    return {
      totalItems: progress.length,
      completedItems: completed.length,
      failedItems: failed.length,
      averageLoadTime: completed.length > 0 ? totalLoadTime / completed.length : 0,
      cacheHitRate: 0.85 // Mock cache hit rate
    };
  }
}

// Export singleton instance
export const criticalDataPreloader = new CriticalDataPreloader();
export default CriticalDataPreloader;