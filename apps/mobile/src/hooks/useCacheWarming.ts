import { useEffect, useCallback } from 'react';
import { AppState, AppStateStatus } from 'react-native';
import { useRecipeStore } from '../store/recipe_store';

export interface CacheWarmingConfig {
  enabled: boolean;
  onAppForeground: boolean;
  intervalMinutes: number;
  maxRecipesToWarm: number;
}

const DEFAULT_CONFIG: CacheWarmingConfig = {
  enabled: true,
  onAppForeground: true,
  intervalMinutes: 30,
  maxRecipesToWarm: 20,
};

export const useCacheWarming = (config: Partial<CacheWarmingConfig> = {}) => {
  const effectiveConfig = { ...DEFAULT_CONFIG, ...config };
  const { warmCache, isOffline, networkStatus } = useRecipeStore();

  // Warm cache with most frequently accessed recipes
  const warmFrequentRecipes = useCallback(async () => {
    if (!effectiveConfig.enabled || isOffline()) {
      return;
    }

    try {
      console.log('Starting cache warming for frequently accessed recipes...');
      await warmCache();
      console.log('Cache warming completed');
    } catch (error) {
      console.error('Cache warming failed:', error);
    }
  }, [effectiveConfig.enabled, isOffline, warmCache]);

  // Warm cache with specific recipe IDs
  const warmSpecificRecipes = useCallback(async (recipeIds: string[]) => {
    if (!effectiveConfig.enabled || isOffline()) {
      return;
    }

    try {
      console.log(`Starting cache warming for ${recipeIds.length} specific recipes...`);
      const limitedIds = recipeIds.slice(0, effectiveConfig.maxRecipesToWarm);
      await warmCache(limitedIds);
      console.log('Specific recipe cache warming completed');
    } catch (error) {
      console.error('Specific recipe cache warming failed:', error);
    }
  }, [effectiveConfig.enabled, effectiveConfig.maxRecipesToWarm, isOffline, warmCache]);

  // Handle app state changes (foreground/background)
  useEffect(() => {
    if (!effectiveConfig.onAppForeground) {
      return;
    }

    const handleAppStateChange = (nextAppState: AppStateStatus) => {
      if (nextAppState === 'active') {
        // App came to foreground, warm the cache
        warmFrequentRecipes();
      }
    };

    const subscription = AppState.addEventListener('change', handleAppStateChange);
    
    return () => subscription?.remove();
  }, [effectiveConfig.onAppForeground, warmFrequentRecipes]);

  // Periodic cache warming
  useEffect(() => {
    if (!effectiveConfig.enabled || effectiveConfig.intervalMinutes <= 0) {
      return;
    }

    const interval = setInterval(() => {
      warmFrequentRecipes();
    }, effectiveConfig.intervalMinutes * 60 * 1000);

    return () => clearInterval(interval);
  }, [effectiveConfig.enabled, effectiveConfig.intervalMinutes, warmFrequentRecipes]);

  // Network status change cache warming
  useEffect(() => {
    // When we come back online, warm the cache
    if (networkStatus.isConnected && networkStatus.isInternetReachable !== false) {
      const timer = setTimeout(() => {
        warmFrequentRecipes();
      }, 2000); // Wait 2 seconds to ensure network is stable

      return () => clearTimeout(timer);
    }
  }, [networkStatus.isConnected, networkStatus.isInternetReachable, warmFrequentRecipes]);

  return {
    warmFrequentRecipes,
    warmSpecificRecipes,
    config: effectiveConfig,
  };
};