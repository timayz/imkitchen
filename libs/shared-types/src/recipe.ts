// Recipe-related TypeScript interfaces and types
// Shared between mobile app and backend API

export interface RecipeIngredient {
  name: string;
  amount: number;
  unit: string;
  category: 'produce' | 'dairy' | 'pantry' | 'protein' | 'other';
}

export interface RecipeInstruction {
  stepNumber: number;
  instruction: string;
  estimatedMinutes?: number;
}

export interface RecipeNutrition {
  calories?: number;
  protein?: number; // grams
  carbs?: number;   // grams
  fat?: number;     // grams
  fiber?: number;   // grams
  sugar?: number;   // grams
  sodium?: number;  // mg
}

export interface Recipe {
  id: string;
  externalId?: string;
  externalSource?: 'spoonacular' | 'edamam' | 'user_generated';
  
  // Basic Recipe Info
  title: string;
  description?: string;
  imageUrl?: string;
  sourceUrl?: string;
  
  // Timing
  prepTime: number;  // minutes
  cookTime: number;  // minutes
  totalTime: number; // calculated: prep + cook
  
  // Classification
  mealType: ('breakfast' | 'lunch' | 'dinner' | 'snack')[];
  complexity: 'simple' | 'moderate' | 'complex';
  cuisineType?: string;
  
  // Recipe Data
  servings: number;
  ingredients: RecipeIngredient[];
  instructions: RecipeInstruction[];
  
  // Nutritional Information
  nutrition?: RecipeNutrition;
  dietaryLabels: string[]; // vegetarian, vegan, gluten-free, etc.
  
  // Quality Metrics
  averageRating: number;
  totalRatings: number;
  difficultyScore?: number; // 1-10 scale
  
  // Metadata
  createdAt: Date;
  updatedAt: Date;
  deletedAt?: Date;
}

// Input DTOs for recipe creation and updates
export interface CreateRecipeInput {
  title: string;
  description?: string;
  prepTime: number;
  cookTime: number;
  mealType: ('breakfast' | 'lunch' | 'dinner' | 'snack')[];
  complexity: 'simple' | 'moderate' | 'complex';
  cuisineType?: string;
  servings: number;
  ingredients: RecipeIngredient[];
  instructions: RecipeInstruction[];
  dietaryLabels?: string[];
  imageUrl?: string;
  sourceUrl?: string;
}

export interface UpdateRecipeInput {
  title?: string;
  description?: string;
  prepTime?: number;
  cookTime?: number;
  mealType?: ('breakfast' | 'lunch' | 'dinner' | 'snack')[];
  complexity?: 'simple' | 'moderate' | 'complex';
  cuisineType?: string;
  servings?: number;
  ingredients?: RecipeIngredient[];
  instructions?: RecipeInstruction[];
  dietaryLabels?: string[];
  imageUrl?: string;
  sourceUrl?: string;
}

// Search and filter interfaces
export interface RecipeFilters {
  mealType?: ('breakfast' | 'lunch' | 'dinner' | 'snack')[];
  complexity?: ('simple' | 'moderate' | 'complex')[];
  maxPrepTime?: number;
  maxCookTime?: number;
  maxTotalTime?: number;
  cuisineType?: string;
  dietaryLabels?: string[];
  search?: string; // full-text search
}

export interface RecipeSearchParams extends RecipeFilters {
  page?: number;
  limit?: number;
  sortBy?: 'created_at' | 'updated_at' | 'total_time' | 'average_rating';
  sortOrder?: 'asc' | 'desc';
}

export interface RecipeSearchResponse {
  recipes: Recipe[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

// Import-related interfaces
export interface ImportRecipeInput {
  url: string;
  overrideFields?: Partial<CreateRecipeInput>;
}

export interface ImportRecipeResult {
  success: boolean;
  recipe?: Recipe;
  error?: string;
  warnings?: string[];
}