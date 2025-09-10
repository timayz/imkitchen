import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import AsyncStorage from '@react-native-async-storage/async-storage';
import favoritesService from '../services/favorites_service';
import { FavoritesExport } from '../components/favorites/FavoritesImportExport';

// Recipe interface (basic structure)
export interface Recipe {
  id: string;
  name: string;
  description?: string;
  prepTime: number;
  complexity: 'simple' | 'moderate' | 'complex';
  tags?: string[];
}

// Recipe favorite record
export interface RecipeFavorite {
  recipeId: string;
  recipe: Recipe;
  favoriteAt: string;
  rotationMultiplier?: number; // Default 1.5x
}

interface FavoritesState {
  // State
  favorites: RecipeFavorite[];
  isLoading: boolean;
  error: string | null;
  lastUpdated: string | null;

  // Actions
  loadFavorites: () => Promise<void>;
  toggleFavorite: (recipeId: string) => Promise<void>;
  addFavorite: (recipeId: string) => Promise<void>;
  removeFavorite: (recipeId: string) => Promise<void>;
  isFavorite: (recipeId: string) => boolean;
  getFavoriteMultiplier: (recipeId: string) => number;
  
  // Import/Export
  exportFavorites: (data: FavoritesExport) => Promise<void>;
  importFavorites: (data: FavoritesExport) => Promise<{ imported: number; skipped: number }>;
  
  // Utility
  clearError: () => void;
  refreshFavorites: () => Promise<void>;
}

export const useFavoritesStore = create<FavoritesState>()(
  persist(
    (set, get) => ({
      // Initial state
      favorites: [],
      isLoading: false,
      error: null,
      lastUpdated: null,

      // Load favorites from API
      loadFavorites: async () => {
        set({ isLoading: true, error: null });
        
        try {
          const favorites = await favoritesService.getUserFavorites();
          
          set({
            favorites,
            isLoading: false,
            lastUpdated: new Date().toISOString(),
            error: null,
          });
        } catch (error) {
          console.error('Failed to load favorites:', error);
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Failed to load favorites',
          });
        }
      },

      // Toggle favorite status
      toggleFavorite: async (recipeId: string) => {
        const { favorites, isFavorite } = get();
        
        if (isFavorite(recipeId)) {
          await get().removeFavorite(recipeId);
        } else {
          await get().addFavorite(recipeId);
        }
      },

      // Add recipe to favorites
      addFavorite: async (recipeId: string) => {
        const { favorites } = get();
        
        // Check if already favorited
        if (favorites.find(fav => fav.recipeId === recipeId)) {
          return;
        }

        set({ isLoading: true, error: null });

        try {
          const favoritedRecipe = await favoritesService.addFavorite(recipeId);
          
          set(state => ({
            favorites: [...state.favorites, favoritedRecipe],
            isLoading: false,
            lastUpdated: new Date().toISOString(),
            error: null,
          }));
        } catch (error) {
          console.error('Failed to add favorite:', error);
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Failed to add favorite',
          });
          throw error;
        }
      },

      // Remove recipe from favorites
      removeFavorite: async (recipeId: string) => {
        const { favorites } = get();
        
        const favoriteIndex = favorites.findIndex(fav => fav.recipeId === recipeId);
        if (favoriteIndex === -1) {
          return;
        }

        set({ isLoading: true, error: null });

        try {
          await favoritesService.removeFavorite(recipeId);
          
          set(state => ({
            favorites: state.favorites.filter(fav => fav.recipeId !== recipeId),
            isLoading: false,
            lastUpdated: new Date().toISOString(),
            error: null,
          }));
        } catch (error) {
          console.error('Failed to remove favorite:', error);
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Failed to remove favorite',
          });
          throw error;
        }
      },

      // Check if recipe is favorited
      isFavorite: (recipeId: string) => {
        const { favorites } = get();
        return favorites.some(fav => fav.recipeId === recipeId);
      },

      // Get rotation multiplier for favorited recipe
      getFavoriteMultiplier: (recipeId: string) => {
        const { favorites } = get();
        const favorite = favorites.find(fav => fav.recipeId === recipeId);
        return favorite?.rotationMultiplier || 1.0;
      },

      // Export favorites data
      exportFavorites: async (data: FavoritesExport) => {
        try {
          await favoritesService.exportFavorites(data);
        } catch (error) {
          console.error('Failed to export favorites:', error);
          throw error;
        }
      },

      // Import favorites data
      importFavorites: async (data: FavoritesExport) => {
        set({ isLoading: true, error: null });
        
        try {
          const result = await favoritesService.importFavorites(data);
          
          // Refresh favorites after import
          await get().loadFavorites();
          
          set({ isLoading: false, error: null });
          return result;
        } catch (error) {
          console.error('Failed to import favorites:', error);
          set({
            isLoading: false,
            error: error instanceof Error ? error.message : 'Failed to import favorites',
          });
          throw error;
        }
      },

      // Clear error state
      clearError: () => {
        set({ error: null });
      },

      // Refresh favorites (force reload)
      refreshFavorites: async () => {
        await get().loadFavorites();
      },
    }),
    {
      name: 'imkitchen-favorites-store',
      storage: {
        getItem: async (name: string) => {
          try {
            const value = await AsyncStorage.getItem(name);
            return value ? JSON.parse(value) : null;
          } catch (error) {
            console.error('Failed to load favorites from storage:', error);
            return null;
          }
        },
        setItem: async (name: string, value: any) => {
          try {
            await AsyncStorage.setItem(name, JSON.stringify(value));
          } catch (error) {
            console.error('Failed to save favorites to storage:', error);
          }
        },
        removeItem: async (name: string) => {
          try {
            await AsyncStorage.removeItem(name);
          } catch (error) {
            console.error('Failed to remove favorites from storage:', error);
          }
        },
      },
    }
  )
);