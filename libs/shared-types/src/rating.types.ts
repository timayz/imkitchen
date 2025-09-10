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

// Community recipe types
export interface CommunityRecipe {
  id: string;
  title: string;
  description?: string;
  imageURL?: string;
  prepTime: number;
  cookTime: number;
  totalTime: number;
  complexity: RecipeComplexity;
  cuisineType?: string;
  mealType: MealType[];
  servings: number;
  averageRating: number;
  totalRatings: number;
  ratingDistribution: RatingDistribution;
  userTags: string[];
  trendingScore: number;
  importedFrom?: {
    originalId: string;
    contributorId: string;
    importDate: Date;
  };
  contributorName?: string;
  importCount: number;
  isPopular: boolean;
  isTrending: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface CommunityRecipeFilters {
  sortBy?: CommunitySortOption;
  minRating?: number;
  maxPrepTime?: number;
  searchQuery?: string;
  mealTypes?: MealType[];
  complexities?: RecipeComplexity[];
  cuisineTypes?: string[];
  dietaryLabels?: string[];
  tags?: string[];
  category?: RecipeCategory;
  timeFilter?: 'day' | 'week' | 'month';
}

export interface CommunityRecipeResponse {
  recipes: CommunityRecipe[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

export type RecipeCategory = 'vegetarian' | 'vegan' | 'quick_meals' | 'comfort_food' | 'healthy' | 'budget_friendly' | 'family_friendly';

export interface RecipeImportRequest {
  communityRecipeId: string;
  customizations?: {
    title?: string;
    notes?: string;
    servingAdjustment?: number;
  };
  preserveAttribution: boolean;
}

export interface RecipeImportResponse {
  success: boolean;
  personalRecipeId?: string;
  message: string;
  attribution?: {
    originalContributor: string;
    importDate: Date;
    communityMetrics: {
      totalImports: number;
      averageRating: number;
    };
  };
}

// Pagination helper types
export interface PaginatedRatingResponse {
  ratings: RecipeRating[];
  total: number;
  page: number;
  limit: number;
  hasNext: boolean;
  hasPrevious: boolean;
}

export interface UserRatingHistoryResponse {
  ratings: UserRatingHistory[];
  total: number;
  page: number;
  limit: number;
  hasNext: boolean;
  hasPrevious: boolean;
}

export interface RecipeRatingSubmission {
  overallRating: number;
  reviewText?: string;
  difficultyRating?: number;
  tasteRating?: number;
  wouldMakeAgain?: boolean;
  actualPrepTime?: number;
  actualCookTime?: number;
}

export interface FlagRatingRequest {
  reason: 'spam' | 'inappropriate' | 'offensive' | 'misleading' | 'other';
  description?: string;
}

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

// Tag Management Types (Task 4)
export interface TagSuggestion {
  tag: string;
  confidence: number;
  usageCount: number;
  category: string;
}

export interface PopularTag {
  tag: string;
  usageCount: number;
  category: string;
  trendingUp: boolean;
  description?: string;
}

export interface RecipeTag {
  tag: string;
  voteCount: number;
  userVoted: boolean;
  confidence: number;
}

export interface TagStat {
  usageCount: number;
  trending: boolean;
  category: string;
  confidence: number;
}

export interface InvalidTagResult {
  tag: string;
  reason: string;
}

export interface TagValidationResult {
  validTags: string[];
  invalidTags: InvalidTagResult[];
}

export interface TagUpdateResponse {
  recipeId: string;
  updatedTags: string[];
  message: string;
}

export interface RecipeTagsResponse {
  recipeId: string;
  userTags: string[];
  communityTags: RecipeTag[];
  tagStats: Record<string, TagStat>;
}

export interface TagVoteResponse {
  tag: string;
  voteCount: number;
  userVoted: boolean;
  message: string;
}

export interface TagSuggestionsRequest {
  query: string;
  recipeId?: string;
  exclude?: string[];
  limit?: number;
}

export interface TagSuggestionsResponse {
  suggestions: TagSuggestion[];
}

export interface PopularTagsResponse {
  tags: PopularTag[];
}

export interface CommunityTag {
  tag: string;
  voteCount: number;
  userVoted: boolean;
  confidence: number;
}

// Tag Management Service Response Types
export interface TagCategoriesResponse {
  categories: Record<string, string[]>;
}

export interface UserTagStatsResponse {
  totalTags: number;
  mostUsedTags: Array<{ tag: string; count: number }>;
  recentTags: string[];
}

export interface TagCleanupResponse {
  removedTags: string[];
}

export interface TagExportResponse {
  recipes: Array<{
    recipeId: string;
    title: string;
    tags: string[];
  }>;
}

export interface BatchTagUpdateResponse {
  success: boolean;
  results: TagUpdateResponse[];
}

// Enhanced Recipe Attribution Types (Task 5)
export interface RecipeAttribution {
  id: string;
  recipeId: string;
  originalContributorId: string;
  originalContributor: string;
  importDate: Date;
  preserveAttribution: boolean;
  customizations: string[];
  communityMetrics: CommunityMetrics;
  recipeChain: RecipeChainLink[];
  engagementStats?: EngagementStats;
}

export interface CommunityMetrics {
  totalImports: number;
  averageRating: number;
  totalRatings: number;
  trendingScore: number;
  popularityRank?: number;
}

export interface RecipeChainLink {
  contributorId: string;
  contributorName: string;
  adaptedAt: Date;
  recipeId: string;
}

export interface EngagementStats {
  weeklyViews: number;
  savesToMealPlans: number;
  socialShares: number;
}

export interface ContributorProfile {
  id: string;
  username: string;
  displayName: string;
  avatarUrl?: string;
  totalRecipes: number;
  averageRating: number;
  totalImports: number;
  joinedAt: Date;
  badges: ContributorBadge[];
  achievements: ContributorAchievement[];
  bio?: string;
  location?: string;
  website?: string;
}

export interface ContributorBadge {
  id: string;
  name: string;
  description: string;
  emoji: string;
  earnedAt: Date;
}

export interface ContributorAchievement {
  id: string;
  title: string;
  description: string;
  emoji: string;
  earnedAt: Date;
  category: string;
  points: number;
}

export interface CommunityMetricsData {
  overview?: MetricsOverview;
  popularity?: PopularityMetrics;
  engagement?: EngagementMetrics;
  geographic?: GeographicMetrics;
  achievements?: ContributorAchievement[];
}

export interface MetricsOverview {
  totalImports: number;
  averageRating: number;
  uniqueUsers: number;
  engagementRate: number;
  importTrend?: number;
  ratingTrend?: number;
  reachTrend?: number;
  engagementTrend?: number;
}

export interface PopularityMetrics {
  weeklyRank: number;
  trendingScore: number;
  viralityIndex: number;
  featuredIn?: FeatureHighlight[];
}

export interface FeatureHighlight {
  title: string;
  date: Date;
  type: string;
}

export interface EngagementMetrics {
  views: number;
  saves: number;
  shares: number;
  comments: number;
  ratings: number;
}

export interface GeographicMetrics {
  countries: number;
  cities: number;
  topRegions?: RegionMetric[];
}

export interface RegionMetric {
  name: string;
  flag: string;
  count: number;
}

export type MetricsTimeframe = 'day' | 'week' | 'month' | 'quarter' | 'year' | 'all';

// Attribution Service Response Types
export interface RecipeAttributionResponse {
  attribution: RecipeAttribution;
}

export interface ContributorProfileResponse {
  profile: ContributorProfile;
}

export interface CommunityMetricsResponse {
  metrics: CommunityMetricsData;
}

export interface AchievementCreationResponse {
  success: boolean;
  achievement: ContributorAchievement;
  message: string;
}