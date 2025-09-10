import type {
  Recipe,
  CreateRecipeInput,
  UpdateRecipeInput,
  RecipeSearchParams,
  RecipeSearchResponse,
  ImportRecipeInput,
  ImportRecipeResult
} from '@imkitchen/shared-types';

export interface APIError {
  error: string;
  details?: string;
}

export class RecipeClient {
  private baseUrl: string;
  private authToken?: string;

  constructor(baseUrl: string, authToken?: string) {
    this.baseUrl = baseUrl.replace(/\/$/, ''); // Remove trailing slash
    this.authToken = authToken;
  }

  setAuthToken(token: string) {
    this.authToken = token;
  }

  private async makeRequest<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    if (this.authToken) {
      headers.Authorization = `Bearer ${this.authToken}`;
    }

    const response = await fetch(url, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const error: APIError = await response.json();
      throw new Error(error.error || `HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json();
  }

  // Create a new recipe
  async createRecipe(input: CreateRecipeInput): Promise<Recipe> {
    return this.makeRequest<Recipe>('/api/v1/recipes', {
      method: 'POST',
      body: JSON.stringify(input),
    });
  }

  // Get a single recipe by ID
  async getRecipe(id: string): Promise<Recipe> {
    return this.makeRequest<Recipe>(`/api/v1/recipes/${id}`);
  }

  // Update an existing recipe
  async updateRecipe(id: string, input: UpdateRecipeInput): Promise<Recipe> {
    return this.makeRequest<Recipe>(`/api/v1/recipes/${id}`, {
      method: 'PUT',
      body: JSON.stringify(input),
    });
  }

  // Delete a recipe
  async deleteRecipe(id: string): Promise<{ message: string }> {
    return this.makeRequest<{ message: string }>(`/api/v1/recipes/${id}`, {
      method: 'DELETE',
    });
  }

  // Search and filter recipes
  async searchRecipes(params: RecipeSearchParams = {}): Promise<RecipeSearchResponse> {
    const searchParams = new URLSearchParams();
    
    // Add search parameters
    if (params.search) searchParams.append('search', params.search);
    if (params.mealType?.length) {
      params.mealType.forEach(type => searchParams.append('mealType[]', type));
    }
    if (params.complexity?.length) {
      params.complexity.forEach(complexity => searchParams.append('complexity[]', complexity));
    }
    if (params.maxPrepTime) searchParams.append('maxPrepTime', params.maxPrepTime.toString());
    if (params.maxCookTime) searchParams.append('maxCookTime', params.maxCookTime.toString());
    if (params.maxTotalTime) searchParams.append('maxTotalTime', params.maxTotalTime.toString());
    if (params.cuisineType) searchParams.append('cuisineType', params.cuisineType);
    if (params.dietaryLabels?.length) {
      params.dietaryLabels.forEach(label => searchParams.append('dietaryLabels[]', label));
    }
    if (params.page) searchParams.append('page', params.page.toString());
    if (params.limit) searchParams.append('limit', params.limit.toString());
    if (params.sortBy) searchParams.append('sortBy', params.sortBy);
    if (params.sortOrder) searchParams.append('sortOrder', params.sortOrder);

    const queryString = searchParams.toString();
    const endpoint = `/api/v1/recipes${queryString ? `?${queryString}` : ''}`;
    
    return this.makeRequest<RecipeSearchResponse>(endpoint);
  }

  // Import recipe from URL
  async importRecipe(input: ImportRecipeInput): Promise<ImportRecipeResult> {
    return this.makeRequest<ImportRecipeResult>('/api/v1/recipes/import', {
      method: 'POST',
      body: JSON.stringify(input),
    });
  }

  // Utility methods for common searches
  async getMyRecipes(page = 1, limit = 20): Promise<RecipeSearchResponse> {
    return this.searchRecipes({
      page,
      limit,
      sortBy: 'created_at',
      sortOrder: 'desc',
    });
  }

  async searchRecipesByMealType(
    mealType: ('breakfast' | 'lunch' | 'dinner' | 'snack')[],
    page = 1,
    limit = 20
  ): Promise<RecipeSearchResponse> {
    return this.searchRecipes({
      mealType,
      page,
      limit,
      sortBy: 'average_rating',
      sortOrder: 'desc',
    });
  }

  async getQuickRecipes(maxTotalTime = 30, page = 1, limit = 20): Promise<RecipeSearchResponse> {
    return this.searchRecipes({
      maxTotalTime,
      page,
      limit,
      sortBy: 'total_time',
      sortOrder: 'asc',
    });
  }

  async searchRecipesByIngredient(ingredient: string, page = 1, limit = 20): Promise<RecipeSearchResponse> {
    return this.searchRecipes({
      search: ingredient,
      page,
      limit,
      sortBy: 'average_rating',
      sortOrder: 'desc',
    });
  }
}