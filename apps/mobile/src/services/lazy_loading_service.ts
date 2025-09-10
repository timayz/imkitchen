/**
 * Lazy Loading Service
 * 
 * Provides advanced lazy loading capabilities for React Native components,
 * screens, and resources with intelligent preloading and error boundaries.
 * 
 * Features:
 * - Screen-level code splitting with React.lazy()
 * - Component preloading based on navigation patterns
 * - Bundle size optimization with dynamic imports
 * - Error boundary integration for failed lazy loads
 * - Loading state management and progress tracking
 * - Priority-based resource loading (critical vs non-critical)
 */

import * as React from 'react';
import { ComponentType, lazy, Suspense, ReactNode } from 'react';
import { ActivityIndicator, View, StyleSheet } from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';

export interface LazyLoadOptions {
  priority: 'critical' | 'high' | 'normal' | 'low';
  preloadTimeout?: number;
  fallbackComponent?: ComponentType;
  errorBoundary?: boolean;
  cacheStrategy: 'memory' | 'storage' | 'none';
}

export interface ScreenMetadata {
  name: string;
  route: string;
  importPath: string;
  dependencies?: string[];
  priority: LazyLoadOptions['priority'];
  estimatedSize: number; // Bundle size in bytes
}

export interface LoadingProgress {
  screenName: string;
  loaded: boolean;
  loading: boolean;
  error: Error | null;
  progress: number; // 0-100
  loadTime: number;
}

class LazyLoadingService {
  private loadedScreens = new Map<string, ComponentType<any>>();
  private loadingPromises = new Map<string, Promise<ComponentType<any>>>();
  private preloadQueue: ScreenMetadata[] = [];
  private loadingProgress = new Map<string, LoadingProgress>();
  private navigationPatterns: string[] = [];

  constructor() {
    this.initializeNavigationTracking();
  }

  private async initializeNavigationTracking() {
    try {
      const patterns = await AsyncStorage.getItem('navigation_patterns');
      this.navigationPatterns = patterns ? JSON.parse(patterns) : [];
    } catch (error) {
      console.warn('Failed to load navigation patterns:', error);
    }
  }

  /**
   * Creates a lazily loaded screen component with advanced loading features
   */
  createLazyScreen<T = any>(
    screenName: string,
    importFactory: () => Promise<{ default: ComponentType<T> }>,
    options: LazyLoadOptions = {
      priority: 'normal',
      cacheStrategy: 'memory',
      errorBoundary: true
    }
  ): ComponentType<T> {
    // Create lazy component with error boundary
    const LazyComponent = lazy(importFactory);

    // Initialize loading progress tracking
    this.loadingProgress.set(screenName, {
      screenName,
      loaded: false,
      loading: false,
      error: null,
      progress: 0,
      loadTime: 0
    });

    const WrappedComponent: ComponentType<T> = (props) => {
      return React.createElement(
        Suspense,
        { fallback: this.createLoadingFallback(screenName, options) },
        options.errorBoundary
          ? React.createElement(
              LazyErrorBoundary,
              { screenName },
              React.createElement(LazyComponent, props)
            )
          : React.createElement(LazyComponent, props)
      );
    };

    // Pre-warm the component if priority is critical or high
    if (options.priority === 'critical' || options.priority === 'high') {
      this.preloadScreen(screenName, importFactory, options);
    }

    return WrappedComponent;
  }

  /**
   * Preloads a screen component in the background
   */
  async preloadScreen<T>(
    screenName: string,
    importFactory: () => Promise<{ default: ComponentType<T> }>,
    options: LazyLoadOptions
  ): Promise<ComponentType<T> | null> {
    if (this.loadedScreens.has(screenName)) {
      return this.loadedScreens.get(screenName);
    }

    if (this.loadingPromises.has(screenName)) {
      return this.loadingPromises.get(screenName) as Promise<ComponentType<T>>;
    }

    const startTime = Date.now();
    this.updateLoadingProgress(screenName, { loading: true, progress: 10 });

    const loadPromise = importFactory().then(module => {
      const loadTime = Date.now() - startTime;
      const component = module.default;

      this.loadedScreens.set(screenName, component);
      this.updateLoadingProgress(screenName, {
        loaded: true,
        loading: false,
        progress: 100,
        loadTime
      });

      // Cache component if strategy allows
      if (options.cacheStrategy === 'storage') {
        this.cacheScreenMetadata(screenName, loadTime);
      }

      console.log(`[LazyLoading] Screen '${screenName}' loaded in ${loadTime}ms`);
      return component;
    }).catch(error => {
      this.updateLoadingProgress(screenName, {
        loading: false,
        error,
        progress: 0
      });
      console.error(`[LazyLoading] Failed to load screen '${screenName}':`, error);
      throw error;
    });

    this.loadingPromises.set(screenName, loadPromise);
    return loadPromise;
  }

  /**
   * Creates an intelligent loading fallback with progress indication
   */
  private createLoadingFallback(screenName: string, options: LazyLoadOptions): ReactNode {
    if (options.fallbackComponent) {
      return React.createElement(options.fallbackComponent);
    }

    return React.createElement(DefaultLoadingScreen, { screenName });
  }

  /**
   * Preloads screens based on navigation patterns and priority
   */
  async preloadCriticalScreens(screenDefinitions: ScreenMetadata[]): Promise<void> {
    // Sort screens by priority and navigation patterns
    const sortedScreens = this.prioritizeScreensForPreload(screenDefinitions);
    
    console.log(`[LazyLoading] Preloading ${sortedScreens.length} critical screens`);

    const preloadPromises = sortedScreens
      .filter(screen => screen.priority === 'critical' || screen.priority === 'high')
      .slice(0, 5) // Limit concurrent preloads
      .map(screen => this.preloadScreenByMetadata(screen));

    await Promise.allSettled(preloadPromises);
  }

  private async preloadScreenByMetadata(metadata: ScreenMetadata): Promise<void> {
    try {
      // Dynamic import based on metadata
      const importFactory = () => import(metadata.importPath);
      await this.preloadScreen(metadata.name, importFactory, {
        priority: metadata.priority,
        cacheStrategy: 'memory'
      });
    } catch (error) {
      console.warn(`[LazyLoading] Failed to preload ${metadata.name}:`, error);
    }
  }

  /**
   * Prioritizes screens for preloading based on usage patterns
   */
  private prioritizeScreensForPreload(screens: ScreenMetadata[]): ScreenMetadata[] {
    return screens.sort((a, b) => {
      // Priority weight
      const priorityWeight = {
        critical: 1000,
        high: 100,
        normal: 10,
        low: 1
      };

      // Navigation pattern weight (how often user navigates to this screen)
      const aPatternWeight = this.getNavigationPatternWeight(a.name);
      const bPatternWeight = this.getNavigationPatternWeight(b.name);

      // Bundle size weight (smaller bundles load faster)
      const aSizeWeight = 1000000 / (a.estimatedSize || 100000);
      const bSizeWeight = 1000000 / (b.estimatedSize || 100000);

      const aScore = priorityWeight[a.priority] + aPatternWeight + aSizeWeight;
      const bScore = priorityWeight[b.priority] + bPatternWeight + bSizeWeight;

      return bScore - aScore;
    });
  }

  private getNavigationPatternWeight(screenName: string): number {
    const occurrences = this.navigationPatterns.filter(pattern => 
      pattern.includes(screenName)
    ).length;
    return occurrences * 10;
  }

  /**
   * Updates loading progress for a screen
   */
  private updateLoadingProgress(screenName: string, updates: Partial<LoadingProgress>): void {
    const current = this.loadingProgress.get(screenName);
    if (current) {
      this.loadingProgress.set(screenName, { ...current, ...updates });
    }
  }

  /**
   * Gets loading progress for all screens
   */
  getLoadingProgress(): LoadingProgress[] {
    return Array.from(this.loadingProgress.values());
  }

  /**
   * Records navigation patterns for intelligent preloading
   */
  recordNavigation(fromScreen: string, toScreen: string): void {
    const pattern = `${fromScreen}>${toScreen}`;
    this.navigationPatterns.unshift(pattern);
    
    // Keep only recent patterns (last 100)
    if (this.navigationPatterns.length > 100) {
      this.navigationPatterns = this.navigationPatterns.slice(0, 100);
    }

    // Persist patterns for next app launch
    AsyncStorage.setItem('navigation_patterns', JSON.stringify(this.navigationPatterns))
      .catch(error => console.warn('Failed to save navigation patterns:', error));
  }

  /**
   * Caches screen metadata for performance optimization
   */
  private async cacheScreenMetadata(screenName: string, loadTime: number): Promise<void> {
    try {
      const metadata = {
        screenName,
        loadTime,
        cachedAt: Date.now()
      };
      await AsyncStorage.setItem(`screen_cache_${screenName}`, JSON.stringify(metadata));
    } catch (error) {
      console.warn(`Failed to cache metadata for ${screenName}:`, error);
    }
  }

  /**
   * Clears all cached screens and resets state
   */
  clearCache(): void {
    this.loadedScreens.clear();
    this.loadingPromises.clear();
    this.loadingProgress.clear();
    console.log('[LazyLoading] Cache cleared');
  }

  /**
   * Gets performance metrics for loaded screens
   */
  getPerformanceMetrics(): {
    totalScreens: number;
    loadedScreens: number;
    averageLoadTime: number;
    failedLoads: number;
  } {
    const progress = this.getLoadingProgress();
    const loadedScreens = progress.filter(p => p.loaded);
    const failedLoads = progress.filter(p => p.error !== null);
    const totalLoadTime = loadedScreens.reduce((sum, p) => sum + p.loadTime, 0);
    const averageLoadTime = loadedScreens.length > 0 ? totalLoadTime / loadedScreens.length : 0;

    return {
      totalScreens: progress.length,
      loadedScreens: loadedScreens.length,
      averageLoadTime,
      failedLoads: failedLoads.length
    };
  }
}

// Default loading screen component
const DefaultLoadingScreen: React.FC<{ screenName: string }> = ({ screenName }) =>
  React.createElement(
    View,
    { style: styles.loadingContainer },
    React.createElement(ActivityIndicator, { size: "large", color: "#007AFF" })
  );

// Error boundary for lazy loaded components
class LazyErrorBoundary extends React.Component<
  { children: ReactNode; screenName: string },
  { hasError: boolean; error: Error | null }
> {
  constructor(props: any) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: any) {
    console.error(`[LazyLoading] Error in screen '${this.props.screenName}':`, error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return React.createElement(
        View,
        { style: styles.errorContainer },
        React.createElement(ActivityIndicator, { size: "small", color: "#FF3B30" })
      );
    }

    return this.props.children;
  }
}

const styles = StyleSheet.create({
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#FFFFFF'
  },
  errorContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#FFFFFF'
  }
});

// Export singleton instance
export const lazyLoadingService = new LazyLoadingService();
export default LazyLoadingService;