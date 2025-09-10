import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import AsyncStorage from '@react-native-async-storage/async-storage';
import type {
  Recipe,
  CreateRecipeInput,
  UpdateRecipeInput,
  RecipeSearchParams,
  RecipeSearchResponse,
  ImportRecipeInput,
  ImportRecipeResult
} from '@imkitchen/shared-types';
import { RecipeClient } from '@imkitchen/api-client';
import { PhotoService, PhotoUploadResult } from '../services/photo_service';
import { RecipeCacheService } from '../services/recipe_cache_service';
import { OfflineRecipeRepository, NetworkStatus, OfflineOperationResult } from '../services/offline_recipe_repository';

interface RecipeStore {
  // State
  recipes: Recipe[];
  currentRecipe: Recipe | null;
  searchResults: RecipeSearchResponse | null;
  loading: boolean;
  error: string | null;
  
  // Enhanced offline support
  networkStatus: NetworkStatus;
  syncInProgress: boolean;
  lastSyncAttempt: Date | null;
  syncErrors: string[];
  
  // Legacy offline support (deprecated but kept for migration)
  cachedRecipes: Record<string, Recipe>;
  pendingSync: {
    create: CreateRecipeInput[];
    update: { id: string; input: UpdateRecipeInput }[];
    delete: string[];
  };
  
  // Search filters
  filters: RecipeSearchParams;
  
  // Actions
  setRecipeClient: (client: RecipeClient) => void;
  setPhotoService: (service: PhotoService) => void;
  
  // Enhanced caching and offline support
  initializeOfflineServices: () => void;
  getNetworkStatus: () => NetworkStatus;
  isOffline: () => boolean;
  
  // Recipe CRUD
  createRecipe: (input: CreateRecipeInput) => Promise<Recipe | null>;
  getRecipe: (id: string) => Promise<Recipe | null>;
  updateRecipe: (id: string, input: UpdateRecipeInput) => Promise<Recipe | null>;
  deleteRecipe: (id: string) => Promise<boolean>;
  
  // Search and filtering
  searchRecipes: (params?: RecipeSearchParams) => Promise<void>;
  setFilters: (filters: Partial<RecipeSearchParams>) => void;
  clearFilters: () => void;
  
  // Import
  importRecipe: (input: ImportRecipeInput) => Promise<ImportRecipeResult | null>;
  
  // Photo management
  uploadRecipePhoto: (recipeId: string, imageUri: string) => Promise<PhotoUploadResult>;
  deleteRecipePhoto: (recipeId: string) => Promise<PhotoUploadResult>;
  
  // Enhanced offline support
  syncPendingChanges: () => Promise<void>;
  invalidateCache: (recipeId?: string) => Promise<void>;
  warmCache: (recipeIds?: string[]) => Promise<void>;
  cleanupExpiredCache: () => Promise<void>;
  
  // Legacy cache support (deprecated)
  cacheRecipe: (recipe: Recipe) => void;
  getCachedRecipe: (id: string) => Recipe | null;
  
  // UI helpers
  setCurrentRecipe: (recipe: Recipe | null) => void;
  clearError: () => void;
  reset: () => void;
}

const initialFilters: RecipeSearchParams = {
  page: 1,
  limit: 20,
  sortBy: 'created_at',
  sortOrder: 'desc',
};

const initialState = {
  recipes: [],
  currentRecipe: null,
  searchResults: null,
  loading: false,
  error: null,
  networkStatus: {
    isConnected: false,
    type: 'unknown',
    isInternetReachable: null,
  } as NetworkStatus,
  syncInProgress: false,
  lastSyncAttempt: null,
  syncErrors: [],
  cachedRecipes: {},
  pendingSync: {
    create: [],
    update: [],
    delete: [],
  },
  filters: initialFilters,
};

let recipeClient: RecipeClient;
let photoService: PhotoService;
let cacheService: RecipeCacheService;
let offlineRepository: OfflineRecipeRepository;

export const useRecipeStore = create<RecipeStore>()(
  persist(
    (set, get) => ({
      ...initialState,

      setRecipeClient: (client: RecipeClient) => {
        recipeClient = client;
      },

      setPhotoService: (service: PhotoService) => {
        photoService = service;
      },

      initializeOfflineServices: () => {
        cacheService = new RecipeCacheService();
        if (recipeClient) {
          offlineRepository = new OfflineRecipeRepository(recipeClient, cacheService);
          
          // Update network status periodically
          const updateNetworkStatus = () => {
            const status = offlineRepository.getNetworkStatus();
            set({ networkStatus: status });
          };
          
          // Update immediately and then every 30 seconds
          updateNetworkStatus();
          setInterval(updateNetworkStatus, 30000);
        }
      },

      getNetworkStatus: () => {
        return offlineRepository?.getNetworkStatus() || get().networkStatus;
      },

      isOffline: () => {
        return !offlineRepository?.isOnline() ?? true;
      },

      createRecipe: async (input: CreateRecipeInput) => {
        set({ loading: true, error: null });
        
        try {
          if (!recipeClient) {
            throw new Error('Recipe client not initialized');
          }

          const recipe = await recipeClient.createRecipe(input);
          
          set(state => ({
            recipes: [recipe, ...state.recipes],
            loading: false,
          }));
          
          get().cacheRecipe(recipe);
          return recipe;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to create recipe';
          
          // Add to pending sync for offline support
          set(state => ({
            pendingSync: {
              ...state.pendingSync,
              create: [...state.pendingSync.create, input],
            },
            loading: false,
            error: errorMessage,
          }));
          
          return null;
        }
      },

      getRecipe: async (id: string) => {
        set({ loading: true, error: null });
        
        try {
          // Check cache first
          const cached = get().getCachedRecipe(id);
          if (cached) {
            set({ currentRecipe: cached, loading: false });
            return cached;
          }

          if (!recipeClient) {
            throw new Error('Recipe client not initialized');
          }

          const recipe = await recipeClient.getRecipe(id);
          
          set({ currentRecipe: recipe, loading: false });
          get().cacheRecipe(recipe);
          return recipe;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to get recipe';
          set({ loading: false, error: errorMessage });
          return null;
        }
      },

      updateRecipe: async (id: string, input: UpdateRecipeInput) => {
        set({ loading: true, error: null });
        
        try {
          if (!recipeClient) {
            throw new Error('Recipe client not initialized');
          }

          const recipe = await recipeClient.updateRecipe(id, input);
          
          set(state => ({
            recipes: state.recipes.map(r => r.id === id ? recipe : r),
            currentRecipe: state.currentRecipe?.id === id ? recipe : state.currentRecipe,
            loading: false,
          }));
          
          get().cacheRecipe(recipe);
          return recipe;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to update recipe';
          
          // Add to pending sync for offline support
          set(state => ({
            pendingSync: {
              ...state.pendingSync,
              update: [...state.pendingSync.update, { id, input }],
            },
            loading: false,
            error: errorMessage,
          }));
          
          return null;
        }
      },

      deleteRecipe: async (id: string) => {
        set({ loading: true, error: null });
        
        try {
          if (!recipeClient) {
            throw new Error('Recipe client not initialized');
          }

          await recipeClient.deleteRecipe(id);
          
          set(state => ({
            recipes: state.recipes.filter(r => r.id !== id),
            currentRecipe: state.currentRecipe?.id === id ? null : state.currentRecipe,
            loading: false,
          }));
          
          // Remove from cache
          set(state => {
            const newCachedRecipes = { ...state.cachedRecipes };
            delete newCachedRecipes[id];
            return { cachedRecipes: newCachedRecipes };
          });
          
          return true;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to delete recipe';
          
          // Add to pending sync for offline support
          set(state => ({
            pendingSync: {
              ...state.pendingSync,
              delete: [...state.pendingSync.delete, id],
            },
            loading: false,
            error: errorMessage,
          }));
          
          return false;
        }
      },

      searchRecipes: async (params?: RecipeSearchParams) => {
        set({ loading: true, error: null });
        
        const searchParams = params || get().filters;
        
        try {
          if (!recipeClient) {
            throw new Error('Recipe client not initialized');
          }

          const results = await recipeClient.searchRecipes(searchParams);
          
          set({
            searchResults: results,
            recipes: searchParams.page === 1 ? results.recipes : [...get().recipes, ...results.recipes],
            loading: false,
          });
          
          // Cache all recipes
          results.recipes.forEach(recipe => get().cacheRecipe(recipe));
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to search recipes';
          set({ loading: false, error: errorMessage });
        }
      },

      setFilters: (filters: Partial<RecipeSearchParams>) => {
        set(state => ({
          filters: { ...state.filters, ...filters, page: 1 }, // Reset to page 1 when filters change
        }));
      },

      clearFilters: () => {
        set({ filters: initialFilters });
      },

      importRecipe: async (input: ImportRecipeInput) => {
        set({ loading: true, error: null });
        
        try {
          if (!recipeClient) {
            throw new Error('Recipe client not initialized');
          }

          const result = await recipeClient.importRecipe(input);
          
          if (result.success && result.recipe) {
            set(state => ({
              recipes: [result.recipe!, ...state.recipes],
              currentRecipe: result.recipe!,
              loading: false,
            }));
            
            get().cacheRecipe(result.recipe);
          } else {
            set({ loading: false });
          }
          
          return result;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to import recipe';
          set({ loading: false, error: errorMessage });
          return null;
        }
      },

      uploadRecipePhoto: async (recipeId: string, imageUri: string) => {
        set({ loading: true, error: null });
        
        try {
          if (!photoService) {
            throw new Error('Photo service not initialized');
          }

          const result = await photoService.uploadRecipePhoto(recipeId, imageUri);
          
          if (result.success && result.url) {
            // Update recipe in store with new image URL
            set(state => ({
              recipes: state.recipes.map(r => 
                r.id === recipeId 
                  ? { ...r, imageUrl: result.url }
                  : r
              ),
              currentRecipe: state.currentRecipe?.id === recipeId 
                ? { ...state.currentRecipe, imageUrl: result.url }
                : state.currentRecipe,
              loading: false,
            }));
            
            // Update cached recipe
            const cachedRecipe = get().getCachedRecipe(recipeId);
            if (cachedRecipe) {
              get().cacheRecipe({ ...cachedRecipe, imageUrl: result.url });
            }
          } else {
            set({ loading: false, error: result.error || 'Photo upload failed' });
          }
          
          return result;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Photo upload failed';
          set({ loading: false, error: errorMessage });
          return { success: false, error: errorMessage };
        }
      },

      deleteRecipePhoto: async (recipeId: string) => {
        set({ loading: true, error: null });
        
        try {
          if (!photoService) {
            throw new Error('Photo service not initialized');
          }

          const result = await photoService.deleteRecipePhoto(recipeId);
          
          if (result.success) {
            // Update recipe in store to remove image URL
            set(state => ({
              recipes: state.recipes.map(r => 
                r.id === recipeId 
                  ? { ...r, imageUrl: undefined }
                  : r
              ),
              currentRecipe: state.currentRecipe?.id === recipeId 
                ? { ...state.currentRecipe, imageUrl: undefined }
                : state.currentRecipe,
              loading: false,
            }));
            
            // Update cached recipe
            const cachedRecipe = get().getCachedRecipe(recipeId);
            if (cachedRecipe) {
              get().cacheRecipe({ ...cachedRecipe, imageUrl: undefined });
            }
          } else {
            set({ loading: false, error: result.error || 'Photo delete failed' });
          }
          
          return result;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Photo delete failed';
          set({ loading: false, error: errorMessage });
          return { success: false, error: errorMessage };
        }
      },

      syncPendingChanges: async () => {
        if (!offlineRepository) {
          // Fallback to legacy sync
          const { pendingSync } = get();
          
          // Sync creates
          for (const input of pendingSync.create) {
            await get().createRecipe(input);
          }
          
          // Sync updates
          for (const { id, input } of pendingSync.update) {
            await get().updateRecipe(id, input);
          }
          
          // Sync deletes
          for (const id of pendingSync.delete) {
            await get().deleteRecipe(id);
          }
          
          // Clear pending sync
          set({
            pendingSync: {
              create: [],
              update: [],
              delete: [],
            },
          });
          return;
        }

        // Enhanced sync with offline repository
        set({ syncInProgress: true, lastSyncAttempt: new Date(), syncErrors: [] });
        
        try {
          const result = await offlineRepository.syncPendingOperations();
          
          set({ 
            syncInProgress: false,
            syncErrors: result.errors,
          });
          
          // Clear legacy pending sync as well
          set({
            pendingSync: {
              create: [],
              update: [],
              delete: [],
            },
          });
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Sync failed';
          set({ 
            syncInProgress: false,
            syncErrors: [errorMessage],
          });
        }
      },

      invalidateCache: async (recipeId?: string) => {
        if (cacheService) {
          await cacheService.invalidateCache(recipeId);
        }
        
        if (recipeId) {
          // Also remove from legacy cache
          set(state => {
            const newCachedRecipes = { ...state.cachedRecipes };
            delete newCachedRecipes[recipeId];
            return { cachedRecipes: newCachedRecipes };
          });
        } else {
          // Clear all legacy cache
          set({ cachedRecipes: {} });
        }
      },

      warmCache: async (recipeIds?: string[]) => {
        if (!offlineRepository) return;
        
        if (recipeIds) {
          await cacheService?.warmCache(recipeIds);
        } else {
          await offlineRepository.warmCacheWithFrequentRecipes();
        }
      },

      cleanupExpiredCache: async () => {
        if (cacheService) {
          const cleanedCount = await cacheService.cleanupExpiredEntries();
          console.log(`Cleaned up ${cleanedCount} expired cache entries`);
        }
      },

      cacheRecipe: (recipe: Recipe) => {
        set(state => ({
          cachedRecipes: {
            ...state.cachedRecipes,
            [recipe.id]: recipe,
          },
        }));
      },

      getCachedRecipe: (id: string) => {
        return get().cachedRecipes[id] || null;
      },

      setCurrentRecipe: (recipe: Recipe | null) => {
        set({ currentRecipe: recipe });
      },

      clearError: () => {
        set({ error: null });
      },

      reset: () => {
        set(initialState);
      },
    }),
    {
      name: 'recipe-store',
      storage: createJSONStorage(() => AsyncStorage),
      partialize: (state) => ({
        cachedRecipes: state.cachedRecipes,
        pendingSync: state.pendingSync,
        filters: state.filters,
        lastSyncAttempt: state.lastSyncAttempt,
        syncErrors: state.syncErrors,
      }),
    }
  )
);