import type {
  RecipeRating,
  RecipeRatingSubmission,
  RecipeRatingResponse,
  PaginatedRatingResponse,
  UserRatingHistoryResponse,
  FlagRatingRequest,
} from '@imkitchen/shared-types';

export interface RatingServiceConfig {
  baseUrl: string;
  getAuthToken: () => string | null;
}

export class RatingService {
  private baseUrl: string;
  private getAuthToken: () => string | null;

  constructor(config: RatingServiceConfig) {
    this.baseUrl = config.baseUrl;
    this.getAuthToken = config.getAuthToken;
  }

  private async makeRequest<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const token = this.getAuthToken();
    
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...(token && { Authorization: `Bearer ${token}` }),
        ...options.headers,
      },
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: 'Request failed' }));
      throw new Error(error.message || `Request failed with status ${response.status}`);
    }

    return response.json();
  }

  async submitRating(
    recipeId: string,
    rating: RecipeRatingSubmission
  ): Promise<RecipeRatingResponse> {
    return this.makeRequest<RecipeRatingResponse>(
      `/api/v1/recipes/${recipeId}/rating`,
      {
        method: 'POST',
        body: JSON.stringify(rating),
      }
    );
  }

  async updateRating(
    recipeId: string,
    rating: RecipeRatingSubmission
  ): Promise<RecipeRatingResponse> {
    return this.makeRequest<RecipeRatingResponse>(
      `/api/v1/recipes/${recipeId}/rating`,
      {
        method: 'PUT',
        body: JSON.stringify(rating),
      }
    );
  }

  async deleteRating(recipeId: string): Promise<void> {
    await this.makeRequest(
      `/api/v1/recipes/${recipeId}/rating`,
      {
        method: 'DELETE',
      }
    );
  }

  async getRecipeRatings(
    recipeId: string,
    page: number = 1,
    limit: number = 20
  ): Promise<PaginatedRatingResponse> {
    const params = new URLSearchParams({
      page: page.toString(),
      limit: limit.toString(),
    });

    return this.makeRequest<PaginatedRatingResponse>(
      `/api/v1/recipes/${recipeId}/ratings?${params}`
    );
  }

  async getUserRating(recipeId: string): Promise<RecipeRating | null> {
    try {
      const response = await this.makeRequest<{ rating: RecipeRating }>(
        `/api/v1/recipes/${recipeId}/rating/me`
      );
      return response.rating;
    } catch (error) {
      // Return null if no rating found (404)
      if (error instanceof Error && error.message.includes('404')) {
        return null;
      }
      throw error;
    }
  }

  async getUserRatingHistory(
    page: number = 1,
    limit: number = 20
  ): Promise<UserRatingHistoryResponse> {
    const params = new URLSearchParams({
      page: page.toString(),
      limit: limit.toString(),
    });

    return this.makeRequest<UserRatingHistoryResponse>(
      `/api/v1/users/me/ratings?${params}`
    );
  }

  async flagRating(
    ratingId: string,
    request: FlagRatingRequest
  ): Promise<void> {
    await this.makeRequest(
      `/api/v1/ratings/${ratingId}/flag`,
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );
  }

  async unflagRating(ratingId: string): Promise<void> {
    await this.makeRequest(
      `/api/v1/ratings/${ratingId}/flag`,
      {
        method: 'DELETE',
      }
    );
  }
}