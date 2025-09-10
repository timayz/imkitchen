import { RecipeFavorite } from '../store/favorites_store';
import { FavoritesExport } from '../components/favorites/FavoritesImportExport';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:8080/api/v1';

interface ApiResponse<T> {
  data: T;
  message?: string;
  metadata?: {
    total?: number;
    page?: number;
    limit?: number;
  };
}

class FavoritesService {
  private async getAuthToken(): Promise<string> {
    // Get token from auth store or AsyncStorage
    const AsyncStorage = require('@react-native-async-storage/async-storage').default;
    const token = await AsyncStorage.getItem('authToken') || '';
    return token;
  }

  private async apiRequest<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    const token = await this.getAuthToken();
    
    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
        ...options.headers,
      },
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.message || `HTTP error! status: ${response.status}`);
    }

    return response.json();
  }

  // Get user's favorite recipes
  async getUserFavorites(page = 1, limit = 50): Promise<RecipeFavorite[]> {
    const response = await this.apiRequest<RecipeFavorite[]>(
      `/users/favorites?page=${page}&limit=${limit}`
    );
    return response.data;
  }

  // Add recipe to favorites
  async addFavorite(recipeId: string): Promise<RecipeFavorite> {
    const response = await this.apiRequest<RecipeFavorite>(
      `/users/favorites/${recipeId}`,
      {
        method: 'POST',
      }
    );
    return response.data;
  }

  // Remove recipe from favorites
  async removeFavorite(recipeId: string): Promise<void> {
    await this.apiRequest<void>(`/users/favorites/${recipeId}`, {
      method: 'DELETE',
    });
  }

  // Check if recipe is favorited
  async isFavorite(recipeId: string): Promise<boolean> {
    try {
      await this.apiRequest<RecipeFavorite>(`/users/favorites/${recipeId}`);
      return true;
    } catch (error) {
      // If 404, recipe is not favorited
      if (error instanceof Error && error.message.includes('404')) {
        return false;
      }
      throw error;
    }
  }

  // Get favorites count
  async getFavoritesCount(): Promise<number> {
    const response = await this.apiRequest<{ count: number }>('/users/favorites/count');
    return response.data.count;
  }

  // Export favorites data
  async exportFavorites(data: FavoritesExport): Promise<void> {
    // In a real app, this would create a file and share it
    // For now, we'll simulate the export by logging the data
    console.log('Exporting favorites:', data);
    
    // Could use react-native-fs or similar to write to file system
    // await RNFS.writeFile(path, JSON.stringify(data, null, 2));
  }

  // Import favorites data
  async importFavorites(data: FavoritesExport): Promise<{ imported: number; skipped: number }> {
    let imported = 0;
    let skipped = 0;

    for (const favorite of data.favorites) {
      try {
        // Check if already favorited
        const isAlreadyFavorite = await this.isFavorite(favorite.recipeId);
        
        if (isAlreadyFavorite) {
          skipped++;
          continue;
        }

        // Add to favorites
        await this.addFavorite(favorite.recipeId);
        imported++;
      } catch (error) {
        console.error(`Failed to import favorite ${favorite.recipeId}:`, error);
        skipped++;
      }
    }

    return { imported, skipped };
  }

  // Bulk update favorites (for advanced operations)
  async bulkUpdateFavorites(operations: {
    add?: string[];
    remove?: string[];
  }): Promise<{ added: number; removed: number }> {
    const response = await this.apiRequest<{ added: number; removed: number }>(
      '/users/favorites/bulk',
      {
        method: 'PUT',
        body: JSON.stringify(operations),
      }
    );
    return response.data;
  }

  // Get favorites with recipes (includes full recipe data)
  async getFavoritesWithRecipes(page = 1, limit = 50): Promise<RecipeFavorite[]> {
    const response = await this.apiRequest<RecipeFavorite[]>(
      `/users/favorites/recipes?page=${page}&limit=${limit}&include=recipe`
    );
    return response.data;
  }

  // Search within favorites
  async searchFavorites(
    query: string, 
    filters?: {
      complexity?: string;
      maxPrepTime?: number;
      tags?: string[];
    }
  ): Promise<RecipeFavorite[]> {
    const params = new URLSearchParams({
      q: query,
      ...(filters?.complexity && { complexity: filters.complexity }),
      ...(filters?.maxPrepTime && { maxPrepTime: filters.maxPrepTime.toString() }),
      ...(filters?.tags && { tags: filters.tags.join(',') }),
    });

    const response = await this.apiRequest<RecipeFavorite[]>(
      `/users/favorites/search?${params.toString()}`
    );
    return response.data;
  }

  // Get favorites analytics (for insights)
  async getFavoritesAnalytics(): Promise<{
    totalFavorites: number;
    favoritesByComplexity: Record<string, number>;
    favoritesByTag: Record<string, number>;
    averagePrepTime: number;
    favoriteFrequency: Record<string, number>;
  }> {
    const response = await this.apiRequest<any>('/users/favorites/analytics');
    return response.data;
  }
}

export default new FavoritesService();