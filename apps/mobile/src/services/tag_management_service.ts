import { ApiClient } from './api_client';
import type {
  TagSuggestion,
  PopularTag,
  RecipeTag,
  TagValidationResult,
  TagUpdateResponse,
  RecipeTagsResponse,
  TagVoteResponse,
} from '@imkitchen/shared-types';

export class TagManagementService {
  private apiClient: ApiClient;

  constructor() {
    this.apiClient = new ApiClient();
  }

  /**
   * Get tag suggestions based on query and context
   */
  async getTagSuggestions(
    query: string,
    recipeId?: string,
    exclude: string[] = [],
    limit: number = 10
  ): Promise<TagSuggestion[]> {
    try {
      const response = await this.apiClient.post<{ suggestions: TagSuggestion[] }>(
        '/api/v1/tags/suggestions',
        {
          query,
          recipe_id: recipeId,
          exclude,
          limit,
        }
      );

      return response.suggestions;
    } catch (error) {
      console.error('Failed to get tag suggestions:', error);
      throw new Error('Failed to get tag suggestions');
    }
  }

  /**
   * Get popular tags with optional filtering
   */
  async getPopularTags(
    limit: number = 20,
    category?: string,
    period: 'day' | 'week' | 'month' | 'all' = 'week'
  ): Promise<PopularTag[]> {
    try {
      const params = new URLSearchParams({
        limit: limit.toString(),
        period,
      });

      if (category) {
        params.append('category', category);
      }

      const response = await this.apiClient.get<{ tags: PopularTag[] }>(
        `/api/v1/tags/popular?${params.toString()}`
      );

      return response.tags;
    } catch (error) {
      console.error('Failed to get popular tags:', error);
      throw new Error('Failed to get popular tags');
    }
  }

  /**
   * Validate tags before adding them
   */
  async validateTags(tags: string[]): Promise<TagValidationResult> {
    try {
      const response = await this.apiClient.post<TagValidationResult>(
        '/api/v1/tags/validate',
        { tags }
      );

      return response;
    } catch (error) {
      console.error('Failed to validate tags:', error);
      throw new Error('Failed to validate tags');
    }
  }

  /**
   * Update recipe tags (add, remove, or replace)
   */
  async updateRecipeTags(
    recipeId: string,
    tags: string[],
    action: 'add' | 'remove' | 'replace'
  ): Promise<TagUpdateResponse> {
    try {
      const response = await this.apiClient.put<TagUpdateResponse>(
        `/api/v1/recipes/${recipeId}/tags`,
        {
          tags,
          action,
        }
      );

      return response;
    } catch (error) {
      console.error('Failed to update recipe tags:', error);
      throw new Error('Failed to update recipe tags');
    }
  }

  /**
   * Get all tags for a recipe (user and community)
   */
  async getRecipeTags(recipeId: string): Promise<RecipeTagsResponse> {
    try {
      const response = await this.apiClient.get<RecipeTagsResponse>(
        `/api/v1/recipes/${recipeId}/tags`
      );

      return response;
    } catch (error) {
      console.error('Failed to get recipe tags:', error);
      throw new Error('Failed to get recipe tags');
    }
  }

  /**
   * Vote on a community tag
   */
  async voteOnTag(
    recipeId: string,
    tag: string,
    action: 'upvote' | 'downvote' | 'remove'
  ): Promise<TagVoteResponse> {
    try {
      const response = await this.apiClient.post<TagVoteResponse>(
        `/api/v1/recipes/${recipeId}/tags/vote`,
        {
          tag,
          action,
        }
      );

      return response;
    } catch (error) {
      console.error('Failed to vote on tag:', error);
      throw new Error('Failed to vote on tag');
    }
  }

  /**
   * Search recipes by tags
   */
  async searchRecipesByTags(
    tags: string[],
    operator: 'AND' | 'OR' = 'AND',
    limit: number = 50,
    offset: number = 0
  ): Promise<any[]> {
    try {
      const params = new URLSearchParams({
        tags: tags.join(','),
        operator,
        limit: limit.toString(),
        offset: offset.toString(),
      });

      const response = await this.apiClient.get<{ recipes: any[] }>(
        `/api/v1/recipes/search/tags?${params.toString()}`
      );

      return response.recipes;
    } catch (error) {
      console.error('Failed to search recipes by tags:', error);
      throw new Error('Failed to search recipes by tags');
    }
  }

  /**
   * Get tag categories for organizing tags
   */
  async getTagCategories(): Promise<Record<string, string[]>> {
    try {
      const response = await this.apiClient.get<{ categories: Record<string, string[]> }>(
        '/api/v1/tags/categories'
      );

      return response.categories;
    } catch (error) {
      console.error('Failed to get tag categories:', error);
      throw new Error('Failed to get tag categories');
    }
  }

  /**
   * Get trending tags for discovery
   */
  async getTrendingTags(limit: number = 10): Promise<PopularTag[]> {
    try {
      const response = await this.apiClient.get<{ tags: PopularTag[] }>(
        `/api/v1/tags/trending?limit=${limit}`
      );

      return response.tags.filter(tag => tag.trendingUp);
    } catch (error) {
      console.error('Failed to get trending tags:', error);
      throw new Error('Failed to get trending tags');
    }
  }

  /**
   * Batch operations for multiple recipes
   */
  async batchUpdateTags(
    operations: Array<{
      recipeId: string;
      tags: string[];
      action: 'add' | 'remove' | 'replace';
    }>
  ): Promise<{ success: boolean; results: TagUpdateResponse[] }> {
    try {
      const results = await Promise.allSettled(
        operations.map(op =>
          this.updateRecipeTags(op.recipeId, op.tags, op.action)
        )
      );

      const successful = results
        .filter(result => result.status === 'fulfilled')
        .map(result => (result as PromiseFulfilledResult<TagUpdateResponse>).value);

      const failed = results.filter(result => result.status === 'rejected');

      if (failed.length > 0) {
        console.warn(`${failed.length} tag operations failed`);
      }

      return {
        success: failed.length === 0,
        results: successful,
      };
    } catch (error) {
      console.error('Failed to batch update tags:', error);
      throw new Error('Failed to batch update tags');
    }
  }

  /**
   * Get user's tag usage statistics
   */
  async getUserTagStats(): Promise<{
    totalTags: number;
    mostUsedTags: Array<{ tag: string; count: number }>;
    recentTags: string[];
  }> {
    try {
      const response = await this.apiClient.get<{
        total_tags: number;
        most_used: Array<{ tag: string; count: number }>;
        recent: string[];
      }>('/api/v1/user/tags/stats');

      return {
        totalTags: response.total_tags,
        mostUsedTags: response.most_used,
        recentTags: response.recent,
      };
    } catch (error) {
      console.error('Failed to get user tag stats:', error);
      throw new Error('Failed to get user tag stats');
    }
  }

  /**
   * Clean up unused or low-quality tags
   */
  async cleanupTags(recipeId: string): Promise<{ removedTags: string[] }> {
    try {
      const response = await this.apiClient.post<{ removed_tags: string[] }>(
        `/api/v1/recipes/${recipeId}/tags/cleanup`
      );

      return {
        removedTags: response.removed_tags,
      };
    } catch (error) {
      console.error('Failed to cleanup tags:', error);
      throw new Error('Failed to cleanup tags');
    }
  }

  /**
   * Export user's tags for backup or analysis
   */
  async exportUserTags(): Promise<{
    recipes: Array<{
      recipeId: string;
      title: string;
      tags: string[];
    }>;
  }> {
    try {
      const response = await this.apiClient.get<{
        recipes: Array<{
          recipe_id: string;
          title: string;
          tags: string[];
        }>;
      }>('/api/v1/user/tags/export');

      return {
        recipes: response.recipes.map(recipe => ({
          recipeId: recipe.recipe_id,
          title: recipe.title,
          tags: recipe.tags,
        })),
      };
    } catch (error) {
      console.error('Failed to export user tags:', error);
      throw new Error('Failed to export user tags');
    }
  }
}