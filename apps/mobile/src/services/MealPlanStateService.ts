import type { 
  MealPlanResponse, 
  DayOfWeek, 
  MealType,
  MealSlotWithRecipe 
} from '@imkitchen/shared-types';
import type { MealPlanSnapshot } from './ChangeHistoryTracker';
import { mealPlanService } from './meal_plan_service';

export interface StateRestorationResult {
  success: boolean;
  updatedMealPlan?: MealPlanResponse;
  error?: string;
  changesApplied?: Array<{
    day: DayOfWeek;
    mealType: MealType;
    action: 'add' | 'remove' | 'update' | 'lock' | 'unlock';
    details?: string;
  }>;
}

export class MealPlanStateService {
  /**
   * Apply a snapshot to restore meal plan state
   */
  async applySnapshot(
    currentMealPlan: MealPlanResponse,
    targetSnapshot: MealPlanSnapshot
  ): Promise<StateRestorationResult> {
    const changesApplied: StateRestorationResult['changesApplied'] = [];
    
    try {
      // Compare current state with target snapshot and apply necessary changes
      const currentSnapshot = this.createCurrentSnapshot(currentMealPlan);
      const changes = this.calculateRequiredChanges(currentSnapshot, targetSnapshot);
      
      // Apply changes in the correct order to avoid conflicts
      const sortedChanges = this.sortChangesByPriority(changes);
      
      for (const change of sortedChanges) {
        try {
          await this.applyChange(currentMealPlan.id, change);
          changesApplied.push({
            day: change.day,
            mealType: change.mealType,
            action: change.action,
            details: change.details,
          });
        } catch (error) {
          console.error(`Failed to apply change for ${change.day} ${change.mealType}:`, error);
          // Continue with other changes even if one fails
        }
      }

      // Fetch updated meal plan
      const updatedMealPlan = await mealPlanService.getMealPlan(currentMealPlan.id);
      
      return {
        success: true,
        updatedMealPlan,
        changesApplied,
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error occurred',
        changesApplied,
      };
    }
  }

  /**
   * Create a snapshot from current meal plan
   */
  private createCurrentSnapshot(mealPlan: MealPlanResponse): MealPlanSnapshot {
    const snapshot: MealPlanSnapshot = {
      mealPlanId: mealPlan.id,
      version: Date.now(),
      timestamp: new Date(),
      meals: {},
      totalEstimatedTime: mealPlan.totalEstimatedTime,
      completionPercentage: mealPlan.completionPercentage,
    };

    // Convert meal plan data to snapshot format
    Object.entries(mealPlan.populatedMeals).forEach(([day, dayMeals]) => {
      if (dayMeals) {
        snapshot.meals[day as DayOfWeek] = {};
        dayMeals.forEach(meal => {
          if (meal && snapshot.meals[day as DayOfWeek]) {
            snapshot.meals[day as DayOfWeek]![meal.mealType] = {
              recipeId: meal.recipe?.id,
              recipeName: meal.recipe?.title,
              servings: meal.servings,
              isLocked: meal.isLocked,
              isCompleted: meal.isCompleted,
              notes: meal.notes,
            };
          }
        });
      }
    });

    return snapshot;
  }

  /**
   * Calculate what changes are needed to transform current state to target state
   */
  private calculateRequiredChanges(
    current: MealPlanSnapshot, 
    target: MealPlanSnapshot
  ): StateChange[] {
    const changes: StateChange[] = [];
    
    // Get all unique day/mealType combinations from both snapshots
    const allSlots = new Set<string>();
    
    // Add slots from current snapshot
    Object.keys(current.meals).forEach(day => {
      Object.keys(current.meals[day as DayOfWeek] || {}).forEach(mealType => {
        allSlots.add(`${day}-${mealType}`);
      });
    });
    
    // Add slots from target snapshot
    Object.keys(target.meals).forEach(day => {
      Object.keys(target.meals[day as DayOfWeek] || {}).forEach(mealType => {
        allSlots.add(`${day}-${mealType}`);
      });
    });

    // Compare each slot
    allSlots.forEach(slotKey => {
      const [day, mealType] = slotKey.split('-') as [DayOfWeek, MealType];
      
      const currentSlot = current.meals[day]?.[mealType];
      const targetSlot = target.meals[day]?.[mealType];
      
      if (!currentSlot && !targetSlot) {
        return; // Both empty, no change needed
      }
      
      if (!currentSlot && targetSlot) {
        // Need to add meal
        changes.push({
          day,
          mealType,
          action: 'add',
          newRecipeId: targetSlot.recipeId,
          newServings: targetSlot.servings,
          newLocked: targetSlot.isLocked,
          details: `Add ${targetSlot.recipeName || 'meal'}`,
        });
      } else if (currentSlot && !targetSlot) {
        // Need to remove meal
        changes.push({
          day,
          mealType,
          action: 'remove',
          details: `Remove ${currentSlot.recipeName || 'meal'}`,
        });
      } else if (currentSlot && targetSlot) {
        // Compare meal properties
        if (currentSlot.recipeId !== targetSlot.recipeId) {
          changes.push({
            day,
            mealType,
            action: 'update',
            newRecipeId: targetSlot.recipeId,
            newServings: targetSlot.servings,
            details: `Change from ${currentSlot.recipeName || 'unknown'} to ${targetSlot.recipeName || 'unknown'}`,
          });
        }
        
        if (currentSlot.isLocked !== targetSlot.isLocked) {
          changes.push({
            day,
            mealType,
            action: targetSlot.isLocked ? 'lock' : 'unlock',
            details: `${targetSlot.isLocked ? 'Lock' : 'Unlock'} ${targetSlot.recipeName || 'meal'}`,
          });
        }
      }
    });

    return changes;
  }

  /**
   * Sort changes by priority to avoid conflicts
   */
  private sortChangesByPriority(changes: StateChange[]): StateChange[] {
    // Priority order: remove > unlock > add > update > lock
    const priorityOrder = { remove: 1, unlock: 2, add: 3, update: 4, lock: 5 };
    
    return changes.sort((a, b) => 
      (priorityOrder[a.action] || 99) - (priorityOrder[b.action] || 99)
    );
  }

  /**
   * Apply a single state change
   */
  private async applyChange(mealPlanId: string, change: StateChange): Promise<void> {
    switch (change.action) {
      case 'add':
      case 'update':
        if (change.newRecipeId) {
          await mealPlanService.updateMealSlot(mealPlanId, {
            day: change.day,
            mealType: change.mealType,
            recipeId: change.newRecipeId,
            servings: change.newServings || 1,
          });
        }
        break;
        
      case 'remove':
        await mealPlanService.updateMealSlot(mealPlanId, {
          day: change.day,
          mealType: change.mealType,
          recipeId: null, // Remove recipe
        });
        break;
        
      case 'lock':
        await mealPlanService.toggleMealLock(
          mealPlanId,
          change.day,
          change.mealType,
          true
        );
        break;
        
      case 'unlock':
        await mealPlanService.toggleMealLock(
          mealPlanId,
          change.day,
          change.mealType,
          false
        );
        break;
    }
  }

  /**
   * Validate if a snapshot can be applied
   */
  validateSnapshot(snapshot: MealPlanSnapshot): { valid: boolean; errors: string[] } {
    const errors: string[] = [];
    
    if (!snapshot.mealPlanId) {
      errors.push('Snapshot missing meal plan ID');
    }
    
    if (!snapshot.meals) {
      errors.push('Snapshot missing meals data');
    }
    
    // Validate meal structure
    Object.entries(snapshot.meals).forEach(([day, dayMeals]) => {
      if (!dayMeals) return;
      
      Object.entries(dayMeals).forEach(([mealType, meal]) => {
        if (!meal) return;
        
        if (meal.recipeId && !meal.recipeName) {
          errors.push(`Recipe ID without name for ${day} ${mealType}`);
        }
        
        if (meal.servings && meal.servings <= 0) {
          errors.push(`Invalid servings count for ${day} ${mealType}`);
        }
      });
    });
    
    return {
      valid: errors.length === 0,
      errors,
    };
  }
}

interface StateChange {
  day: DayOfWeek;
  mealType: MealType;
  action: 'add' | 'remove' | 'update' | 'lock' | 'unlock';
  newRecipeId?: string | null;
  newServings?: number;
  newLocked?: boolean;
  details: string;
}

export const mealPlanStateService = new MealPlanStateService();