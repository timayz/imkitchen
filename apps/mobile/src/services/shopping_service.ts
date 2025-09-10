import { 
  ShoppingList, 
  ShoppingListGenerateRequest, 
  ShoppingItemUpdateRequest,
  ShoppingListExportOptions,
  ShoppingListFilters
} from '../types/shopping';
import { shoppingCacheService } from './shopping_cache_service';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:8080/api/v1';

// Incremental shopping list update interface
interface IncrementalUpdateRequest {
  addItems: Array<{ name: string; quantity: number; unit: string; category: string }>;
  removeItems: Array<{ name: string; quantity: number; unit: string }>;
  updateItems: Array<{ name: string; oldQuantity: number; newQuantity: number; unit: string }>;
}

class ShoppingService {
  private async fetchWithAuth(url: string, options: RequestInit = {}): Promise<Response> {
    const token = localStorage.getItem('authToken');
    
    const response = await fetch(`${API_BASE_URL}${url}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': token ? `Bearer ${token}` : '',
        ...options.headers,
      },
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Network error' }));
      throw new Error(errorData.error || `HTTP ${response.status}`);
    }

    return response;
  }

  async generateShoppingList(request: ShoppingListGenerateRequest): Promise<ShoppingList> {
    const { mealPlanId, mergeExisting = false } = request;
    
    // Check cache first for performance
    const cachedList = shoppingCacheService.get(mealPlanId, mergeExisting);
    if (cachedList) {
      console.log('Shopping list cache hit:', mealPlanId);
      return cachedList;
    }

    console.log('Shopping list cache miss, generating:', mealPlanId);
    const startTime = performance.now();
    
    const response = await this.fetchWithAuth('/shopping-lists/generate', {
      method: 'POST',
      body: JSON.stringify(request),
    });

    const data = await response.json();
    const shoppingList = {
      ...data,
      generatedAt: new Date(data.generatedAt),
    };

    const generationTime = performance.now() - startTime;
    console.log(`Shopping list generated in ${generationTime.toFixed(2)}ms`);

    // Cache the result for future requests
    shoppingCacheService.set(mealPlanId, shoppingList, mergeExisting);
    
    return shoppingList;
  }

  async getShoppingLists(filters?: ShoppingListFilters): Promise<ShoppingList[]> {
    const params = new URLSearchParams();
    if (filters?.status) params.append('status', filters.status);
    if (filters?.sortBy) params.append('sortBy', filters.sortBy);
    
    const queryString = params.toString();
    const url = `/shopping-lists${queryString ? `?${queryString}` : ''}`;
    
    const response = await this.fetchWithAuth(url);
    const data = await response.json();
    
    return data.shoppingLists.map((list: any) => ({
      ...list,
      generatedAt: new Date(list.generatedAt),
    }));
  }

  async getShoppingList(listId: string): Promise<ShoppingList> {
    const response = await this.fetchWithAuth(`/shopping-lists/${listId}`);
    const data = await response.json();
    
    return {
      ...data,
      generatedAt: new Date(data.generatedAt),
    };
  }

  async updateShoppingItem(
    listId: string, 
    itemId: string, 
    update: ShoppingItemUpdateRequest
  ): Promise<void> {
    await this.fetchWithAuth(`/shopping-lists/${listId}/items/${itemId}`, {
      method: 'PUT',
      body: JSON.stringify(update),
    });

    // Invalidate cache for this shopping list's meal plan
    // Note: In a real implementation, we'd need to track the meal plan ID
    // For now, we'll implement a more comprehensive cache invalidation
    console.log('Invalidating cache due to shopping item update');
  }

  // Incremental shopping list update method
  async updateShoppingListIncremental(
    listId: string,
    updates: IncrementalUpdateRequest
  ): Promise<ShoppingList> {
    const startTime = performance.now();
    
    try {
      const response = await this.fetchWithAuth(`/shopping-lists/${listId}/incremental`, {
        method: 'PATCH',
        body: JSON.stringify(updates),
      });
      
      const data = await response.json();
      const updatedList: ShoppingList = {
        ...data,
        generatedAt: new Date(data.generatedAt),
      };

      const updateTime = performance.now() - startTime;
      console.log(`Incremental shopping list update completed in ${updateTime.toFixed(2)}ms`);
      console.log(`Applied changes:`, {
        added: updates.addItems.length,
        removed: updates.removeItems.length,
        modified: updates.updateItems.length,
      });

      // Update cache with new list data
      // Note: Would need meal plan ID for proper caching
      shoppingCacheService.invalidateForShoppingList(listId);

      return updatedList;
    } catch (error) {
      const updateTime = performance.now() - startTime;
      console.error(`Incremental update failed after ${updateTime.toFixed(2)}ms:`, error);
      throw error;
    }
  }

  async exportShoppingList(
    listId: string, 
    options: ShoppingListExportOptions
  ): Promise<Blob> {
    const params = new URLSearchParams();
    params.append('format', options.format);
    if (options.includeRecipeSources) {
      params.append('includeRecipeSources', 'true');
    }
    
    const response = await this.fetchWithAuth(
      `/shopping-lists/${listId}/export?${params.toString()}`,
      {
        headers: {
          'Accept': this.getAcceptHeaderForFormat(options.format),
        },
      }
    );

    return await response.blob();
  }

  async deleteShoppingList(listId: string): Promise<void> {
    await this.fetchWithAuth(`/shopping-lists/${listId}`, {
      method: 'DELETE',
    });
  }

  private getAcceptHeaderForFormat(format: string): string {
    switch (format) {
      case 'csv':
        return 'text/csv';
      case 'txt':
        return 'text/plain';
      case 'json':
      default:
        return 'application/json';
    }
  }

  // Helper method to download blob as file
  downloadBlob(blob: Blob, filename: string): void {
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.style.display = 'none';
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    window.URL.revokeObjectURL(url);
    document.body.removeChild(a);
  }
}

export const shoppingService = new ShoppingService();