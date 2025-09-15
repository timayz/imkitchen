import { apiClient } from '@/lib/api-client';
import type {
  InventoryItem,
  InventoryItemCreate,
  InventoryItemUpdate,
  CategoryFilters,
  BulkUpdateRequest,
  CustomCategory,
  CustomCategoryCreate,
  CustomCategoryUpdate,
  CategoryStats,
} from '@/types/inventory';

export class InventoryService {
  static async getItems(filters?: CategoryFilters): Promise<InventoryItem[]> {
    const params: Record<string, string> = {};

    if (filters?.location) params.location = filters.location;
    if (filters?.category) params.category = filters.category;
    if (filters?.expiringSoon) params.expiringSoon = 'true';
    if (filters?.search) params.search = filters.search;
    if (filters?.sortBy) params.sortBy = filters.sortBy;
    if (filters?.sortDirection) params.sortDirection = filters.sortDirection;
    if (filters?.showOnlyExpiring) params.showOnlyExpiring = 'true';

    return apiClient.get<InventoryItem[]>('/inventory', params);
  }

  static async createItem(item: InventoryItemCreate): Promise<InventoryItem> {
    const payload = {
      ...item,
      expirationDate: item.expirationDate?.toISOString(),
      purchaseDate: item.purchaseDate?.toISOString(),
    };

    return apiClient.post<InventoryItem>('/inventory', payload);
  }

  static async updateItem(
    id: string,
    updates: InventoryItemUpdate
  ): Promise<InventoryItem> {
    const payload = {
      ...updates,
      expirationDate: updates.expirationDate?.toISOString(),
    };

    return apiClient.put<InventoryItem>(`/inventory/${id}`, payload);
  }

  static async deleteItem(id: string): Promise<void> {
    return apiClient.delete<void>(`/inventory/${id}`);
  }

  static async getExpiringItems(): Promise<InventoryItem[]> {
    return apiClient.get<InventoryItem[]>('/inventory', {
      expiringSoon: 'true',
    });
  }

  // Category Management
  static async getCategories(): Promise<CustomCategory[]> {
    return apiClient.get<CustomCategory[]>('/inventory/categories');
  }

  static async createCategory(
    category: CustomCategoryCreate
  ): Promise<CustomCategory> {
    return apiClient.post<CustomCategory>('/inventory/categories', category);
  }

  static async updateCategory(
    id: string,
    updates: CustomCategoryUpdate
  ): Promise<CustomCategory> {
    return apiClient.put<CustomCategory>(
      `/inventory/categories/${id}`,
      updates
    );
  }

  static async deleteCategory(id: string): Promise<void> {
    return apiClient.delete<void>(`/inventory/categories/${id}`);
  }

  static async getCategoryStats(): Promise<CategoryStats[]> {
    return apiClient.get<CategoryStats[]>('/inventory/categories/stats');
  }

  // Bulk Operations
  static async bulkUpdateItems(
    request: BulkUpdateRequest
  ): Promise<InventoryItem[]> {
    return apiClient.put<InventoryItem[]>('/inventory/bulk-update', request);
  }
}
