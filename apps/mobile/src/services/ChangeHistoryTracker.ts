import type { DayOfWeek, MealType, MealSlotWithRecipe } from '@imkitchen/shared-types';

export interface MealPlanChange {
  id: string;
  timestamp: Date;
  type: 'substitution' | 'swap' | 'reorder' | 'lock' | 'unlock' | 'add' | 'remove' | 'batch';
  description: string;
  beforeState: MealPlanSnapshot;
  afterState: MealPlanSnapshot;
  metadata?: {
    reason?: string;
    batchId?: string;
    parentChange?: string;
    userInitiated?: boolean;
    cost?: number;
    affectedSlots?: Array<{ day: DayOfWeek; mealType: MealType }>;
  };
}

export interface MealPlanSnapshot {
  mealPlanId: string;
  version: number;
  timestamp: Date;
  meals: {
    [key in DayOfWeek]?: {
      [key in MealType]?: {
        recipeId?: string;
        recipeName?: string;
        servings?: number;
        isLocked?: boolean;
        isCompleted?: boolean;
        notes?: string;
      };
    };
  };
  totalEstimatedTime?: number;
  completionPercentage?: number;
}

export interface BatchOperation {
  id: string;
  startTime: Date;
  operations: MealPlanChange[];
  isActive: boolean;
  description: string;
}

interface ChangeHistoryConfig {
  maxHistoryLength: number;
  enableBatching: boolean;
  batchTimeout: number; // milliseconds
  autoCleanup: boolean;
  persistToStorage: boolean;
}

export class ChangeHistoryTracker {
  private history: MealPlanChange[] = [];
  private currentIndex: number = -1;
  private currentBatch: BatchOperation | null = null;
  private config: ChangeHistoryConfig;
  private mealPlanId: string;
  private changeIdCounter: number = 0;

  constructor(mealPlanId: string, config?: Partial<ChangeHistoryConfig>) {
    this.mealPlanId = mealPlanId;
    this.config = {
      maxHistoryLength: 20,
      enableBatching: true,
      batchTimeout: 2000, // 2 seconds
      autoCleanup: true,
      persistToStorage: true,
      ...config,
    };

    this.loadPersistedHistory();
  }

  /**
   * Record a new meal plan change
   */
  recordChange(change: Omit<MealPlanChange, 'id' | 'timestamp'>): string {
    const changeId = this.generateChangeId();
    const newChange: MealPlanChange = {
      ...change,
      id: changeId,
      timestamp: new Date(),
    };

    // If we're in the middle of history (after undo), remove future changes
    if (this.currentIndex < this.history.length - 1) {
      this.history = this.history.slice(0, this.currentIndex + 1);
    }

    // Handle batching
    if (this.config.enableBatching && this.shouldBatchChange(newChange)) {
      this.addToBatch(newChange);
    } else {
      this.addChange(newChange);
    }

    // Cleanup old history if needed
    this.enforceHistoryLimit();

    // Persist to storage
    if (this.config.persistToStorage) {
      this.persistHistory();
    }

    return changeId;
  }

  /**
   * Start a batch operation for grouping related changes
   */
  startBatch(description: string): string {
    if (this.currentBatch) {
      this.endBatch(); // End existing batch
    }

    const batchId = this.generateChangeId();
    this.currentBatch = {
      id: batchId,
      startTime: new Date(),
      operations: [],
      isActive: true,
      description,
    };

    return batchId;
  }

  /**
   * End the current batch operation
   */
  endBatch(): MealPlanChange | null {
    if (!this.currentBatch || this.currentBatch.operations.length === 0) {
      this.currentBatch = null;
      return null;
    }

    // Create a batch change that encompasses all operations
    const batchChange: MealPlanChange = {
      id: this.currentBatch.id,
      timestamp: this.currentBatch.startTime,
      type: 'batch',
      description: this.currentBatch.description,
      beforeState: this.currentBatch.operations[0].beforeState,
      afterState: this.currentBatch.operations[this.currentBatch.operations.length - 1].afterState,
      metadata: {
        batchId: this.currentBatch.id,
        userInitiated: true,
        affectedSlots: this.extractAffectedSlots(this.currentBatch.operations),
      },
    };

    this.addChange(batchChange);
    this.currentBatch = null;

    return batchChange;
  }

  /**
   * Undo the last change
   */
  undo(): { change: MealPlanChange; newState: MealPlanSnapshot } | null {
    if (!this.canUndo()) {
      return null;
    }

    const change = this.history[this.currentIndex];
    this.currentIndex--;

    return {
      change,
      newState: change.beforeState,
    };
  }

  /**
   * Redo the next change
   */
  redo(): { change: MealPlanChange; newState: MealPlanSnapshot } | null {
    if (!this.canRedo()) {
      return null;
    }

    this.currentIndex++;
    const change = this.history[this.currentIndex];

    return {
      change,
      newState: change.afterState,
    };
  }

  /**
   * Check if undo is possible
   */
  canUndo(): boolean {
    return this.currentIndex >= 0;
  }

  /**
   * Check if redo is possible
   */
  canRedo(): boolean {
    return this.currentIndex < this.history.length - 1;
  }

  /**
   * Get the complete change history
   */
  getHistory(): MealPlanChange[] {
    return [...this.history];
  }

  /**
   * Get recent changes (up to specified limit)
   */
  getRecentChanges(limit: number = 5): MealPlanChange[] {
    return this.history
      .slice(Math.max(0, this.history.length - limit))
      .reverse(); // Most recent first
  }

  /**
   * Get the current state index and total count
   */
  getState(): { currentIndex: number; totalChanges: number; canUndo: boolean; canRedo: boolean } {
    return {
      currentIndex: this.currentIndex,
      totalChanges: this.history.length,
      canUndo: this.canUndo(),
      canRedo: this.canRedo(),
    };
  }

  /**
   * Clear all history
   */
  clear(): void {
    this.history = [];
    this.currentIndex = -1;
    this.currentBatch = null;
    
    if (this.config.persistToStorage) {
      this.persistHistory();
    }
  }

  /**
   * Create a snapshot of the current meal plan state
   */
  createSnapshot(meals: { [key in DayOfWeek]?: MealSlotWithRecipe[] }, metadata?: {
    totalEstimatedTime?: number;
    completionPercentage?: number;
  }): MealPlanSnapshot {
    const snapshot: MealPlanSnapshot = {
      mealPlanId: this.mealPlanId,
      version: Date.now(),
      timestamp: new Date(),
      meals: {},
    };

    // Convert meal data to snapshot format
    Object.entries(meals).forEach(([day, dayMeals]) => {
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

    if (metadata) {
      snapshot.totalEstimatedTime = metadata.totalEstimatedTime;
      snapshot.completionPercentage = metadata.completionPercentage;
    }

    return snapshot;
  }

  /**
   * Generate human-readable description for common change types
   */
  generateDescription(
    type: MealPlanChange['type'],
    day?: DayOfWeek,
    mealType?: MealType,
    recipeName?: string,
    targetDay?: DayOfWeek,
    targetMealType?: MealType
  ): string {
    const dayLabel = day ? this.formatDay(day) : '';
    const mealLabel = mealType ? this.formatMealType(mealType) : '';
    const targetDayLabel = targetDay ? this.formatDay(targetDay) : '';
    const targetMealLabel = targetMealType ? this.formatMealType(targetMealType) : '';

    switch (type) {
      case 'substitution':
        return `Replaced ${dayLabel} ${mealLabel}${recipeName ? ` with ${recipeName}` : ''}`;
      
      case 'swap':
        return `Swapped ${dayLabel} ${mealLabel}${recipeName ? ` to ${recipeName}` : ''}`;
      
      case 'reorder':
        return `Moved ${dayLabel} ${mealLabel} to ${targetDayLabel} ${targetMealLabel}`;
      
      case 'lock':
        return `Locked ${dayLabel} ${mealLabel}`;
      
      case 'unlock':
        return `Unlocked ${dayLabel} ${mealLabel}`;
      
      case 'add':
        return `Added ${recipeName || 'recipe'} to ${dayLabel} ${mealLabel}`;
      
      case 'remove':
        return `Removed ${recipeName || 'recipe'} from ${dayLabel} ${mealLabel}`;
      
      case 'batch':
        return `Batch operation`;
      
      default:
        return 'Meal plan modified';
    }
  }

  // Private methods

  private generateChangeId(): string {
    return `change_${this.mealPlanId}_${Date.now()}_${++this.changeIdCounter}`;
  }

  private shouldBatchChange(change: MealPlanChange): boolean {
    if (!this.config.enableBatching) return false;

    // Always batch if there's an active batch
    if (this.currentBatch && this.currentBatch.isActive) {
      return true;
    }

    // Start batching for certain change types
    const batchableTypes: MealPlanChange['type'][] = ['reorder', 'lock', 'unlock'];
    return batchableTypes.includes(change.type);
  }

  private addToBatch(change: MealPlanChange): void {
    if (!this.currentBatch) {
      // Create implicit batch
      this.startBatch(`Multiple ${change.type} operations`);
    }

    if (this.currentBatch) {
      this.currentBatch.operations.push(change);

      // Auto-end batch after timeout
      setTimeout(() => {
        if (this.currentBatch && this.currentBatch.id === change.metadata?.batchId) {
          this.endBatch();
        }
      }, this.config.batchTimeout);
    }
  }

  private addChange(change: MealPlanChange): void {
    this.history.push(change);
    this.currentIndex = this.history.length - 1;
  }

  private enforceHistoryLimit(): void {
    if (this.history.length > this.config.maxHistoryLength) {
      const excessCount = this.history.length - this.config.maxHistoryLength;
      this.history = this.history.slice(excessCount);
      this.currentIndex = Math.max(-1, this.currentIndex - excessCount);
    }
  }

  private extractAffectedSlots(operations: MealPlanChange[]): Array<{ day: DayOfWeek; mealType: MealType }> {
    const slots: Array<{ day: DayOfWeek; mealType: MealType }> = [];
    
    operations.forEach(op => {
      if (op.metadata?.affectedSlots) {
        slots.push(...op.metadata.affectedSlots);
      }
    });

    // Remove duplicates
    return slots.filter((slot, index, self) => 
      index === self.findIndex(s => s.day === slot.day && s.mealType === slot.mealType)
    );
  }

  private formatDay(day: DayOfWeek): string {
    const dayMap: { [key in DayOfWeek]: string } = {
      monday: 'Monday',
      tuesday: 'Tuesday',
      wednesday: 'Wednesday',
      thursday: 'Thursday',
      friday: 'Friday',
      saturday: 'Saturday',
      sunday: 'Sunday',
    };
    return dayMap[day];
  }

  private formatMealType(mealType: MealType): string {
    return mealType.charAt(0).toUpperCase() + mealType.slice(1);
  }

  private persistHistory(): void {
    if (typeof localStorage !== 'undefined') {
      try {
        const historyData = {
          history: this.history,
          currentIndex: this.currentIndex,
          mealPlanId: this.mealPlanId,
          lastUpdated: new Date().toISOString(),
        };
        localStorage.setItem(`meal_plan_history_${this.mealPlanId}`, JSON.stringify(historyData));
      } catch (error) {
        console.warn('Failed to persist change history:', error);
      }
    }
  }

  private loadPersistedHistory(): void {
    if (typeof localStorage !== 'undefined') {
      try {
        const stored = localStorage.getItem(`meal_plan_history_${this.mealPlanId}`);
        if (stored) {
          const historyData = JSON.parse(stored);
          this.history = historyData.history || [];
          this.currentIndex = historyData.currentIndex ?? -1;
          
          // Convert string dates back to Date objects
          this.history.forEach(change => {
            change.timestamp = new Date(change.timestamp);
            change.beforeState.timestamp = new Date(change.beforeState.timestamp);
            change.afterState.timestamp = new Date(change.afterState.timestamp);
          });
        }
      } catch (error) {
        console.warn('Failed to load persisted change history:', error);
      }
    }
  }
}