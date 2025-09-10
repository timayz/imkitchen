/**
 * Shared TypeScript interfaces for recipe rating system
 * Used by both frontend and backend components
 */

export interface RecipeRating {
  id: string;
  recipeId: string;
  userId: string;
  overallRating: number; // 1-5 stars
  difficultyRating?: number; // 1-5 stars
  tasteRating?: number; // 1-5 stars
  reviewText?: string; // Optional review text, max 500 characters
  wouldMakeAgain?: boolean;
  actualPrepTime?: number; // minutes
  actualCookTime?: number; // minutes
  mealPlanId?: string;
  cookingContext?: CookingContext;
  moderationStatus: ModerationStatus;
  flaggedReason?: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface RatingDistribution {
  '1': number;
  '2': number;
  '3': number;
  '4': number;
  '5': number;
}

export interface RecipeWithRatings {
  id: string;
  title: string;
  description?: string;
  imageUrl?: string;
  prepTime: number;
  cookTime: number;
  complexity: RecipeComplexity;
  mealType: MealType[];
  averageRating: number;
  totalRatings: number;
  ratingDistribution: RatingDistribution;
  isPublic: boolean;
  isCommunity: boolean;
  recommendationScore?: number;
  eligibleForRecommendations?: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface RatingSubmission {
  recipeId: string;
  overallRating: number; // 1-5 stars, required
  reviewText?: string; // Optional, max 500 characters
  difficultyRating?: number; // 1-5 stars
  tasteRating?: number; // 1-5 stars
  wouldMakeAgain?: boolean;
  actualPrepTime?: number;
  actualCookTime?: number;
  cookingContext?: CookingContext;
}

export interface RatingUpdateRequest {
  overallRating?: number;
  reviewText?: string;
  difficultyRating?: number;
  tasteRating?: number;
  wouldMakeAgain?: boolean;
  actualPrepTime?: number;
  actualCookTime?: number;
  cookingContext?: CookingContext;
}

export interface UserRatingHistory {
  id: string;
  userId: string;
  recipeId: string;
  recipeTitle: string;
  recipeImageUrl?: string;
  overallRating: number;
  reviewText?: string;
  wouldMakeAgain?: boolean;
  moderationStatus: ModerationStatus;
  createdAt: Date;
  updatedAt: Date;
}

export interface RatingsListResponse {
  ratings: RecipeRating[];
  pagination: {
    total: number;
    page: number;
    limit: number;
    hasNext: boolean;
    hasPrevious: boolean;
  };
  aggregates: {
    averageRating: number;
    totalRatings: number;
    ratingDistribution: RatingDistribution;
  };
}

export interface CommunityRecipesResponse {
  recipes: RecipeWithRatings[];
  pagination: {
    total: number;
    page: number;
    limit: number;
    hasNext: boolean;
    hasPrevious: boolean;
  };
  filters: {
    sortBy: CommunitySortOption;
    minRating?: number;
    maxPrepTime?: number;
    mealTypes?: MealType[];
    complexities?: RecipeComplexity[];
  };
}

// Enums and string literal types
export type ModerationStatus = 'pending' | 'approved' | 'rejected' | 'flagged';

export type CookingContext = 'weeknight' | 'weekend' | 'special_occasion';

export type RecipeComplexity = 'simple' | 'moderate' | 'complex';

export type MealType = 'breakfast' | 'lunch' | 'dinner' | 'snack';

export type CommunitySortOption = 'rating' | 'recent' | 'popular' | 'trending';

// API response types
export interface RatingResponse {
  rating: RecipeRating;
  message: string;
}

export interface RatingErrorResponse {
  error: string;
  code: RatingErrorCode;
  message: string;
  details?: Record<string, any>;
}

export type RatingErrorCode = 
  | 'DUPLICATE_RATING'
  | 'INVALID_RATING_VALUE' 
  | 'RECIPE_NOT_FOUND'
  | 'AUTHENTICATION_REQUIRED'
  | 'INAPPROPRIATE_CONTENT'
  | 'RATE_LIMIT_EXCEEDED'
  | 'VALIDATION_ERROR'
  | 'SERVER_ERROR';

// Frontend state types
export interface RatingState {
  isSubmitting: boolean;
  isLoading: boolean;
  error: string | null;
  optimisticRating: RecipeRating | null;
}

export interface CommunityState {
  recipes: RecipeWithRatings[];
  isLoading: boolean;
  error: string | null;
  pagination: {
    page: number;
    limit: number;
    total: number;
    hasNext: boolean;
    hasPrevious: boolean;
  };
  filters: {
    sortBy: CommunitySortOption;
    minRating?: number;
    searchQuery?: string;
    mealTypes: MealType[];
    complexities: RecipeComplexity[];
  };
}

export interface UserRatingsState {
  ratings: UserRatingHistory[];
  isLoading: boolean;
  error: string | null;
  pagination: {
    page: number;
    limit: number;
    total: number;
    hasNext: boolean;
    hasPrevious: boolean;
  };
}

// Validation types
export interface RatingValidationRules {
  overallRating: {
    required: true;
    min: 1;
    max: 5;
  };
  reviewText: {
    maxLength: 500;
    profanityFilter: true;
  };
  difficultyRating: {
    min: 1;
    max: 5;
  };
  tasteRating: {
    min: 1;
    max: 5;
  };
}