import type {
  MealPlanResponse,
  CreateMealPlanInput,
  UpdateMealPlanInput,
  UpdateMealSlotInput,
  MealPlanFilters,
  MealPlanGenerationResponse,
  GenerateMealPlanInput,
  Recipe,
  DayOfWeek,
  MealType,
} from '@imkitchen/shared-types';
import type { SwapSuggestion, SwapFilters } from '../components/meal-plans/QuickSwapModal';
import { SwapSuggestionService } from './SwapSuggestionService';

const API_BASE_URL = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8080/api/v1';

class MealPlanService {
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${API_BASE_URL}${endpoint}`;
    
    const defaultOptions: RequestInit = {
      headers: {
        'Content-Type': 'application/json',
        // TODO: Add authentication headers
        // 'Authorization': `Bearer ${token}`,
      },
    };

    const config = { ...defaultOptions, ...options };

    try {
      const response = await fetch(url, config);
      
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || `HTTP ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error('API Request failed:', error);
      throw error;
    }
  }

  // Get meal plans for user with optional filters
  async getMealPlans(filters?: MealPlanFilters): Promise<{ mealPlans: MealPlanResponse[]; count: number }> {
    const queryParams = new URLSearchParams();
    
    if (filters?.weekStart) {
      queryParams.append('weekStart', filters.weekStart.toISOString().split('T')[0]);
    }
    
    if (filters?.weekEnd) {
      queryParams.append('weekEnd', filters.weekEnd.toISOString().split('T')[0]);
    }
    
    if (filters?.status) {
      queryParams.append('status', filters.status);
    }

    const query = queryParams.toString() ? `?${queryParams.toString()}` : '';
    return this.request<{ mealPlans: MealPlanResponse[]; count: number }>(`/meal-plans${query}`);
  }

  // Get a specific meal plan by ID
  async getMealPlan(id: string): Promise<MealPlanResponse> {
    return this.request<MealPlanResponse>(`/meal-plans/${id}`);
  }

  // Get meal plan by week start date
  async getMealPlanByWeek(weekStart: Date): Promise<MealPlanResponse> {
    const dateStr = weekStart.toISOString().split('T')[0];
    return this.request<MealPlanResponse>(`/meal-plans/week/${dateStr}`);
  }

  // Create a new meal plan
  async createMealPlan(input: CreateMealPlanInput): Promise<MealPlanResponse> {
    return this.request<MealPlanResponse>('/meal-plans', {
      method: 'POST',
      body: JSON.stringify(input),
    });
  }

  // Update an existing meal plan
  async updateMealPlan(id: string, input: UpdateMealPlanInput): Promise<MealPlanResponse> {
    return this.request<MealPlanResponse>(`/meal-plans/${id}`, {
      method: 'PUT',
      body: JSON.stringify(input),
    });
  }

  // Update a specific meal slot
  async updateMealSlot(
    mealPlanId: string,
    day: string,
    mealType: string,
    input: UpdateMealSlotInput
  ): Promise<MealPlanResponse> {
    return this.request<MealPlanResponse>(
      `/meal-plans/${mealPlanId}/entries/${day}/${mealType}`,
      {
        method: 'PUT',
        body: JSON.stringify(input),
      }
    );
  }

  // Remove a meal from a slot
  async deleteMealSlot(
    mealPlanId: string,
    day: string,
    mealType: string
  ): Promise<MealPlanResponse> {
    return this.request<MealPlanResponse>(
      `/meal-plans/${mealPlanId}/entries/${day}/${mealType}`,
      {
        method: 'DELETE',
      }
    );
  }

  // Delete a meal plan
  async deleteMealPlan(id: string): Promise<void> {
    await this.request<void>(`/meal-plans/${id}`, {
      method: 'DELETE',
    });
  }

  // Move a meal from one slot to another
  async moveMeal(
    mealPlanId: string,
    fromDay: string,
    fromMealType: string,
    toDay: string,
    toMealType: string
  ): Promise<MealPlanResponse> {
    // This is a convenience method that combines delete and update operations
    // In a real implementation, this might be a dedicated API endpoint
    
    // First, get the current meal data
    const mealPlan = await this.getMealPlan(mealPlanId);
    const fromDayMeals = mealPlan.populatedMeals[fromDay as keyof typeof mealPlan.populatedMeals];
    const sourceMeal = fromDayMeals?.find(meal => meal.mealType === fromMealType);
    
    if (!sourceMeal?.recipe) {
      throw new Error('Source meal not found');
    }

    // Remove from source slot
    await this.deleteMealSlot(mealPlanId, fromDay, fromMealType);
    
    // Add to target slot
    return this.updateMealSlot(mealPlanId, toDay, toMealType, {
      recipeId: sourceMeal.recipe.id,
      servings: sourceMeal.servings,
      notes: sourceMeal.notes,
    });
  }

  // Helper method to get the start of week (Monday) for a given date
  getWeekStart(date: Date): Date {
    const weekStart = new Date(date);
    const day = weekStart.getDay();
    const diff = weekStart.getDate() - day + (day === 0 ? -6 : 1); // Adjust when day is Sunday
    weekStart.setDate(diff);
    weekStart.setHours(0, 0, 0, 0);
    return weekStart;
  }

  // "Fill My Week" meal plan generation
  async generateWeeklyMealPlan(options?: {
    weekStartDate?: Date;
    maxPrepTimePerMeal?: number;
    preferredComplexityLevel?: 'simple' | 'moderate' | 'complex';
    cuisinePreferences?: string[];
    avoidRecipeIDs?: string[];
  }): Promise<MealPlanGenerationResponse> {
    const requestBody: any = {};
    
    if (options?.weekStartDate) {
      requestBody.weekStartDate = options.weekStartDate.toISOString();
    }
    
    if (options?.maxPrepTimePerMeal) {
      requestBody.maxPrepTimePerMeal = options.maxPrepTimePerMeal;
    }
    
    if (options?.preferredComplexityLevel) {
      requestBody.preferredComplexityLevel = options.preferredComplexityLevel;
    }
    
    if (options?.cuisinePreferences?.length) {
      requestBody.cuisinePreferences = options.cuisinePreferences;
    }
    
    if (options?.avoidRecipeIDs?.length) {
      requestBody.avoidRecipeIDs = options.avoidRecipeIDs;
    }

    return this.request<MealPlanGenerationResponse>('/meal-plans/generate', {
      method: 'POST',
      body: JSON.stringify(requestBody),
    });
  }

  // Initialize swap suggestion service
  private swapSuggestionService = new SwapSuggestionService();

  // Get swap suggestions for a meal
  async getSwapSuggestions(
    mealPlanId: string,
    day: DayOfWeek,
    mealType: MealType,
    filters: SwapFilters = {}
  ): Promise<SwapSuggestion[]> {
    // Get the current meal plan to identify the recipe to swap
    const mealPlan = await this.getMealPlan(mealPlanId);
    const dayMeals = mealPlan.populatedMeals[day];
    const currentMeal = dayMeals?.find(meal => meal.mealType === mealType);

    if (!currentMeal?.recipe) {
      throw new Error('No recipe found for this meal slot');
    }

    // Get candidate recipes from the API
    const candidateRecipes = await this.getCandidateRecipes(currentMeal.recipe, filters);
    
    // Use the suggestion service to calculate compatibility and generate suggestions
    return this.swapSuggestionService.getSuggestions(
      currentMeal.recipe,
      candidateRecipes,
      filters
    );
  }

  // Preview shopping list changes for a recipe swap
  async previewShoppingListChanges(
    mealPlanId: string,
    day: DayOfWeek,
    mealType: MealType,
    newRecipeId: string
  ): Promise<{ itemsAdded: number; itemsRemoved: number; estimatedCostChange: number }> {
    return this.request<{ itemsAdded: number; itemsRemoved: number; estimatedCostChange: number }>(
      `/meal-plans/${mealPlanId}/entries/${day}/${mealType}/swap-preview`,
      {
        method: 'POST',
        body: JSON.stringify({ newRecipeId }),
      }
    );
  }

  // Perform recipe swap
  async swapRecipe(
    mealPlanId: string,
    day: DayOfWeek,
    mealType: MealType,
    newRecipeId: string,
    changeReason?: string
  ): Promise<MealPlanResponse> {
    const requestBody = {
      recipeId: newRecipeId,
      changeReason: changeReason || 'Quick swap replacement',
      updateShoppingList: true,
    };

    return this.request<MealPlanResponse>(
      `/meal-plans/${mealPlanId}/entries/${day}/${mealType}`,
      {
        method: 'PUT',
        body: JSON.stringify(requestBody),
      }
    );
  }

  // Toggle meal lock status
  async toggleMealLock(
    mealPlanId: string,
    day: DayOfWeek,
    mealType: MealType,
    isLocked: boolean
  ): Promise<MealPlanResponse> {
    return this.request<MealPlanResponse>(
      `/meal-plans/${mealPlanId}/entries/${day}/${mealType}`,
      {
        method: 'PUT',
        body: JSON.stringify({ isLocked }),
      }
    );
  }

  // Reorder meal from one slot to another
  async reorderMeal(
    mealPlanId: string,
    sourceDay: DayOfWeek,
    sourceMealType: MealType,
    targetDay: DayOfWeek,
    targetMealType: MealType
  ): Promise<MealPlanResponse> {
    return this.request<MealPlanResponse>(
      `/meal-plans/${mealPlanId}/reorder`,
      {
        method: 'POST',
        body: JSON.stringify({
          sourceDay,
          sourceMealType,
          targetDay,
          targetMealType,
        }),
      }
    );
  }

  // Get candidate recipes for swapping (mock implementation)
  private async getCandidateRecipes(
    originalRecipe: Recipe,
    filters: SwapFilters
  ): Promise<Recipe[]> {
    // In a real implementation, this would call a recipe search API
    // For now, we'll return a mock set of recipes
    
    const queryParams = new URLSearchParams();
    
    // Add meal type filter based on original recipe
    originalRecipe.mealType.forEach(mt => queryParams.append('mealType', mt));
    
    if (filters.maxPrepTime) {
      queryParams.append('maxPrepTime', filters.maxPrepTime.toString());
    }
    
    if (filters.complexity) {
      queryParams.append('complexity', filters.complexity);
    }
    
    if (filters.cuisine) {
      queryParams.append('cuisine', filters.cuisine);
    }
    
    if (filters.excludeRecipeIds?.length) {
      filters.excludeRecipeIds.forEach(id => queryParams.append('excludeId', id));
    }

    try {
      return this.request<Recipe[]>(`/recipes/search?${queryParams.toString()}`);
    } catch (error) {
      console.error('Failed to fetch candidate recipes:', error);
      
      // Return mock data for development
      return this.getMockCandidateRecipes(originalRecipe, filters);
    }
  }

  // Mock candidate recipes for development
  private getMockCandidateRecipes(originalRecipe: Recipe, filters: SwapFilters): Recipe[] {
    // This is mock data for development purposes
    const mockRecipes: Recipe[] = [
      {
        id: 'mock-recipe-1',
        title: 'Quick Vegetable Stir Fry',
        prepTime: 15,
        cookTime: 10,
        totalTime: 25,
        complexity: 'simple',
        mealType: originalRecipe.mealType,
        servings: 4,
        ingredients: [
          { name: 'Mixed vegetables', amount: '2 cups', unit: 'cups' },
          { name: 'Soy sauce', amount: '2 tbsp', unit: 'tablespoons' },
          { name: 'Garlic', amount: '2 cloves', unit: 'cloves' },
        ],
        instructions: ['Heat oil', 'Add vegetables', 'Stir fry', 'Season and serve'],
        dietaryLabels: ['vegetarian'],
        averageRating: 4.2,
        totalRatings: 35,
        cuisineType: 'Asian',
        createdAt: new Date(),
        updatedAt: new Date(),
      },
      {
        id: 'mock-recipe-2',
        title: 'Mediterranean Quinoa Bowl',
        prepTime: 20,
        cookTime: 15,
        totalTime: 35,
        complexity: 'moderate',
        mealType: originalRecipe.mealType,
        servings: 4,
        ingredients: [
          { name: 'Quinoa', amount: '1 cup', unit: 'cups' },
          { name: 'Cherry tomatoes', amount: '1 cup', unit: 'cups' },
          { name: 'Feta cheese', amount: '1/2 cup', unit: 'cups' },
        ],
        instructions: ['Cook quinoa', 'Prepare vegetables', 'Combine ingredients', 'Serve'],
        dietaryLabels: ['vegetarian', 'gluten-free'],
        averageRating: 4.5,
        totalRatings: 48,
        cuisineType: 'Mediterranean',
        createdAt: new Date(),
        updatedAt: new Date(),
      },
    ];

    return mockRecipes.filter(recipe => {
      if (filters.maxPrepTime && (recipe.prepTime || 0) > filters.maxPrepTime) {
        return false;
      }
      if (filters.complexity && recipe.complexity !== filters.complexity) {
        return false;
      }
      if (filters.cuisine && recipe.cuisineType !== filters.cuisine) {
        return false;
      }
      if (filters.excludeRecipeIds?.includes(recipe.id)) {
        return false;
      }
      return true;
    });
  }

  // Helper method to format date for API calls
  static formatDateForAPI(date: Date): string {
    return date.toISOString().split('T')[0];
  }
}

export const mealPlanService = new MealPlanService();