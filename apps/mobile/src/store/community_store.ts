import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import AsyncStorage from '@react-native-async-storage/async-storage';
import type {
  RecipeRating,
  RecipeRatingSubmission,
  CommunityRecipe,
  PaginatedRatingResponse,
  UserRatingHistoryResponse,
  CommunityRecipeFilters,
  CommunityRecipeResponse,
  FlagRatingRequest,
} from '@imkitchen/shared-types';
import { RatingService } from '../services/rating_service';

interface CommunityStore {
  // State
  communityRecipes: CommunityRecipe[];
  currentRecipeRatings: PaginatedRatingResponse | null;
  userRatingHistory: UserRatingHistoryResponse | null;
  userRatings: Record<string, RecipeRating>; // recipeId -> rating
  loading: {
    recipes: boolean;
    ratings: boolean;
    history: boolean;
    submit: boolean;
  };
  error: string | null;
  
  // Filters and pagination
  filters: CommunityRecipeFilters;
  currentPage: number;
  hasMoreRecipes: boolean;
  
  // Actions
  setRatingService: (service: RatingService) => void;
  
  // Community recipes
  fetchCommunityRecipes: (filters?: CommunityRecipeFilters, page?: number) => Promise<void>;
  searchCommunityRecipes: (query: string, filters?: Omit<CommunityRecipeFilters, 'searchQuery'>) => Promise<void>;
  getTrendingRecipes: (limit?: number) => Promise<CommunityRecipe[]>;
  getHighlyRatedRecipes: (minRatings?: number, limit?: number) => Promise<CommunityRecipe[]>;
  getRecommendedRecipes: (limit?: number) => Promise<CommunityRecipe[]>;
  
  // Ratings
  submitRating: (recipeId: string, rating: RecipeRatingSubmission) => Promise<void>;
  updateRating: (recipeId: string, rating: RecipeRatingSubmission) => Promise<void>;
  deleteRating: (recipeId: string) => Promise<void>;
  fetchRecipeRatings: (recipeId: string, page?: number) => Promise<void>;
  fetchUserRating: (recipeId: string) => Promise<RecipeRating | null>;
  
  // User rating history
  fetchUserRatingHistory: (page?: number) => Promise<void>;
  
  // Moderation
  flagRating: (ratingId: string, request: FlagRatingRequest) => Promise<void>;
  unflagRating: (ratingId: string) => Promise<void>;
  
  // Filters
  setFilters: (filters: Partial<CommunityRecipeFilters>) => void;
  clearFilters: () => void;
  
  // UI helpers
  clearError: () => void;
  reset: () => void;
}

const initialFilters: CommunityRecipeFilters = {
  sortBy: 'rating',
};

const initialState = {
  communityRecipes: [],
  currentRecipeRatings: null,
  userRatingHistory: null,
  userRatings: {},
  loading: {
    recipes: false,
    ratings: false,
    history: false,
    submit: false,
  },
  error: null,
  filters: initialFilters,
  currentPage: 1,
  hasMoreRecipes: true,
};

let ratingService: RatingService;

export const useCommunityStore = create<CommunityStore>()(
  persist(
    (set, get) => ({
      ...initialState,

      setRatingService: (service: RatingService) => {
        ratingService = service;
      },

      fetchCommunityRecipes: async (filters?: CommunityRecipeFilters, page = 1) => {
        set(state => ({
          loading: { ...state.loading, recipes: true },
          error: null,
        }));
        
        try {
          if (!ratingService) {
            throw new Error('Rating service not initialized');
          }

          const searchFilters = filters || get().filters;
          const limit = 20;
          
          // This would typically call a community recipes API endpoint
          // For now, we'll simulate the call since we don't have the frontend API client
          // const response = await communityService.getCommunityRecipes(searchFilters, page, limit);
          
          const response: CommunityRecipeResponse = {
            recipes: [],
            total: 0,
            page: page,
            limit: limit,
            totalPages: 0,
          };

          set(state => ({
            communityRecipes: page === 1 ? response.recipes : [...state.communityRecipes, ...response.recipes],
            currentPage: page,
            hasMoreRecipes: page < response.totalPages,
            filters: searchFilters,
            loading: { ...state.loading, recipes: false },
          }));
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to fetch community recipes';
          set(state => ({
            loading: { ...state.loading, recipes: false },
            error: errorMessage,
          }));
        }
      },

      searchCommunityRecipes: async (query: string, filters?) => {
        const searchFilters: CommunityRecipeFilters = {
          ...get().filters,
          ...filters,
          searchQuery: query,
        };
        
        await get().fetchCommunityRecipes(searchFilters, 1);
      },

      getTrendingRecipes: async (limit = 20) => {
        try {
          if (!ratingService) {
            throw new Error('Rating service not initialized');
          }

          // This would call the trending recipes endpoint
          // const recipes = await communityService.getTrendingRecipes(limit);
          const recipes: CommunityRecipe[] = [];
          
          return recipes;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to fetch trending recipes';
          set({ error: errorMessage });
          return [];
        }
      },

      getHighlyRatedRecipes: async (minRatings = 3, limit = 20) => {
        try {
          if (!ratingService) {
            throw new Error('Rating service not initialized');
          }

          // This would call the highly-rated recipes endpoint
          // const recipes = await communityService.getHighlyRatedRecipes(minRatings, limit);
          const recipes: CommunityRecipe[] = [];
          
          return recipes;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to fetch highly-rated recipes';
          set({ error: errorMessage });
          return [];
        }
      },

      getRecommendedRecipes: async (limit = 10) => {
        try {
          if (!ratingService) {
            throw new Error('Rating service not initialized');
          }

          // This would call the personalized recommendations endpoint
          // const recipes = await communityService.getRecommendedRecipes(limit);
          const recipes: CommunityRecipe[] = [];
          
          return recipes;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to fetch recommended recipes';
          set({ error: errorMessage });
          return [];
        }
      },

      submitRating: async (recipeId: string, rating: RecipeRatingSubmission) => {
        set(state => ({
          loading: { ...state.loading, submit: true },
          error: null,
        }));

        try {
          if (!ratingService) {
            throw new Error('Rating service not initialized');
          }

          const response = await ratingService.submitRating(recipeId, rating);
          
          set(state => ({
            userRatings: {
              ...state.userRatings,
              [recipeId]: response.rating,
            },
            loading: { ...state.loading, submit: false },
          }));

          // Update community recipes list if the rated recipe is in it
          set(state => ({
            communityRecipes: state.communityRecipes.map(recipe =>
              recipe.id === recipeId
                ? {
                    ...recipe,
                    averageRating: response.aggregatedStats.averageRating,
                    totalRatings: response.aggregatedStats.totalRatings,
                    ratingDistribution: response.aggregatedStats.ratingDistribution,
                  }
                : recipe
            ),
          }));
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to submit rating';
          set(state => ({
            loading: { ...state.loading, submit: false },
            error: errorMessage,
          }));
          throw error;
        }
      },

      updateRating: async (recipeId: string, rating: RecipeRatingSubmission) => {
        set(state => ({
          loading: { ...state.loading, submit: true },
          error: null,
        }));

        try {
          if (!ratingService) {
            throw new Error('Rating service not initialized');
          }

          const response = await ratingService.updateRating(recipeId, rating);
          
          set(state => ({
            userRatings: {
              ...state.userRatings,
              [recipeId]: response.rating,
            },
            loading: { ...state.loading, submit: false },
          }));

          // Update community recipes list if the rated recipe is in it
          set(state => ({
            communityRecipes: state.communityRecipes.map(recipe =>
              recipe.id === recipeId
                ? {
                    ...recipe,
                    averageRating: response.aggregatedStats.averageRating,
                    totalRatings: response.aggregatedStats.totalRatings,
                    ratingDistribution: response.aggregatedStats.ratingDistribution,
                  }
                : recipe
            ),
          }));
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to update rating';
          set(state => ({
            loading: { ...state.loading, submit: false },
            error: errorMessage,
          }));
          throw error;
        }
      },

      deleteRating: async (recipeId: string) => {
        set(state => ({
          loading: { ...state.loading, submit: true },
          error: null,
        }));

        try {
          if (!ratingService) {
            throw new Error('Rating service not initialized');
          }

          await ratingService.deleteRating(recipeId);
          
          set(state => {
            const newUserRatings = { ...state.userRatings };
            delete newUserRatings[recipeId];
            
            return {
              userRatings: newUserRatings,
              loading: { ...state.loading, submit: false },
            };
          });
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to delete rating';
          set(state => ({
            loading: { ...state.loading, submit: false },
            error: errorMessage,
          }));
          throw error;
        }
      },

      fetchRecipeRatings: async (recipeId: string, page = 1) => {
        set(state => ({
          loading: { ...state.loading, ratings: true },
          error: null,
        }));

        try {
          if (!ratingService) {
            throw new Error('Rating service not initialized');
          }

          const response = await ratingService.getRecipeRatings(recipeId, page, 20);
          
          set(state => ({
            currentRecipeRatings: page === 1 
              ? response 
              : {
                  ...response,
                  ratings: [...(state.currentRecipeRatings?.ratings || []), ...response.ratings],
                },
            loading: { ...state.loading, ratings: false },
          }));
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to fetch recipe ratings';
          set(state => ({
            loading: { ...state.loading, ratings: false },
            error: errorMessage,
          }));
        }
      },

      fetchUserRating: async (recipeId: string) => {
        try {
          if (!ratingService) {
            throw new Error('Rating service not initialized');
          }

          const rating = await ratingService.getUserRating(recipeId);
          
          if (rating) {
            set(state => ({
              userRatings: {
                ...state.userRatings,
                [recipeId]: rating,
              },
            }));
          }
          
          return rating;
        } catch (error) {
          // Don't set error for missing user ratings, it's expected
          return null;
        }
      },

      fetchUserRatingHistory: async (page = 1) => {
        set(state => ({
          loading: { ...state.loading, history: true },
          error: null,
        }));

        try {
          if (!ratingService) {
            throw new Error('Rating service not initialized');
          }

          const response = await ratingService.getUserRatingHistory(page, 20);
          
          set(state => ({
            userRatingHistory: page === 1 
              ? response 
              : {
                  ...response,
                  ratings: [...(state.userRatingHistory?.ratings || []), ...response.ratings],
                },
            loading: { ...state.loading, history: false },
          }));
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to fetch rating history';
          set(state => ({
            loading: { ...state.loading, history: false },
            error: errorMessage,
          }));
        }
      },

      flagRating: async (ratingId: string, request: FlagRatingRequest) => {
        try {
          if (!ratingService) {
            throw new Error('Rating service not initialized');
          }

          await ratingService.flagRating(ratingId, request);
          
          // Update the rating in current list to show it's flagged
          set(state => ({
            currentRecipeRatings: state.currentRecipeRatings
              ? {
                  ...state.currentRecipeRatings,
                  ratings: state.currentRecipeRatings.ratings.map(rating =>
                    rating.id === ratingId
                      ? { ...rating, flaggedByUser: true }
                      : rating
                  ),
                }
              : null,
          }));
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to flag rating';
          set({ error: errorMessage });
          throw error;
        }
      },

      unflagRating: async (ratingId: string) => {
        try {
          if (!ratingService) {
            throw new Error('Rating service not initialized');
          }

          await ratingService.unflagRating(ratingId);
          
          // Update the rating in current list to show it's not flagged
          set(state => ({
            currentRecipeRatings: state.currentRecipeRatings
              ? {
                  ...state.currentRecipeRatings,
                  ratings: state.currentRecipeRatings.ratings.map(rating =>
                    rating.id === ratingId
                      ? { ...rating, flaggedByUser: false }
                      : rating
                  ),
                }
              : null,
          }));
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to unflag rating';
          set({ error: errorMessage });
          throw error;
        }
      },

      setFilters: (filters: Partial<CommunityRecipeFilters>) => {
        set(state => ({
          filters: { ...state.filters, ...filters },
          currentPage: 1,
          hasMoreRecipes: true,
        }));
      },

      clearFilters: () => {
        set({
          filters: initialFilters,
          currentPage: 1,
          hasMoreRecipes: true,
        });
      },

      clearError: () => {
        set({ error: null });
      },

      reset: () => {
        set(initialState);
      },
    }),
    {
      name: 'community-store',
      storage: createJSONStorage(() => AsyncStorage),
      partialize: (state) => ({
        userRatings: state.userRatings,
        filters: state.filters,
      }),
    }
  )
);