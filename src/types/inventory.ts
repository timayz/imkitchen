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
  category?: CategoryType;
  expiringSoon?: boolean;
  search?: string;
}

export type ExpirationStatus =
  | 'expired'
  | 'expiring_soon'
  | 'expiring_later'
  | 'fresh';

// Custom Category Management
export interface CustomCategory {
  id: string;
  name: string;
  color: string;
  icon: string;
  householdId: string;
  createdBy: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface CustomCategoryCreate {
  name: string;
  color: string;
  icon: string;
}

export interface CustomCategoryUpdate {
  name?: string;
  color?: string;
  icon?: string;
}

// Category with custom support
export type CategoryType = InventoryCategory | string; // string for custom category IDs

// Category Statistics
export interface CategoryStats {
  category: CategoryType;
  itemCount: number;
  expiringThisWeek: number;
  expiringToday: number;
  totalValue?: number;
  lastUpdated: Date;
}

// Bulk Operations
export interface BulkUpdateRequest {
  itemIds: string[];
  updates: {
    category?: CategoryType;
    location?: StorageLocation;
  };
}

// Sorting and filtering
export type SortOption =
  | 'alphabetical'
  | 'expiration'
  | 'quantity'
  | 'recently_added';
export type SortDirection = 'asc' | 'desc';

export interface CategoryFilters extends InventoryFilters {
  sortBy?: SortOption;
  sortDirection?: SortDirection;
  showOnlyExpiring?: boolean;
}

// Drag and drop
export interface DragItem {
  id: string;
  type: 'inventory-item';
  item: InventoryItem;
}

export interface DropResult {
  category: CategoryType;
}
