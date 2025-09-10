/**
 * Data Preloading Service
 * 
 * Manages background preloading of critical data during app initialization
 * to improve perceived performance and user experience.
 * 
 * Features:
 * - Critical data identification and prioritization
 * - Background preloading during splash screen
 * - Data preloading prioritization (user preferences, recent recipes)
 * - Offline data availability verification
 * - Preloading progress tracking and user feedback
 */

import AsyncStorage from '@react-native-async-storage/async-storage';
import NetInfo from '@react-native-community/netinfo';

export interface PreloadingPriority {
  level: 'critical' | 'high' | 'normal' | 'low';
  timeout: number; // milliseconds
  retryOnFailure: boolean;
  fallbackToCache: boolean;
}

export interface DataPreloadConfig {
  dataType: string;
  source: 'api' | 'storage' | 'cache';
  url?: string;
  storageKey?: string;
  priority: PreloadingPriority;
  estimatedSize: number; // bytes
  dependencies?: string[]; // Other data types this depends on
}

export interface PreloadResult {
  dataType: string;
  success: boolean;
  data?: any;
  error?: string;
  loadTime: number;
  fromCache: boolean;
  size: number;
}

export interface PreloadingProgress {
  totalItems: number;
  completedItems: number;
  currentItem: string | null;
  progress: number; // 0-100
  estimatedTimeRemaining: number; // seconds
  errors: string[];
}

class DataPreloadingService {
  private preloadConfigs: DataPreloadConfig[] = [];
  private preloadResults = new Map<string, PreloadResult>();
  private progressCallback?: (progress: PreloadingProgress) => void;
  private isPreloading = false;
  private abortController?: AbortController;

  constructor() {
    this.initializePreloadConfigs();
  }

  /**
   * Initializes preload configurations for different data types
   */
  private initializePreloadConfigs(): void {
    this.preloadConfigs = [
      // Critical data - must load successfully
      {
        dataType: 'user_profile',
        source: 'storage',
        storageKey: 'user_profile',
        priority: {
          level: 'critical',
          timeout: 2000,
          retryOnFailure: true,
          fallbackToCache: false
        },
        estimatedSize: 2048
      },
      {
        dataType: 'app_settings',
        source: 'storage',
        storageKey: 'app_settings',
        priority: {
          level: 'critical',
          timeout: 1000,
          retryOnFailure: false,
          fallbackToCache: true
        },
        estimatedSize: 1024
      },
      {
        dataType: 'auth_token',
        source: 'storage',
        storageKey: 'auth_token',
        priority: {
          level: 'critical',
          timeout: 1500,
          retryOnFailure: true,
          fallbackToCache: false
        },
        estimatedSize: 512
      },

      // High priority data - important for user experience
      {
        dataType: 'recent_recipes',
        source: 'api',
        url: '/api/recipes/recent',
        priority: {
          level: 'high',
          timeout: 3000,
          retryOnFailure: true,
          fallbackToCache: true
        },
        estimatedSize: 15360, // ~15KB
        dependencies: ['auth_token']
      },
      {
        dataType: 'user_preferences',
        source: 'api',
        url: '/api/user/preferences',
        priority: {
          level: 'high',
          timeout: 2500,
          retryOnFailure: true,
          fallbackToCache: true
        },
        estimatedSize: 4096,
        dependencies: ['auth_token']
      },
      {
        dataType: 'meal_plan_cache',
        source: 'storage',
        storageKey: 'cached_meal_plans',
        priority: {
          level: 'high',
          timeout: 2000,
          retryOnFailure: false,
          fallbackToCache: false
        },
        estimatedSize: 20480 // ~20KB
      },

      // Normal priority data - nice to have ready
      {
        dataType: 'favorite_recipes',
        source: 'api',
        url: '/api/recipes/favorites',
        priority: {
          level: 'normal',
          timeout: 4000,
          retryOnFailure: false,
          fallbackToCache: true
        },
        estimatedSize: 12288,
        dependencies: ['auth_token']
      },
      {
        dataType: 'community_trending',
        source: 'api',
        url: '/api/community/trending',
        priority: {
          level: 'normal',
          timeout: 5000,
          retryOnFailure: false,
          fallbackToCache: true
        },
        estimatedSize: 10240
      },
      {
        dataType: 'shopping_lists',
        source: 'storage',
        storageKey: 'shopping_lists',
        priority: {
          level: 'normal',
          timeout: 1500,
          retryOnFailure: false,
          fallbackToCache: false
        },
        estimatedSize: 8192
      },

      // Low priority data - background loading
      {
        dataType: 'recipe_suggestions',
        source: 'api',
        url: '/api/recipes/suggestions',
        priority: {
          level: 'low',
          timeout: 8000,
          retryOnFailure: false,
          fallbackToCache: true
        },
        estimatedSize: 25600, // ~25KB
        dependencies: ['user_preferences']
      }
    ];
  }

  /**
   * Starts preloading critical data with progress tracking
   */
  async preloadCriticalData(progressCallback?: (progress: PreloadingProgress) => void): Promise<PreloadResult[]> {
    if (this.isPreloading) {
      console.warn('[DataPreloading] Preloading already in progress');
      return Array.from(this.preloadResults.values());
    }

    this.isPreloading = true;
    this.progressCallback = progressCallback;
    this.abortController = new AbortController();
    
    console.log('[DataPreloading] Starting critical data preloading...');

    try {
      // Check network connectivity
      const networkState = await NetInfo.fetch();
      const isOnline = networkState.isConnected && networkState.isInternetReachable;

      // Sort configs by priority and dependencies
      const sortedConfigs = this.sortConfigsByPriority();
      const totalItems = sortedConfigs.length;

      let completedItems = 0;
      const errors: string[] = [];
      const startTime = Date.now();

      for (const config of sortedConfigs) {
        if (this.abortController.signal.aborted) {
          break;
        }

        // Update progress
        this.updateProgress({
          totalItems,
          completedItems,
          currentItem: config.dataType,
          progress: (completedItems / totalItems) * 100,
          estimatedTimeRemaining: this.calculateEstimatedTime(startTime, completedItems, totalItems),
          errors
        });

        try {
          // Check if dependencies are loaded
          if (config.dependencies && !this.areDependenciesLoaded(config.dependencies)) {
            console.log(`[DataPreloading] Skipping ${config.dataType} - dependencies not ready`);
            continue;
          }

          // Skip API calls if offline and no fallback
          if (!isOnline && config.source === 'api' && !config.priority.fallbackToCache) {
            console.log(`[DataPreloading] Skipping ${config.dataType} - offline and no cache fallback`);
            continue;
          }

          const result = await this.preloadSingleItem(config, isOnline);
          this.preloadResults.set(config.dataType, result);

          if (!result.success) {
            errors.push(`${config.dataType}: ${result.error}`);
          }

        } catch (error) {
          const errorMsg = error instanceof Error ? error.message : 'Unknown error';
          errors.push(`${config.dataType}: ${errorMsg}`);
          
          console.error(`[DataPreloading] Failed to preload ${config.dataType}:`, error);
        }

        completedItems++;
      }

      // Final progress update
      this.updateProgress({
        totalItems,
        completedItems,
        currentItem: null,
        progress: 100,
        estimatedTimeRemaining: 0,
        errors
      });

      const results = Array.from(this.preloadResults.values());
      const successful = results.filter(r => r.success).length;
      const totalSize = results.reduce((sum, r) => sum + r.size, 0);

      console.log(`[DataPreloading] Preloading completed: ${successful}/${totalItems} successful, ${Math.round(totalSize / 1024)}KB loaded`);
      
      return results;

    } catch (error) {
      console.error('[DataPreloading] Critical error during preloading:', error);
      throw error;
    } finally {
      this.isPreloading = false;
    }
  }

  /**
   * Preloads a single data item
   */
  private async preloadSingleItem(config: DataPreloadConfig, isOnline: boolean): Promise<PreloadResult> {
    const startTime = Date.now();

    try {
      let data: any;
      let fromCache = false;
      let actualSize = 0;

      switch (config.source) {
        case 'storage':
          data = await this.loadFromStorage(config);
          actualSize = this.calculateDataSize(data);
          break;

        case 'cache':
          data = await this.loadFromCache(config);
          fromCache = true;
          actualSize = this.calculateDataSize(data);
          break;

        case 'api':
          if (isOnline) {
            try {
              data = await this.loadFromAPI(config);
              actualSize = this.calculateDataSize(data);
              
              // Cache successful API responses
              await this.cacheData(config.dataType, data);
            } catch (apiError) {
              // Fallback to cache if API fails and fallback is enabled
              if (config.priority.fallbackToCache) {
                console.log(`[DataPreloading] API failed for ${config.dataType}, trying cache...`);
                data = await this.loadFromCache(config);
                fromCache = true;
                actualSize = this.calculateDataSize(data);
              } else {
                throw apiError;
              }
            }
          } else if (config.priority.fallbackToCache) {
            data = await this.loadFromCache(config);
            fromCache = true;
            actualSize = this.calculateDataSize(data);
          } else {
            throw new Error('Offline and no cache fallback available');
          }
          break;

        default:
          throw new Error(`Unknown data source: ${config.source}`);
      }

      const loadTime = Date.now() - startTime;

      return {
        dataType: config.dataType,
        success: true,
        data,
        loadTime,
        fromCache,
        size: actualSize
      };

    } catch (error) {
      const loadTime = Date.now() - startTime;
      const errorMsg = error instanceof Error ? error.message : 'Unknown error';

      return {
        dataType: config.dataType,
        success: false,
        error: errorMsg,
        loadTime,
        fromCache: false,
        size: 0
      };
    }
  }

  /**
   * Loads data from AsyncStorage
   */
  private async loadFromStorage(config: DataPreloadConfig): Promise<any> {
    if (!config.storageKey) {
      throw new Error('Storage key not provided');
    }

    const data = await AsyncStorage.getItem(config.storageKey);
    return data ? JSON.parse(data) : null;
  }

  /**
   * Loads data from cache
   */
  private async loadFromCache(config: DataPreloadConfig): Promise<any> {
    const cacheKey = `cache_${config.dataType}`;
    const cached = await AsyncStorage.getItem(cacheKey);
    
    if (cached) {
      const parsedCache = JSON.parse(cached);
      
      // Check if cache is still valid (1 hour expiry)
      const cacheAge = Date.now() - parsedCache.timestamp;
      if (cacheAge < 3600000) { // 1 hour
        return parsedCache.data;
      }
    }

    return null;
  }

  /**
   * Loads data from API
   */
  private async loadFromAPI(config: DataPreloadConfig): Promise<any> {
    if (!config.url) {
      throw new Error('API URL not provided');
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), config.priority.timeout);

    try {
      const response = await this.makeAPIRequest(config.url, controller.signal);
      clearTimeout(timeoutId);
      return response;
    } catch (error) {
      clearTimeout(timeoutId);
      throw error;
    }
  }

  /**
   * Makes actual HTTP API request
   */
  private async makeAPIRequest(url: string, signal: AbortSignal): Promise<any> {
    try {
      // Get API base URL from config or environment
      const apiBaseUrl = await this.getAPIBaseUrl();
      const fullUrl = `${apiBaseUrl}${url}`;

      // Get authentication headers if available
      const headers = await this.getAuthHeaders();

      const response = await fetch(fullUrl, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          ...headers
        },
        signal
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      return data;

    } catch (error) {
      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          throw new Error('Request aborted');
        }
        throw error;
      }
      throw new Error('Unknown API error');
    }
  }

  /**
   * Gets the API base URL from configuration
   */
  private async getAPIBaseUrl(): Promise<string> {
    try {
      // Try to get from stored configuration first
      const storedConfig = await AsyncStorage.getItem('api_config');
      if (storedConfig) {
        const config = JSON.parse(storedConfig);
        if (config.baseUrl) {
          return config.baseUrl;
        }
      }

      // Fall back to environment or default
      return process.env.REACT_APP_API_BASE_URL || 'https://api.imkitchen.app';
    } catch (error) {
      console.warn('[DataPreloading] Failed to get API base URL:', error);
      return 'https://api.imkitchen.app';
    }
  }

  /**
   * Gets authentication headers for API requests
   */
  private async getAuthHeaders(): Promise<Record<string, string>> {
    try {
      const authToken = await AsyncStorage.getItem('auth_token');
      if (authToken) {
        return {
          'Authorization': `Bearer ${authToken}`
        };
      }
      return {};
    } catch (error) {
      console.warn('[DataPreloading] Failed to get auth headers:', error);
      return {};
    }
  }

  /**
   * Caches data for future use
   */
  private async cacheData(dataType: string, data: any): Promise<void> {
    const cacheKey = `cache_${dataType}`;
    const cacheEntry = {
      data,
      timestamp: Date.now()
    };

    try {
      await AsyncStorage.setItem(cacheKey, JSON.stringify(cacheEntry));
    } catch (error) {
      console.warn(`[DataPreloading] Failed to cache ${dataType}:`, error);
    }
  }

  /**
   * Calculates estimated data size in bytes
   */
  private calculateDataSize(data: any): number {
    if (!data) return 0;
    return new Blob([JSON.stringify(data)]).size;
  }

  /**
   * Sorts configs by priority and dependency order
   */
  private sortConfigsByPriority(): DataPreloadConfig[] {
    const priorityOrder = { critical: 4, high: 3, normal: 2, low: 1 };
    
    return [...this.preloadConfigs].sort((a, b) => {
      // First sort by priority
      const priorityDiff = priorityOrder[b.priority.level] - priorityOrder[a.priority.level];
      if (priorityDiff !== 0) return priorityDiff;

      // Then by dependency count (fewer dependencies first)
      const aDeps = a.dependencies?.length || 0;
      const bDeps = b.dependencies?.length || 0;
      return aDeps - bDeps;
    });
  }

  /**
   * Checks if all dependencies for a config are loaded
   */
  private areDependenciesLoaded(dependencies: string[]): boolean {
    return dependencies.every(dep => {
      const result = this.preloadResults.get(dep);
      return result && result.success;
    });
  }

  /**
   * Calculates estimated time remaining
   */
  private calculateEstimatedTime(startTime: number, completed: number, total: number): number {
    if (completed === 0) return 0;
    
    const elapsed = Date.now() - startTime;
    const avgTimePerItem = elapsed / completed;
    const remaining = total - completed;
    
    return Math.round((remaining * avgTimePerItem) / 1000); // Convert to seconds
  }

  /**
   * Updates progress and calls callback
   */
  private updateProgress(progress: PreloadingProgress): void {
    this.progressCallback?.(progress);
  }

  /**
   * Gets preloaded data by type
   */
  getPreloadedData(dataType: string): any {
    const result = this.preloadResults.get(dataType);
    return result?.success ? result.data : null;
  }

  /**
   * Checks if specific data is available
   */
  isDataAvailable(dataType: string): boolean {
    const result = this.preloadResults.get(dataType);
    return result ? result.success : false;
  }

  /**
   * Gets preloading results summary
   */
  getPreloadingResults(): {
    totalItems: number;
    successfulItems: number;
    failedItems: number;
    totalSize: number;
    totalTime: number;
    cacheHitRate: number;
  } {
    const results = Array.from(this.preloadResults.values());
    const successful = results.filter(r => r.success);
    const failed = results.filter(r => !r.success);
    const fromCache = results.filter(r => r.fromCache);
    
    return {
      totalItems: results.length,
      successfulItems: successful.length,
      failedItems: failed.length,
      totalSize: results.reduce((sum, r) => sum + r.size, 0),
      totalTime: results.reduce((sum, r) => sum + r.loadTime, 0),
      cacheHitRate: results.length > 0 ? fromCache.length / results.length : 0
    };
  }

  /**
   * Verifies offline data availability
   */
  async verifyOfflineAvailability(): Promise<{
    available: string[];
    missing: string[];
    totalOfflineSize: number;
  }> {
    const available: string[] = [];
    const missing: string[] = [];
    let totalSize = 0;

    for (const config of this.preloadConfigs) {
      try {
        let data;
        
        if (config.source === 'storage' && config.storageKey) {
          data = await AsyncStorage.getItem(config.storageKey);
        } else {
          // Check cache for API data
          data = await this.loadFromCache(config);
        }

        if (data) {
          available.push(config.dataType);
          totalSize += this.calculateDataSize(data);
        } else {
          missing.push(config.dataType);
        }
      } catch (error) {
        missing.push(config.dataType);
      }
    }

    return { available, missing, totalOfflineSize: totalSize };
  }

  /**
   * Clears preloaded data and cache
   */
  async clearPreloadedData(): Promise<void> {
    this.preloadResults.clear();

    // Clear cached data
    const keys = await AsyncStorage.getAllKeys();
    const cacheKeys = keys.filter(key => key.startsWith('cache_'));
    
    if (cacheKeys.length > 0) {
      await AsyncStorage.multiRemove(cacheKeys);
    }

    console.log('[DataPreloading] Preloaded data and cache cleared');
  }

  /**
   * Cancels ongoing preloading
   */
  cancelPreloading(): void {
    if (this.isPreloading && this.abortController) {
      this.abortController.abort();
      console.log('[DataPreloading] Preloading cancelled');
    }
  }

  /**
   * Gets current preloading status
   */
  getStatus(): {
    isPreloading: boolean;
    completedItems: number;
    totalItems: number;
  } {
    return {
      isPreloading: this.isPreloading,
      completedItems: this.preloadResults.size,
      totalItems: this.preloadConfigs.length
    };
  }
}

// Export singleton instance
export const dataPreloadingService = new DataPreloadingService();
export default DataPreloadingService;