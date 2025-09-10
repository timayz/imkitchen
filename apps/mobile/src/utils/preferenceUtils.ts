import AsyncStorage from '@react-native-async-storage/async-storage';
import { CoreUserPreferences } from '../store/preference_store';

const PREFERENCE_CACHE_KEY = 'imkitchen-preferences-cache';
const PREFERENCE_SYNC_KEY = 'imkitchen-preferences-last-sync';

/**
 * Utility functions for preference management, caching, and synchronization
 */
export class PreferenceUtils {
  /**
   * Cache preferences locally for offline access
   */
  static async cachePreferences(preferences: CoreUserPreferences): Promise<void> {
    try {
      const cacheData = {
        preferences,
        timestamp: Date.now(),
      };
      await AsyncStorage.setItem(PREFERENCE_CACHE_KEY, JSON.stringify(cacheData));
    } catch (error) {
      console.error('Failed to cache preferences:', error);
    }
  }

  /**
   * Load cached preferences
   */
  static async loadCachedPreferences(): Promise<CoreUserPreferences | null> {
    try {
      const cached = await AsyncStorage.getItem(PREFERENCE_CACHE_KEY);
      if (!cached) return null;

      const { preferences, timestamp } = JSON.parse(cached);
      
      // Check if cache is too old (older than 24 hours)
      const maxAge = 24 * 60 * 60 * 1000; // 24 hours
      if (Date.now() - timestamp > maxAge) {
        return null;
      }

      return preferences;
    } catch (error) {
      console.error('Failed to load cached preferences:', error);
      return null;
    }
  }

  /**
   * Clear cached preferences
   */
  static async clearCache(): Promise<void> {
    try {
      await AsyncStorage.multiRemove([PREFERENCE_CACHE_KEY, PREFERENCE_SYNC_KEY]);
    } catch (error) {
      console.error('Failed to clear preference cache:', error);
    }
  }

  /**
   * Record last sync timestamp
   */
  static async recordSync(): Promise<void> {
    try {
      await AsyncStorage.setItem(PREFERENCE_SYNC_KEY, Date.now().toString());
    } catch (error) {
      console.error('Failed to record sync timestamp:', error);
    }
  }

  /**
   * Check if preferences need syncing (haven't synced in a while)
   */
  static async needsSync(): Promise<boolean> {
    try {
      const lastSync = await AsyncStorage.getItem(PREFERENCE_SYNC_KEY);
      if (!lastSync) return true;

      const timeSinceSync = Date.now() - parseInt(lastSync, 10);
      const syncInterval = 5 * 60 * 1000; // 5 minutes

      return timeSinceSync > syncInterval;
    } catch (error) {
      console.error('Failed to check sync status:', error);
      return true; // Default to needing sync
    }
  }

  /**
   * Validate preferences against business rules
   */
  static validatePreferences(preferences: CoreUserPreferences): {
    isValid: boolean;
    errors: string[];
  } {
    const errors: string[] = [];

    // Validate maxCookTime (15-180 minutes)
    if (typeof preferences.maxCookTime !== 'number') {
      errors.push('Max cook time must be a number');
    } else if (preferences.maxCookTime < 15 || preferences.maxCookTime > 180) {
      errors.push('Max cook time must be between 15 and 180 minutes');
    } else if (!Number.isInteger(preferences.maxCookTime)) {
      errors.push('Max cook time must be a whole number');
    }

    // Validate preferredComplexity
    const validComplexities = ['simple', 'moderate', 'complex'];
    if (!validComplexities.includes(preferences.preferredComplexity)) {
      errors.push('Preferred complexity must be simple, moderate, or complex');
    }

    return {
      isValid: errors.length === 0,
      errors,
    };
  }

  /**
   * Sanitize preferences to ensure they meet requirements
   */
  static sanitizePreferences(preferences: Partial<CoreUserPreferences>): CoreUserPreferences {
    const defaultPrefs: CoreUserPreferences = {
      maxCookTime: 60,
      preferredComplexity: 'moderate',
    };

    // Sanitize maxCookTime
    let maxCookTime = preferences.maxCookTime ?? defaultPrefs.maxCookTime;
    if (typeof maxCookTime !== 'number' || maxCookTime < 15 || maxCookTime > 180) {
      maxCookTime = defaultPrefs.maxCookTime;
    } else {
      // Round to nearest 5 minutes
      maxCookTime = Math.round(maxCookTime / 5) * 5;
    }

    // Sanitize preferredComplexity
    const validComplexities = ['simple', 'moderate', 'complex'] as const;
    let preferredComplexity = preferences.preferredComplexity ?? defaultPrefs.preferredComplexity;
    if (!validComplexities.includes(preferredComplexity)) {
      preferredComplexity = defaultPrefs.preferredComplexity;
    }

    return {
      maxCookTime,
      preferredComplexity,
    };
  }

  /**
   * Generate preference insights for analytics
   */
  static generateInsights(preferences: CoreUserPreferences): {
    cookingProfile: string;
    timeCategory: string;
    complexityLevel: number;
  } {
    // Determine cooking profile
    let cookingProfile = 'balanced';
    if (preferences.maxCookTime <= 30 && preferences.preferredComplexity === 'simple') {
      cookingProfile = 'quick_and_easy';
    } else if (preferences.maxCookTime >= 90 && preferences.preferredComplexity === 'complex') {
      cookingProfile = 'gourmet_enthusiast';
    } else if (preferences.preferredComplexity === 'simple') {
      cookingProfile = 'casual_cook';
    } else if (preferences.preferredComplexity === 'complex') {
      cookingProfile = 'ambitious_cook';
    }

    // Determine time category
    let timeCategory = 'standard';
    if (preferences.maxCookTime <= 30) {
      timeCategory = 'quick';
    } else if (preferences.maxCookTime >= 90) {
      timeCategory = 'extended';
    }

    // Convert complexity to numeric level
    const complexityLevels = {
      simple: 1,
      moderate: 2,
      complex: 3,
    };

    return {
      cookingProfile,
      timeCategory,
      complexityLevel: complexityLevels[preferences.preferredComplexity],
    };
  }

  /**
   * Compare two preference objects for changes
   */
  static hasPreferencesChanged(
    oldPrefs: CoreUserPreferences,
    newPrefs: CoreUserPreferences
  ): boolean {
    return (
      oldPrefs.maxCookTime !== newPrefs.maxCookTime ||
      oldPrefs.preferredComplexity !== newPrefs.preferredComplexity
    );
  }

  /**
   * Format preferences for display
   */
  static formatPreferencesForDisplay(preferences: CoreUserPreferences): {
    maxCookTimeDisplay: string;
    complexityDisplay: string;
  } {
    // Format cook time
    let maxCookTimeDisplay: string;
    if (preferences.maxCookTime >= 60) {
      const hours = Math.floor(preferences.maxCookTime / 60);
      const minutes = preferences.maxCookTime % 60;
      if (minutes === 0) {
        maxCookTimeDisplay = `${hours}h`;
      } else {
        maxCookTimeDisplay = `${hours}h ${minutes}m`;
      }
    } else {
      maxCookTimeDisplay = `${preferences.maxCookTime}m`;
    }

    // Format complexity
    const complexityDisplayMap = {
      simple: 'Simple & Quick',
      moderate: 'Balanced',
      complex: 'Advanced',
    };
    const complexityDisplay = complexityDisplayMap[preferences.preferredComplexity];

    return {
      maxCookTimeDisplay,
      complexityDisplay,
    };
  }
}