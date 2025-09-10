/**
 * Screen Registry
 * 
 * Central registry for all app screens with lazy loading configuration,
 * bundle size estimation, and intelligent preloading strategies.
 * 
 * Features:
 * - Screen metadata with bundle size estimates
 * - Priority-based loading strategies
 * - Dependency tracking for nested components
 * - Navigation pattern learning
 * - Performance monitoring integration
 */

import { ComponentType } from 'react';
import { lazyLoadingService, ScreenMetadata, LazyLoadOptions } from '../services/lazy_loading_service';

// Screen priority definitions
export enum ScreenPriority {
  CRITICAL = 'critical',    // Must load immediately (auth, main)
  HIGH = 'high',           // Load during splash screen
  NORMAL = 'normal',       // Load on navigation
  LOW = 'low'              // Load only when needed
}

// Screen categories for bundle optimization
export enum ScreenCategory {
  AUTH = 'auth',
  RECIPES = 'recipes', 
  MEAL_PLANS = 'meal-plans',
  PREFERENCES = 'preferences',
  ANALYTICS = 'analytics',
  SHOPPING = 'shopping',
  COMMUNITY = 'community',
  PROFILE = 'profile'
}

export interface ScreenRegistryEntry {
  name: string;
  route: string;
  category: ScreenCategory;
  priority: ScreenPriority;
  estimatedSize: number; // Bundle size in bytes
  dependencies: string[];
  importFactory: () => Promise<{ default: ComponentType<any> }>;
  preloadDependencies?: string[]; // Screens to preload when this screen loads
}

class ScreenRegistry {
  private screens = new Map<string, ScreenRegistryEntry>();
  private lazyScreens = new Map<string, ComponentType<any>>();
  private initializationPromise: Promise<void> | null = null;

  constructor() {
    this.registerAllScreens();
  }

  /**
   * Registers all application screens with lazy loading configuration
   */
  private registerAllScreens(): void {
    // Authentication Screens (Critical Priority)
    this.register({
      name: 'LoginScreen',
      route: 'Login',
      category: ScreenCategory.AUTH,
      priority: ScreenPriority.CRITICAL,
      estimatedSize: 45000, // ~45KB
      dependencies: ['react-hook-form', 'yup'],
      importFactory: () => import('../screens/auth/LoginScreen'),
    });

    this.register({
      name: 'RegisterScreen',
      route: 'Register', 
      category: ScreenCategory.AUTH,
      priority: ScreenPriority.HIGH,
      estimatedSize: 52000, // ~52KB
      dependencies: ['react-hook-form', 'yup'],
      importFactory: () => import('../screens/auth/RegisterScreen'),
    });

    // Recipe Management Screens (High Priority - Core Feature)
    this.register({
      name: 'RecipeListScreen',
      route: 'RecipeList',
      category: ScreenCategory.RECIPES,
      priority: ScreenPriority.HIGH,
      estimatedSize: 78000, // ~78KB
      dependencies: ['recipe_cache_service', 'offline_recipe_repository'],
      importFactory: () => import('../screens/recipes/RecipeListScreen'),
      preloadDependencies: ['RecipeDetailScreen']
    });

    this.register({
      name: 'RecipeDetailScreen',
      route: 'RecipeDetail',
      category: ScreenCategory.RECIPES,
      priority: ScreenPriority.HIGH,
      estimatedSize: 65000, // ~65KB  
      dependencies: ['optimized_recipe_image', 'lazy_image'],
      importFactory: () => import('../screens/recipes/RecipeDetailScreen'),
    });

    this.register({
      name: 'AddRecipeScreen',
      route: 'AddRecipe',
      category: ScreenCategory.RECIPES,
      priority: ScreenPriority.NORMAL,
      estimatedSize: 95000, // ~95KB (form heavy)
      dependencies: ['react-hook-form', 'image_cache_service'],
      importFactory: () => import('../screens/recipes/AddRecipeScreen'),
    });

    // Meal Planning Screens (High Priority - Core Feature)
    this.register({
      name: 'MealPlanScreen',
      route: 'MealPlan',
      category: ScreenCategory.MEAL_PLANS,
      priority: ScreenPriority.HIGH,
      estimatedSize: 120000, // ~120KB (complex UI)
      dependencies: ['meal_plan_cache_service', 'optimized_rotation_service'],
      importFactory: () => import('../screens/meal-plans/MealPlanScreen'),
      preloadDependencies: ['ShoppingListScreen']
    });

    // Shopping Lists (Normal Priority)
    this.register({
      name: 'ShoppingListScreen',
      route: 'ShoppingList',
      category: ScreenCategory.SHOPPING,
      priority: ScreenPriority.NORMAL,
      estimatedSize: 58000, // ~58KB
      dependencies: [],
      importFactory: () => import('../screens/shopping/ShoppingListScreen'),
    });

    // Community Features (Normal Priority)
    this.register({
      name: 'CommunityRecipesScreen',
      route: 'CommunityRecipes',
      category: ScreenCategory.COMMUNITY,
      priority: ScreenPriority.NORMAL,
      estimatedSize: 82000, // ~82KB
      dependencies: ['advanced_recipe_search_service'],
      importFactory: () => import('../screens/recipes/CommunityRecipesScreen'),
    });

    this.register({
      name: 'ImportRecipeScreen',
      route: 'ImportRecipe',
      category: ScreenCategory.COMMUNITY,
      priority: ScreenPriority.NORMAL,
      estimatedSize: 67000, // ~67KB
      dependencies: [],
      importFactory: () => import('../screens/recipes/ImportRecipeScreen'),
    });

    // Profile & Settings (Low Priority)
    this.register({
      name: 'ProfileScreen',
      route: 'Profile',
      category: ScreenCategory.PROFILE,
      priority: ScreenPriority.LOW,
      estimatedSize: 48000, // ~48KB
      dependencies: [],
      importFactory: () => import('../screens/auth/ProfileScreen'),
    });

    this.register({
      name: 'AccountSettingsScreen',
      route: 'AccountSettings',
      category: ScreenCategory.PROFILE,
      priority: ScreenPriority.LOW,
      estimatedSize: 55000, // ~55KB
      dependencies: ['react-hook-form'],
      importFactory: () => import('../screens/auth/AccountSettingsScreen'),
    });

    this.register({
      name: 'PreferenceSettingsScreen', 
      route: 'PreferenceSettings',
      category: ScreenCategory.PREFERENCES,
      priority: ScreenPriority.LOW,
      estimatedSize: 72000, // ~72KB
      dependencies: [],
      importFactory: () => import('../screens/PreferenceSettingsScreen'),
    });

    this.register({
      name: 'WeeklyPatternsScreen',
      route: 'WeeklyPatterns', 
      category: ScreenCategory.PREFERENCES,
      priority: ScreenPriority.LOW,
      estimatedSize: 61000, // ~61KB
      dependencies: [],
      importFactory: () => import('../screens/preferences/WeeklyPatternsScreen'),
    });

    // Secondary Features (Low Priority)
    this.register({
      name: 'FavoritesScreen',
      route: 'Favorites',
      category: ScreenCategory.RECIPES,
      priority: ScreenPriority.LOW,
      estimatedSize: 54000, // ~54KB
      dependencies: [],
      importFactory: () => import('../screens/favorites/FavoritesScreen'),
    });

    this.register({
      name: 'ForgotPasswordScreen',
      route: 'ForgotPassword',
      category: ScreenCategory.AUTH,
      priority: ScreenPriority.LOW,
      estimatedSize: 38000, // ~38KB
      dependencies: ['react-hook-form'],
      importFactory: () => import('../screens/auth/ForgotPasswordScreen'),
    });

    this.register({
      name: 'RotationStatsScreen',
      route: 'RotationStats',
      category: ScreenCategory.ANALYTICS,
      priority: ScreenPriority.LOW,
      estimatedSize: 89000, // ~89KB (charts and graphs)
      dependencies: ['react-native-chart-kit'],
      importFactory: () => import('../screens/analytics/RotationStatsScreen'),
    });

    this.register({
      name: 'PersonalRatingsScreen',
      route: 'PersonalRatings',
      category: ScreenCategory.PROFILE,
      priority: ScreenPriority.LOW,
      estimatedSize: 49000, // ~49KB
      dependencies: [],
      importFactory: () => import('../screens/profile/PersonalRatingsScreen'),
    });
  }

  /**
   * Registers a screen with the registry
   */
  private register(entry: ScreenRegistryEntry): void {
    this.screens.set(entry.name, entry);
  }

  /**
   * Gets a lazy-loaded screen component
   */
  getScreen(screenName: string): ComponentType<any> {
    if (this.lazyScreens.has(screenName)) {
      return this.lazyScreens.get(screenName)!;
    }

    const entry = this.screens.get(screenName);
    if (!entry) {
      throw new Error(`Screen '${screenName}' not found in registry`);
    }

    const lazyOptions: LazyLoadOptions = {
      priority: entry.priority,
      cacheStrategy: entry.priority === ScreenPriority.CRITICAL ? 'memory' : 'storage',
      errorBoundary: true,
      preloadTimeout: this.getPreloadTimeout(entry.priority)
    };

    const lazyScreen = lazyLoadingService.createLazyScreen(
      screenName,
      entry.importFactory,
      lazyOptions
    );

    this.lazyScreens.set(screenName, lazyScreen);
    return lazyScreen;
  }

  private getPreloadTimeout(priority: ScreenPriority): number {
    switch (priority) {
      case ScreenPriority.CRITICAL: return 100;
      case ScreenPriority.HIGH: return 500;
      case ScreenPriority.NORMAL: return 1000;
      case ScreenPriority.LOW: return 2000;
    }
  }

  /**
   * Initializes screen registry and preloads critical screens
   */
  async initialize(): Promise<void> {
    if (this.initializationPromise) {
      return this.initializationPromise;
    }

    this.initializationPromise = this.performInitialization();
    return this.initializationPromise;
  }

  private async performInitialization(): Promise<void> {
    console.log('[ScreenRegistry] Initializing screen registry...');
    
    const criticalScreens = this.getScreensByPriority(ScreenPriority.CRITICAL);
    const highPriorityScreens = this.getScreensByPriority(ScreenPriority.HIGH);

    // Create screen metadata for lazy loading service
    const screenMetadata: ScreenMetadata[] = Array.from(this.screens.values()).map(entry => ({
      name: entry.name,
      route: entry.route,
      importPath: '', // Not used with factory function
      dependencies: entry.dependencies,
      priority: entry.priority,
      estimatedSize: entry.estimatedSize
    }));

    // Initialize lazy loading service with screen metadata
    await lazyLoadingService.preloadCriticalScreens(screenMetadata);

    // Preload critical screens immediately
    await this.preloadScreens(criticalScreens);

    // Preload high priority screens with slight delay
    setTimeout(() => {
      this.preloadScreens(highPriorityScreens);
    }, 100);

    console.log('[ScreenRegistry] Screen registry initialized successfully');
  }

  private async preloadScreens(entries: ScreenRegistryEntry[]): Promise<void> {
    const preloadPromises = entries.map(async (entry) => {
      try {
        await lazyLoadingService.preloadScreen(
          entry.name,
          entry.importFactory,
          {
            priority: entry.priority,
            cacheStrategy: 'memory',
            errorBoundary: true
          }
        );

        // Preload dependencies if specified
        if (entry.preloadDependencies) {
          setTimeout(() => {
            entry.preloadDependencies!.forEach(depName => {
              this.getScreen(depName); // This will create the lazy component
            });
          }, 200);
        }
      } catch (error) {
        console.warn(`[ScreenRegistry] Failed to preload ${entry.name}:`, error);
      }
    });

    await Promise.allSettled(preloadPromises);
  }

  /**
   * Gets screens by priority level
   */
  private getScreensByPriority(priority: ScreenPriority): ScreenRegistryEntry[] {
    return Array.from(this.screens.values()).filter(entry => entry.priority === priority);
  }

  /**
   * Gets screens by category
   */
  getScreensByCategory(category: ScreenCategory): ScreenRegistryEntry[] {
    return Array.from(this.screens.values()).filter(entry => entry.category === category);
  }

  /**
   * Gets all registered screen names
   */
  getScreenNames(): string[] {
    return Array.from(this.screens.keys());
  }

  /**
   * Gets screen metadata
   */
  getScreenMetadata(screenName: string): ScreenRegistryEntry | undefined {
    return this.screens.get(screenName);
  }

  /**
   * Gets bundle size analytics
   */
  getBundleAnalytics(): {
    totalEstimatedSize: number;
    sizeByCategory: Record<ScreenCategory, number>;
    sizeByPriority: Record<ScreenPriority, number>;
    largestScreens: Array<{ name: string; size: number }>;
  } {
    const screens = Array.from(this.screens.values());
    
    const totalEstimatedSize = screens.reduce((total, screen) => total + screen.estimatedSize, 0);
    
    const sizeByCategory = screens.reduce((acc, screen) => {
      acc[screen.category] = (acc[screen.category] || 0) + screen.estimatedSize;
      return acc;
    }, {} as Record<ScreenCategory, number>);
    
    const sizeByPriority = screens.reduce((acc, screen) => {
      acc[screen.priority] = (acc[screen.priority] || 0) + screen.estimatedSize;
      return acc;
    }, {} as Record<ScreenPriority, number>);
    
    const largestScreens = screens
      .sort((a, b) => b.estimatedSize - a.estimatedSize)
      .slice(0, 10)
      .map(screen => ({ name: screen.name, size: screen.estimatedSize }));

    return {
      totalEstimatedSize,
      sizeByCategory,
      sizeByPriority,
      largestScreens
    };
  }

  /**
   * Records navigation for intelligent preloading
   */
  recordNavigation(fromScreen: string, toScreen: string): void {
    lazyLoadingService.recordNavigation(fromScreen, toScreen);
  }

  /**
   * Gets performance metrics for screen loading
   */
  getPerformanceMetrics() {
    return lazyLoadingService.getPerformanceMetrics();
  }
}

// Export singleton instance
export const screenRegistry = new ScreenRegistry();
export default ScreenRegistry;