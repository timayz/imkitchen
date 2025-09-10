export interface ShoppingList {
  id: string;
  userId: string;
  mealPlanId?: string;
  name: string;
  status: 'active' | 'completed' | 'archived';
  categories: { [category: string]: ShoppingItem[] };
  totalItems: number;
  completedItems: number;
  generatedAt: Date;
  estimatedCost?: number;
}

export interface ShoppingItem {
  id: string;
  shoppingListId: string;
  ingredientName: string;
  amount: number;
  unit: string;
  category: 'produce' | 'dairy' | 'pantry' | 'protein' | 'other';
  isCompleted: boolean;
  notes?: string;
  recipeSources?: string[];
  estimatedCost?: number;
  completedAt?: Date;
  createdAt: Date;
  updatedAt: Date;
}

export interface ShoppingListGenerateRequest {
  mealPlanId: string;
  mergeExisting?: boolean;
}

export interface ShoppingItemUpdateRequest {
  isCompleted: boolean;
  notes?: string;
}

export interface ShoppingListExportOptions {
  format: 'json' | 'csv' | 'txt';
  includeRecipeSources?: boolean;
}

export interface ShoppingListFilters {
  status?: 'active' | 'completed' | 'archived';
  sortBy?: 'created' | 'updated' | 'name';
}

// Shopping list state for Zustand store
export interface ShoppingState {
  shoppingLists: ShoppingList[];
  currentList: ShoppingList | null;
  isGenerating: boolean;
  isLoading: boolean;
  error: string | null;
  
  // Actions
  generateShoppingList: (mealPlanId: string, mergeExisting?: boolean) => Promise<void>;
  loadShoppingLists: (filters?: ShoppingListFilters) => Promise<void>;
  loadShoppingList: (listId: string) => Promise<void>;
  toggleItemCompleted: (listId: string, itemId: string, isCompleted: boolean, notes?: string) => Promise<void>;
  exportShoppingList: (listId: string, options: ShoppingListExportOptions) => Promise<Blob>;
  deleteShoppingList: (listId: string) => Promise<void>;
  clearError: () => void;
}

// Props for shopping components
export interface ShoppingListScreenProps {
  listId?: string;
}

export interface ShoppingItemCardProps {
  item: ShoppingItem;
  onToggleCompleted: (isCompleted: boolean, notes?: string) => Promise<void>;
  onShowRecipeSources: () => void;
  disabled?: boolean;
}

export interface GrocerySectionHeaderProps {
  category: string;
  itemCount: number;
  completedCount: number;
  isExpanded: boolean;
  onToggleExpanded: () => void;
}

export interface ShoppingProgressBarProps {
  totalItems: number;
  completedItems: number;
  showPercentage?: boolean;
}

export interface ShoppingListExportButtonProps {
  listId: string;
  disabled?: boolean;
}

export interface RecipeSourceModalProps {
  isVisible: boolean;
  onClose: () => void;
  ingredientName: string;
  recipeSources: string[];
}