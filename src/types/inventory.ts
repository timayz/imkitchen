export type InventoryCategory =
  | 'proteins'
  | 'vegetables'
  | 'fruits'
  | 'grains'
  | 'dairy'
  | 'spices'
  | 'condiments'
  | 'beverages'
  | 'baking'
  | 'frozen';

export type StorageLocation = 'pantry' | 'refrigerator' | 'freezer';

export type MeasurementUnit =
  | 'grams'
  | 'kilograms'
  | 'ounces'
  | 'pounds'
  | 'cups'
  | 'tablespoons'
  | 'teaspoons'
  | 'pieces'
  | 'milliliters'
  | 'liters';

export interface InventoryItem {
  id: string;
  name: string;
  quantity: number;
  unit: MeasurementUnit;
  category: InventoryCategory;
  location: StorageLocation;
  expirationDate: Date | null;
  purchaseDate: Date | null;
  estimatedCost?: number | null;
  householdId: string;
  addedBy: string;
  addedByUser?: {
    name: string;
  };
  createdAt: Date;
  updatedAt: Date;
}

export interface InventoryItemCreate {
  name: string;
  quantity: number;
  unit: MeasurementUnit;
  category: InventoryCategory;
  location: StorageLocation;
  expirationDate?: Date | null;
  purchaseDate?: Date | null;
  estimatedCost?: number | null;
}

export interface InventoryItemUpdate {
  name?: string;
  quantity?: number;
  unit?: MeasurementUnit;
  category?: InventoryCategory;
  location?: StorageLocation;
  expirationDate?: Date | null;
  estimatedCost?: number | null;
}

export interface InventoryFilters {
  location?: StorageLocation;
  category?: InventoryCategory;
  expiringSoon?: boolean;
  search?: string;
}

export type ExpirationStatus =
  | 'expired'
  | 'expiring_soon'
  | 'expiring_later'
  | 'fresh';
