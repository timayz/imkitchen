import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import AsyncStorage from '@react-native-async-storage/async-storage';
import preferenceService from '../services/preference_service';

// Weekly pattern interface matching backend model
export interface WeeklyPattern {
  dayOfWeek: number;         // 0=Sunday, 6=Saturday
  maxPrepTime: number;       // Minutes
  preferredComplexity: 'simple' | 'moderate' | 'complex';
  isWeekendPattern: boolean;
}

// Core user preferences matching backend model
export interface CoreUserPreferences {
  maxCookTime: number;           // 15-180 minutes
  preferredComplexity: 'simple' | 'moderate' | 'complex';
}

// API response format
interface PreferenceResponse {
  data: CoreUserPreferences;
  metadata?: {
    retrievedAt?: string;
    updatedAt?: string;
  };
}

interface PreferenceState {
  // State
  preferences: CoreUserPreferences;
  weeklyPatterns: WeeklyPattern[];
  isLoading: boolean;
  error: string | null;
  lastUpdated: string | null;

  // Actions
  loadPreferences: () => Promise<void>;
  updatePreferences: (preferences: Partial<CoreUserPreferences>) => Promise<void>;
  resetPreferences: () => Promise<void>;
  clearError: () => void;

  // Weekly patterns actions
  loadWeeklyPatterns: () => Promise<void>;
  updateWeeklyPatterns: (patterns: WeeklyPattern[]) => Promise<void>;
  updateDayPattern: (dayOfWeek: number, pattern: Partial<WeeklyPattern>) => Promise<void>;

  // Local state management
  setMaxCookTime: (time: number) => void;
  setPreferredComplexity: (complexity: 'simple' | 'moderate' | 'complex') => void;
}

// Default preferences
const DEFAULT_PREFERENCES: CoreUserPreferences = {
  maxCookTime: 60,
  preferredComplexity: 'moderate',
};

// Default weekly patterns
const DEFAULT_WEEKLY_PATTERNS: WeeklyPattern[] = [
  // Sunday - Weekend pattern
  { dayOfWeek: 0, maxPrepTime: 90, preferredComplexity: 'moderate', isWeekendPattern: true },
  // Monday - Weekday pattern
  { dayOfWeek: 1, maxPrepTime: 45, preferredComplexity: 'simple', isWeekendPattern: false },
  // Tuesday - Weekday pattern
  { dayOfWeek: 2, maxPrepTime: 45, preferredComplexity: 'simple', isWeekendPattern: false },
  // Wednesday - Weekday pattern
  { dayOfWeek: 3, maxPrepTime: 60, preferredComplexity: 'moderate', isWeekendPattern: false },
  // Thursday - Weekday pattern
  { dayOfWeek: 4, maxPrepTime: 45, preferredComplexity: 'simple', isWeekendPattern: false },
  // Friday - Weekday pattern
  { dayOfWeek: 5, maxPrepTime: 60, preferredComplexity: 'moderate', isWeekendPattern: false },
  // Saturday - Weekend pattern  
  { dayOfWeek: 6, maxPrepTime: 120, preferredComplexity: 'complex', isWeekendPattern: true },
];

export const usePreferenceStore = create<PreferenceState>()(
  persist(
    (set, get) => ({
      // Initial state
      preferences: DEFAULT_PREFERENCES,
      weeklyPatterns: DEFAULT_WEEKLY_PATTERNS,
      isLoading: false,
      error: null,
      lastUpdated: null,

      // Load preferences from API
      loadPreferences: async () => {
        set({ isLoading: true, error: null });
        
        try {
          const response: PreferenceResponse = await preferenceService.getUserPreferences();
          
          set({
            preferences: response.data,
            isLoading: false,
            lastUpdated: response.metadata?.retrievedAt || new Date().toISOString(),
            error: null,
          });
        } catch (error) {
          console.error('Failed to load preferences:', error);
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Failed to load preferences',
          });
        }
      },

      // Update preferences via API
      updatePreferences: async (updates: Partial<CoreUserPreferences>) => {
        const { preferences } = get();
        const updatedPreferences = { ...preferences, ...updates };

        // Validate locally before API call
        if (updatedPreferences.maxCookTime < 15 || updatedPreferences.maxCookTime > 180) {
          set({ error: 'Max cook time must be between 15 and 180 minutes' });
          return;
        }

        const validComplexities = ['simple', 'moderate', 'complex'];
        if (!validComplexities.includes(updatedPreferences.preferredComplexity)) {
          set({ error: 'Invalid complexity level' });
          return;
        }

        set({ isLoading: true, error: null });

        try {
          const response: PreferenceResponse = await preferenceService.updateUserPreferences(updatedPreferences);
          
          set({
            preferences: response.data,
            isLoading: false,
            lastUpdated: response.metadata?.updatedAt || new Date().toISOString(),
            error: null,
          });
        } catch (error) {
          console.error('Failed to update preferences:', error);
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Failed to update preferences',
          });
        }
      },

      // Reset preferences to defaults
      resetPreferences: async () => {
        set({ isLoading: true, error: null });

        try {
          const response: PreferenceResponse = await preferenceService.resetUserPreferences();
          
          set({
            preferences: response.data,
            isLoading: false,
            lastUpdated: response.metadata?.resetAt || new Date().toISOString(),
            error: null,
          });
        } catch (error) {
          console.error('Failed to reset preferences:', error);
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Failed to reset preferences',
          });
        }
      },

      // Clear error state
      clearError: () => {
        set({ error: null });
      },

      // Weekly patterns actions
      loadWeeklyPatterns: async () => {
        set({ isLoading: true, error: null });
        
        try {
          const patterns: WeeklyPattern[] = await preferenceService.getWeeklyPatterns();
          
          set({
            weeklyPatterns: patterns,
            isLoading: false,
            error: null,
          });
        } catch (error) {
          console.error('Failed to load weekly patterns:', error);
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Failed to load weekly patterns',
          });
        }
      },

      updateWeeklyPatterns: async (patterns: WeeklyPattern[]) => {
        set({ isLoading: true, error: null });

        try {
          const updatedPatterns: WeeklyPattern[] = await preferenceService.updateWeeklyPatterns(patterns);
          
          set({
            weeklyPatterns: updatedPatterns,
            isLoading: false,
            lastUpdated: new Date().toISOString(),
            error: null,
          });
        } catch (error) {
          console.error('Failed to update weekly patterns:', error);
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Failed to update weekly patterns',
          });
        }
      },

      updateDayPattern: async (dayOfWeek: number, patternUpdate: Partial<WeeklyPattern>) => {
        const { weeklyPatterns } = get();
        
        const updatedPatterns = weeklyPatterns.map(pattern => 
          pattern.dayOfWeek === dayOfWeek 
            ? { ...pattern, ...patternUpdate, dayOfWeek }
            : pattern
        );

        // If pattern doesn't exist, create it
        if (!weeklyPatterns.find(p => p.dayOfWeek === dayOfWeek)) {
          updatedPatterns.push({
            dayOfWeek,
            maxPrepTime: 60,
            preferredComplexity: 'moderate',
            isWeekendPattern: dayOfWeek === 0 || dayOfWeek === 6,
            ...patternUpdate
          });
        }

        try {
          await get().updateWeeklyPatterns(updatedPatterns);
        } catch (error) {
          console.error('Failed to update day pattern:', error);
        }
      },

      // Local state updates (for immediate UI feedback)
      setMaxCookTime: (time: number) => {
        const { preferences } = get();
        set({
          preferences: { ...preferences, maxCookTime: time },
        });
      },

      setPreferredComplexity: (complexity: 'simple' | 'moderate' | 'complex') => {
        const { preferences } = get();
        set({
          preferences: { ...preferences, preferredComplexity: complexity },
        });
      },
    }),
    {
      name: 'imkitchen-preferences-store',
      storage: {
        getItem: async (name: string) => {
          try {
            const value = await AsyncStorage.getItem(name);
            return value ? JSON.parse(value) : null;
          } catch (error) {
            console.error('Failed to load preferences from storage:', error);
            return null;
          }
        },
        setItem: async (name: string, value: any) => {
          try {
            await AsyncStorage.setItem(name, JSON.stringify(value));
          } catch (error) {
            console.error('Failed to save preferences to storage:', error);
          }
        },
        removeItem: async (name: string) => {
          try {
            await AsyncStorage.removeItem(name);
          } catch (error) {
            console.error('Failed to remove preferences from storage:', error);
          }
        },
      },
    }
  )
);