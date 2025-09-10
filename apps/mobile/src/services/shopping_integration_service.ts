import type { MealPlanResponse, ShoppingList, DayOfWeek, MealType } from '../types/shopping';
import { shoppingService } from './shopping_service';
import { useShoppingStore } from '../store/shopping_store';
import type { MealPlanSnapshot, MealPlanChange } from './ChangeHistoryTracker';

interface ShoppingListIntegrationOptions {
  autoGenerate?: boolean;
  mergeExisting?: boolean;
  notifyUser?: boolean;
  useIncrementalUpdate?: boolean;
  skipVersionCheck?: boolean;
}

interface ShoppingListVersion {
  id: string;
  shoppingListId: string;
  mealPlanId: string;
  version: number;
  mealPlanVersion: number;
  createdAt: Date;
  changesSinceLastUpdate: MealPlanChange[];
}

interface ShoppingListDiff {
  added: Array<{ name: string; amount: number; unit: string }>;
  removed: Array<{ name: string; amount: number; unit: string }>;
  modified: Array<{ 
    name: string; 
    oldAmount: number; 
    newAmount: number; 
    unit: string; 
  }>;
}

class ShoppingIntegrationService {
  private notificationQueue: Array<{ 
    message: string; 
    type: 'success' | 'info' | 'warning'; 
    timestamp: number;
  }> = [];

  private versionStore: Map<string, ShoppingListVersion> = new Map();
  private updateLocks: Map<string, Promise<ShoppingList | null>> = new Map();

  // Main integration function called after meal plan changes
  async handleMealPlanChange(
    mealPlan: MealPlanResponse,
    previousMealPlan: MealPlanResponse | null,
    options: ShoppingListIntegrationOptions = {},
    changes?: MealPlanChange[]
  ): Promise<ShoppingList | null> {
    const {
      autoGenerate = true,
      mergeExisting = true,
      notifyUser = true,
      useIncrementalUpdate = true,
      skipVersionCheck = false,
    } = options;

    // Prevent concurrent updates to the same meal plan
    const lockKey = `update-${mealPlan.id}`;
    if (this.updateLocks.has(lockKey)) {
      console.log('Shopping list update already in progress for meal plan:', mealPlan.id);
      return this.updateLocks.get(lockKey)!;
    }

    if (!autoGenerate) {
      return null;
    }

    try {
      const updatePromise = this.performShoppingListUpdate(
        mealPlan,
        previousMealPlan,
        {
          autoGenerate,
          mergeExisting,
          notifyUser,
          useIncrementalUpdate,
          skipVersionCheck,
        },
        changes
      );
      
      this.updateLocks.set(lockKey, updatePromise);
      
      const result = await updatePromise;
      this.updateLocks.delete(lockKey);
      
      return result;
    } catch (error) {
      console.error('Shopping list integration error:', error);
      
      if (notifyUser) {
        this.queueNotification(
          'Failed to update shopping list. You can generate it manually.',
          'warning'
        );
      }
      
      return null;
    }
  }

  // Core shopping list update logic with incremental support
  private async performShoppingListUpdate(
    mealPlan: MealPlanResponse,
    previousMealPlan: MealPlanResponse | null,
    options: Required<ShoppingListIntegrationOptions>,
    changes?: MealPlanChange[]
  ): Promise<ShoppingList | null> {
    const { useIncrementalUpdate, mergeExisting, notifyUser } = options;

    // Check if there's an existing shopping list for this meal plan
    const existingList = await this.findExistingShoppingList(mealPlan.id);
    
    if (existingList && previousMealPlan) {
      // Meal plan was updated - use incremental update if possible
      if (useIncrementalUpdate && changes && changes.length > 0) {
        return this.performIncrementalUpdate(
          existingList,
          mealPlan,
          previousMealPlan,
          changes,
          options
        );
      } else {
        // Fall back to full regeneration
        return this.performFullRegeneration(
          existingList,
          mealPlan,
          previousMealPlan,
          options
        );
      }
    } else {
      // New meal plan - generate fresh shopping list
      const newList = await this.generateShoppingList(mealPlan.id, mergeExisting);
      
      // Create version tracking entry
      this.createVersionEntry(newList, mealPlan, []);
      
      if (notifyUser) {
        this.queueNotification(
          `Shopping list generated with ${newList.totalItems} items`,
          'success'
        );
      }
      
      return newList;
    }
  }

  // Perform incremental update based on specific changes
  private async performIncrementalUpdate(
    existingList: ShoppingList,
    newMealPlan: MealPlanResponse,
    oldMealPlan: MealPlanResponse,
    changes: MealPlanChange[],
    options: Required<ShoppingListIntegrationOptions>
  ): Promise<ShoppingList | null> {
    try {
      console.log('Performing incremental shopping list update for changes:', changes.length);
      
      // Calculate specific ingredient changes based on meal plan changes
      const ingredientDiff = await this.calculateIncrementalChanges(changes, oldMealPlan, newMealPlan);
      
      if (!this.hasIncrementalChanges(ingredientDiff)) {
        console.log('No meaningful ingredient changes detected');
        return existingList;
      }

      // Apply incremental changes to shopping list
      const updatedList = await this.applyIncrementalChanges(
        existingList,
        ingredientDiff,
        options.mergeExisting
      );

      // Update version tracking
      this.updateVersionEntry(existingList.id, newMealPlan, changes);

      if (options.notifyUser) {
        const totalChanges = ingredientDiff.added.length + 
                           ingredientDiff.removed.length + 
                           ingredientDiff.modified.length;
        
        this.queueNotification(
          `Shopping list updated incrementally (${totalChanges} changes)`,
          'info'
        );
      }

      return updatedList;
    } catch (error) {
      console.error('Incremental update failed, falling back to full regeneration:', error);
      
      // Fall back to full regeneration if incremental fails
      return this.performFullRegeneration(existingList, newMealPlan, oldMealPlan, options);
    }
  }

  // Perform full shopping list regeneration
  private async performFullRegeneration(
    existingList: ShoppingList | null,
    newMealPlan: MealPlanResponse,
    oldMealPlan: MealPlanResponse | null,
    options: Required<ShoppingListIntegrationOptions>
  ): Promise<ShoppingList | null> {
    const diff = oldMealPlan ? await this.calculateShoppingListDiff(oldMealPlan, newMealPlan) : null;
    
    if (!diff || this.hasMeaningfulChanges(diff)) {
      const updatedList = await this.regenerateShoppingList(
        newMealPlan.id,
        options.mergeExisting
      );
      
      if (options.notifyUser && diff) {
        this.queueNotification(
          `Shopping list updated with ${diff.added.length} new items`,
          'info'
        );
      }
      
      return updatedList;
    }
    
    return existingList;
  }

  // Generate a new shopping list for a meal plan
  async generateShoppingList(
    mealPlanId: string,
    mergeExisting = false
  ): Promise<ShoppingList> {
    return await shoppingService.generateShoppingList({
      mealPlanId,
      mergeExisting,
    });
  }

  // Regenerate shopping list (used for updates)
  async regenerateShoppingList(
    mealPlanId: string,
    mergeExisting = true
  ): Promise<ShoppingList> {
    // Delete existing list first if not merging
    if (!mergeExisting) {
      const existingList = await this.findExistingShoppingList(mealPlanId);
      if (existingList) {
        await shoppingService.deleteShoppingList(existingList.id);
      }
    }

    return await this.generateShoppingList(mealPlanId, mergeExisting);
  }

  // Find existing shopping list for a meal plan
  async findExistingShoppingList(mealPlanId: string): Promise<ShoppingList | null> {
    try {
      const response = await shoppingService.getShoppingLists({ 
        status: 'active',
        sortBy: 'created',
      });
      
      return response.find(list => list.mealPlanId === mealPlanId) || null;
    } catch (error) {
      console.error('Failed to find existing shopping list:', error);
      return null;
    }
  }

  // Calculate differences between meal plans for shopping list updates
  async calculateShoppingListDiff(
    oldMealPlan: MealPlanResponse,
    newMealPlan: MealPlanResponse
  ): Promise<ShoppingListDiff> {
    const oldIngredients = this.extractIngredients(oldMealPlan);
    const newIngredients = this.extractIngredients(newMealPlan);
    
    const diff: ShoppingListDiff = {
      added: [],
      removed: [],
      modified: [],
    };

    // Find added ingredients
    newIngredients.forEach(newIng => {
      const oldIng = oldIngredients.find(old => 
        old.name === newIng.name && old.unit === newIng.unit
      );
      
      if (!oldIng) {
        diff.added.push(newIng);
      } else if (oldIng.amount !== newIng.amount) {
        diff.modified.push({
          name: newIng.name,
          oldAmount: oldIng.amount,
          newAmount: newIng.amount,
          unit: newIng.unit,
        });
      }
    });

    // Find removed ingredients
    oldIngredients.forEach(oldIng => {
      const newIng = newIngredients.find(newItem => 
        newItem.name === oldIng.name && newItem.unit === oldIng.unit
      );
      
      if (!newIng) {
        diff.removed.push(oldIng);
      }
    });

    return diff;
  }

  // Extract ingredients from meal plan for comparison
  private extractIngredients(mealPlan: MealPlanResponse): Array<{ 
    name: string; 
    amount: number; 
    unit: string; 
  }> {
    const ingredients: Array<{ name: string; amount: number; unit: string }> = [];
    
    // This would typically iterate through meal plan entries and extract ingredients
    // For now, returning empty array as mock implementation
    // In real implementation, this would aggregate all recipe ingredients
    
    return ingredients;
  }

  // Check if diff contains meaningful changes
  private hasMeaningfulChanges(diff: ShoppingListDiff): boolean {
    return diff.added.length > 0 || 
           diff.removed.length > 0 || 
           diff.modified.length > 0;
  }

  // Notification system for shopping list updates
  queueNotification(message: string, type: 'success' | 'info' | 'warning'): void {
    this.notificationQueue.push({
      message,
      type,
      timestamp: Date.now(),
    });

    // Auto-clear old notifications
    setTimeout(() => {
      this.notificationQueue = this.notificationQueue.filter(
        notification => Date.now() - notification.timestamp < 10000 // 10 seconds
      );
    }, 10000);
  }

  // Get pending notifications
  getNotifications(): Array<{ 
    message: string; 
    type: 'success' | 'info' | 'warning'; 
    timestamp: number;
  }> {
    return [...this.notificationQueue];
  }

  // Clear notifications
  clearNotifications(): void {
    this.notificationQueue = [];
  }

  // Calculate incremental changes based on specific meal plan modifications
  private async calculateIncrementalChanges(
    changes: MealPlanChange[],
    oldMealPlan: MealPlanResponse,
    newMealPlan: MealPlanResponse
  ): Promise<ShoppingListDiff> {
    const incrementalDiff: ShoppingListDiff = {
      added: [],
      removed: [],
      modified: [],
    };

    for (const change of changes) {
      switch (change.type) {
        case 'add':
        case 'substitution':
        case 'swap':
          const addedIngredients = await this.getIngredientsFromChange(change, 'after');
          incrementalDiff.added.push(...addedIngredients);
          
          if (change.type === 'substitution' || change.type === 'swap') {
            const removedIngredients = await this.getIngredientsFromChange(change, 'before');
            incrementalDiff.removed.push(...removedIngredients);
          }
          break;

        case 'remove':
          const removedIngs = await this.getIngredientsFromChange(change, 'before');
          incrementalDiff.removed.push(...removedIngs);
          break;

        case 'reorder':
          // Reordering doesn't change ingredients, only their organization
          break;

        case 'batch':
          // Handle batch operations by processing their individual changes
          // This would require access to the batch's component operations
          break;
      }
    }

    // Consolidate and deduplicate changes
    return this.consolidateIncrementalChanges(incrementalDiff);
  }

  // Extract ingredients from a specific change
  private async getIngredientsFromChange(
    change: MealPlanChange,
    state: 'before' | 'after'
  ): Promise<Array<{ name: string; amount: number; unit: string }>> {
    const snapshot = state === 'before' ? change.beforeState : change.afterState;
    const ingredients: Array<{ name: string; amount: number; unit: string }> = [];

    // Extract ingredients from affected slots
    if (change.metadata?.affectedSlots) {
      for (const slot of change.metadata.affectedSlots) {
        const mealSlot = snapshot.meals[slot.day]?.[slot.mealType];
        if (mealSlot?.recipeId) {
          // In a real implementation, this would fetch recipe ingredients
          // For now, return mock ingredients based on recipe
          const recipeIngredients = await this.getRecipeIngredients(mealSlot.recipeId, mealSlot.servings || 1);
          ingredients.push(...recipeIngredients);
        }
      }
    }

    return ingredients;
  }

  // Mock method to get recipe ingredients (would call recipe service in real implementation)
  private async getRecipeIngredients(
    recipeId: string,
    servings: number
  ): Promise<Array<{ name: string; amount: number; unit: string }>> {
    // This is a mock implementation - in reality would fetch from recipe service
    return [
      { name: `Ingredient for ${recipeId}`, amount: servings * 1, unit: 'cup' },
      { name: `Secondary ingredient for ${recipeId}`, amount: servings * 0.5, unit: 'tsp' },
    ];
  }

  // Consolidate and deduplicate incremental changes
  private consolidateIncrementalChanges(diff: ShoppingListDiff): ShoppingListDiff {
    const consolidated: ShoppingListDiff = {
      added: [],
      removed: [],
      modified: [],
    };

    // Create maps for deduplication
    const addedMap = new Map<string, { name: string; amount: number; unit: string }>();
    const removedMap = new Map<string, { name: string; amount: number; unit: string }>();

    // Process added ingredients
    diff.added.forEach(ingredient => {
      const key = `${ingredient.name}-${ingredient.unit}`;
      const existing = addedMap.get(key);
      if (existing) {
        existing.amount += ingredient.amount;
      } else {
        addedMap.set(key, { ...ingredient });
      }
    });

    // Process removed ingredients
    diff.removed.forEach(ingredient => {
      const key = `${ingredient.name}-${ingredient.unit}`;
      const existing = removedMap.get(key);
      if (existing) {
        existing.amount += ingredient.amount;
      } else {
        removedMap.set(key, { ...ingredient });
      }
    });

    // Calculate net changes
    addedMap.forEach((ingredient, key) => {
      const removed = removedMap.get(key);
      if (removed) {
        const netAmount = ingredient.amount - removed.amount;
        if (netAmount > 0) {
          consolidated.modified.push({
            name: ingredient.name,
            oldAmount: removed.amount,
            newAmount: ingredient.amount,
            unit: ingredient.unit,
          });
        } else if (netAmount < 0) {
          consolidated.removed.push({
            name: ingredient.name,
            amount: Math.abs(netAmount),
            unit: ingredient.unit,
          });
        }
        removedMap.delete(key);
      } else {
        consolidated.added.push(ingredient);
      }
    });

    // Add remaining removed ingredients
    removedMap.forEach(ingredient => {
      consolidated.removed.push(ingredient);
    });

    return consolidated;
  }

  // Check if incremental changes are meaningful
  private hasIncrementalChanges(diff: ShoppingListDiff): boolean {
    return diff.added.length > 0 || diff.removed.length > 0 || diff.modified.length > 0;
  }

  // Apply incremental changes to existing shopping list
  private async applyIncrementalChanges(
    existingList: ShoppingList,
    diff: ShoppingListDiff,
    mergeExisting: boolean
  ): Promise<ShoppingList> {
    try {
      // Call shopping service to apply incremental changes
      return await shoppingService.updateShoppingListIncremental(existingList.id, {
        addItems: diff.added.map(item => ({
          name: item.name,
          quantity: item.amount,
          unit: item.unit,
          category: 'Other', // Would be determined by ingredient category service
        })),
        removeItems: diff.removed.map(item => ({
          name: item.name,
          quantity: item.amount,
          unit: item.unit,
        })),
        updateItems: diff.modified.map(item => ({
          name: item.name,
          oldQuantity: item.oldAmount,
          newQuantity: item.newAmount,
          unit: item.unit,
        })),
      });
    } catch (error) {
      console.error('Failed to apply incremental changes:', error);
      throw error;
    }
  }

  // Version tracking methods
  private createVersionEntry(
    shoppingList: ShoppingList,
    mealPlan: MealPlanResponse,
    initialChanges: MealPlanChange[]
  ): void {
    const versionEntry: ShoppingListVersion = {
      id: `version-${shoppingList.id}-${Date.now()}`,
      shoppingListId: shoppingList.id,
      mealPlanId: mealPlan.id,
      version: 1,
      mealPlanVersion: mealPlan.version || 1,
      createdAt: new Date(),
      changesSinceLastUpdate: initialChanges,
    };

    this.versionStore.set(shoppingList.id, versionEntry);
  }

  private updateVersionEntry(
    shoppingListId: string,
    mealPlan: MealPlanResponse,
    newChanges: MealPlanChange[]
  ): void {
    const existing = this.versionStore.get(shoppingListId);
    if (existing) {
      existing.version += 1;
      existing.mealPlanVersion = mealPlan.version || existing.mealPlanVersion + 1;
      existing.changesSinceLastUpdate = newChanges;
      existing.createdAt = new Date();
    }
  }

  // Get version info for shopping list
  getVersionInfo(shoppingListId: string): ShoppingListVersion | null {
    return this.versionStore.get(shoppingListId) || null;
  }

  // Integration with multiple meal plans for smart aggregation
  async aggregateShoppingListsFromMultiplePlans(
    mealPlanIds: string[]
  ): Promise<ShoppingList | null> {
    if (mealPlanIds.length === 0) {
      return null;
    }

    try {
      // For now, generate shopping list from the first meal plan
      // In a real implementation, this would aggregate ingredients from all plans
      const primaryMealPlanId = mealPlanIds[0];
      
      return await this.generateShoppingList(primaryMealPlanId, true);
    } catch (error) {
      console.error('Failed to aggregate shopping lists:', error);
      return null;
    }
  }

  // Hook for meal plan store integration
  onMealPlanGenerated = async (mealPlan: MealPlanResponse): Promise<void> => {
    await this.handleMealPlanChange(mealPlan, null, {
      autoGenerate: true,
      mergeExisting: false,
      notifyUser: true,
    });
  };

  onMealPlanUpdated = async (
    newMealPlan: MealPlanResponse,
    oldMealPlan: MealPlanResponse
  ): Promise<void> => {
    await this.handleMealPlanChange(newMealPlan, oldMealPlan, {
      autoGenerate: true,
      mergeExisting: true,
      notifyUser: true,
    });
  };
}

export const shoppingIntegrationService = new ShoppingIntegrationService();