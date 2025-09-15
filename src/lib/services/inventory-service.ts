import { apiClient } from '@/lib/api-client';
import type {
  InventoryItem,
  InventoryItemCreate,
  InventoryItemUpdate,
  InventoryFilters,
} from '@/types/inventory';

export class InventoryService {
  static async getItems(filters?: InventoryFilters): Promise<InventoryItem[]> {
    const params: Record<string, string> = {};

    if (filters?.location) params.location = filters.location;
    if (filters?.category) params.category = filters.category;
    if (filters?.expiringSoon) params.expiringSoon = 'true';
    if (filters?.search) params.search = filters.search;

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
}
