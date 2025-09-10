import type { 
  RecipeImportRequest,
  RecipeImportResponse,
  ImportConflict,
  RecipeImport,
  ImportStats
} from '@imkitchen/shared-types';

export class RecipeImportService {
  private baseURL: string;
  private token: string | null = null;

  constructor(baseURL: string) {
    this.baseURL = baseURL;
  }

  setAuthToken(token: string) {
    this.token = token;
  }

  private getAuthHeaders() {
    return {
      'Content-Type': 'application/json',
      ...(this.token && { Authorization: `Bearer ${this.token}` }),
    };
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Network error' }));
      throw new Error(errorData.error || `HTTP ${response.status}: ${response.statusText}`);
    }
    return response.json();
  }

  /**
   * Import a community recipe to personal collection
   */
  async importCommunityRecipe(request: RecipeImportRequest): Promise<RecipeImportResponse> {
    const response = await fetch(`${this.baseURL}/api/v1/recipes/import`, {
      method: 'POST',
      headers: this.getAuthHeaders(),
      body: JSON.stringify(request),
    });

    return this.handleResponse<RecipeImportResponse>(response);
  }

  /**
   * Check for import conflicts before importing
   */
  async checkImportConflict(communityRecipeId: string): Promise<{ hasConflict: boolean; conflict?: ImportConflict }> {
    const response = await fetch(`${this.baseURL}/api/v1/recipes/import/check/${communityRecipeId}`, {
      method: 'GET',
      headers: this.getAuthHeaders(),
    });

    return this.handleResponse<{ hasConflict: boolean; conflict?: ImportConflict }>(response);
  }

  /**
   * Get user's import history with pagination
   */
  async getImportHistory(page: number = 1, limit: number = 20): Promise<{
    imports: RecipeImport[];
    pagination: {
      page: number;
      limit: number;
      total: number;
      hasNext: boolean;
      hasPrevious: boolean;
    };
  }> {
    const params = new URLSearchParams({
      page: page.toString(),
      limit: limit.toString(),
    });

    const response = await fetch(`${this.baseURL}/api/v1/recipes/import/history?${params}`, {
      method: 'GET',
      headers: this.getAuthHeaders(),
    });

    return this.handleResponse(response);
  }

  /**
   * Get user's import statistics
   */
  async getImportStats(): Promise<ImportStats> {
    const response = await fetch(`${this.baseURL}/api/v1/recipes/import/stats`, {
      method: 'GET',
      headers: this.getAuthHeaders(),
    });

    return this.handleResponse<ImportStats>(response);
  }

  /**
   * Quick import with minimal customization
   */
  async quickImport(communityRecipeId: string, preserveAttribution: boolean = true): Promise<RecipeImportResponse> {
    return this.importCommunityRecipe({
      communityRecipeId,
      preserveAttribution,
    });
  }

  /**
   * Import with customizations
   */
  async importWithCustomizations(
    communityRecipeId: string,
    customizations: {
      title?: string;
      notes?: string;
      servingAdjustment?: number;
    },
    preserveAttribution: boolean = true
  ): Promise<RecipeImportResponse> {
    return this.importCommunityRecipe({
      communityRecipeId,
      customizations,
      preserveAttribution,
    });
  }

  /**
   * Batch import multiple recipes (with rate limiting consideration)
   */
  async batchImport(
    recipeIds: string[],
    preserveAttribution: boolean = true,
    onProgress?: (completed: number, total: number) => void
  ): Promise<{ successful: RecipeImportResponse[]; failed: { id: string; error: string }[] }> {
    const successful: RecipeImportResponse[] = [];
    const failed: { id: string; error: string }[] = [];

    for (let i = 0; i < recipeIds.length; i++) {
      try {
        const result = await this.quickImport(recipeIds[i], preserveAttribution);
        successful.push(result);
      } catch (error) {
        failed.push({
          id: recipeIds[i],
          error: error instanceof Error ? error.message : 'Unknown error',
        });
      }

      // Call progress callback if provided
      onProgress?.(i + 1, recipeIds.length);

      // Add delay between requests to respect rate limits
      if (i < recipeIds.length - 1) {
        await new Promise(resolve => setTimeout(resolve, 200)); // 200ms delay
      }
    }

    return { successful, failed };
  }

  /**
   * Resolve import conflict by choosing resolution strategy
   */
  async resolveConflict(
    communityRecipeId: string,
    resolution: 'rename' | 'merge' | 'replace' | 'cancel',
    customizations?: {
      title?: string;
      notes?: string;
      servingAdjustment?: number;
    }
  ): Promise<RecipeImportResponse | null> {
    if (resolution === 'cancel') {
      return null;
    }

    // For rename strategy, modify the title
    if (resolution === 'rename' && !customizations?.title) {
      const timestamp = new Date().toISOString().slice(0, 10);
      customizations = {
        ...customizations,
        title: `Imported Recipe - ${timestamp}`,
      };
    }

    return this.importCommunityRecipe({
      communityRecipeId,
      customizations,
      preserveAttribution: true,
    });
  }
}

// Export a singleton instance
let importServiceInstance: RecipeImportService | null = null;

export const getRecipeImportService = (baseURL?: string): RecipeImportService => {
  if (!importServiceInstance) {
    if (!baseURL) {
      throw new Error('RecipeImportService must be initialized with baseURL');
    }
    importServiceInstance = new RecipeImportService(baseURL);
  }
  return importServiceInstance;
};

export default RecipeImportService;