import { useEffect, useCallback } from 'react';
import { usePreferenceStore, CoreUserPreferences } from '../store/preference_store';

/**
 * Custom hook for preference management with optimized re-renders
 * and convenient methods for common operations
 */
export const usePreferences = () => {
  const store = usePreferenceStore();

  // Memoized selectors to prevent unnecessary re-renders
  const preferences = usePreferenceStore(state => state.preferences);
  const isLoading = usePreferenceStore(state => state.isLoading);
  const error = usePreferenceStore(state => state.error);
  const lastUpdated = usePreferenceStore(state => state.lastUpdated);

  // Auto-load preferences on mount if not already loaded
  useEffect(() => {
    if (!lastUpdated && !isLoading && !error) {
      store.loadPreferences();
    }
  }, [lastUpdated, isLoading, error, store]);

  // Memoized update function with optimistic updates
  const updatePreferencesOptimistic = useCallback(async (updates: Partial<CoreUserPreferences>) => {
    // Optimistic update for immediate UI feedback
    if (updates.maxCookTime !== undefined) {
      store.setMaxCookTime(updates.maxCookTime);
    }
    if (updates.preferredComplexity !== undefined) {
      store.setPreferredComplexity(updates.preferredComplexity);
    }

    // Then sync with server
    try {
      await store.updatePreferences(updates);
    } catch (err) {
      // Revert optimistic update on failure
      await store.loadPreferences();
      throw err;
    }
  }, [store]);

  // Convenience methods
  const setMaxCookTime = useCallback((time: number) => {
    updatePreferencesOptimistic({ maxCookTime: time });
  }, [updatePreferencesOptimistic]);

  const setComplexity = useCallback((complexity: 'simple' | 'moderate' | 'complex') => {
    updatePreferencesOptimistic({ preferredComplexity: complexity });
  }, [updatePreferencesOptimistic]);

  const resetToDefaults = useCallback(async () => {
    await store.resetPreferences();
  }, [store]);

  const refreshPreferences = useCallback(async () => {
    await store.loadPreferences();
  }, [store]);

  const clearError = useCallback(() => {
    store.clearError();
  }, [store]);

  return {
    // State
    preferences,
    isLoading,
    error,
    lastUpdated,

    // Actions
    updatePreferences: updatePreferencesOptimistic,
    setMaxCookTime,
    setComplexity,
    resetToDefaults,
    refreshPreferences,
    clearError,

    // Computed values
    hasPreferences: !!lastUpdated,
    isDefaultSettings: preferences.maxCookTime === 60 && preferences.preferredComplexity === 'moderate',
  };
};

/**
 * Hook for preference validation
 */
export const usePreferenceValidation = () => {
  const validatePreferences = useCallback((preferences: CoreUserPreferences) => {
    const errors: string[] = [];

    // Validate maxCookTime
    if (preferences.maxCookTime < 15 || preferences.maxCookTime > 180) {
      errors.push('Max cook time must be between 15 and 180 minutes');
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
  }, []);

  return { validatePreferences };
};

/**
 * Hook for preference analytics and insights
 */
export const usePreferenceInsights = () => {
  const preferences = usePreferenceStore(state => state.preferences);

  const insights = {
    cookingStyle: preferences.preferredComplexity === 'simple' 
      ? 'Quick & Easy' 
      : preferences.preferredComplexity === 'complex' 
      ? 'Gourmet Chef' 
      : 'Balanced Cook',
    
    timeCommitment: preferences.maxCookTime <= 30 
      ? 'Minimal Time' 
      : preferences.maxCookTime <= 60 
      ? 'Standard Time' 
      : 'Generous Time',
    
    estimatedMealsPerWeek: Math.floor((7 * 60) / preferences.maxCookTime), // Rough estimate
    
    complexityDescription: {
      simple: 'Perfect for busy weekdays',
      moderate: 'Great balance of flavor and effort',
      complex: 'Love to experiment and learn',
    }[preferences.preferredComplexity],
  };

  return insights;
};